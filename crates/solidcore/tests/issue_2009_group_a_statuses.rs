//! #2009 group A: statuses SMLib drops and the SolidCraft fork passes on, behind
//! `UpstreamFixes::brep_edit_statuses_pass_on` (WR-654). No input fails these calls on
//! valid geometry, so each test injects the callee's failure and checks that the
//! status reaches the caller with the flag on and is dropped with it off.

#![allow(clippy::expect_used, clippy::indexing_slicing)] // test helpers

use solidcore::math::Point3;
use solidcore::primitive::api_primitives::create_box;
use solidcore::topology::brep::mirror::transform_matrix;
use solidcore::topology::Brep;
use solidcore::{Kernel, SolidError};

fn kernel(pass_on: bool) -> Kernel {
    let mut k = Kernel::default();
    k.fixes.brep_edit_statuses_pass_on = pass_on;
    k
}

fn unit_box(k: &mut Kernel) -> Brep {
    create_box(k, Point3::new(0.0, 0.0, 0.0), 1.0, 2.0, 3.0).expect("box")
}

/// fork 60a8a51 (`SmBrep::CalculateTightBoundingBox`, `SER(sFaceList[ii]->CalculateBoundingBox(..))`):
/// a face without surface has no box. Upstream unions the boxes it has; the fork
/// returns the face's error.
#[test]
fn tight_box_passes_on_a_failed_face_box() {
    for pass_on in [false, true] {
        let mut k = kernel(pass_on);
        let mut b = unit_box(&mut k);
        let f = b.face_list()[0];
        b.face_mut(f).expect("face").surface = None;
        let r = solidcore::query::brep::calculate_tight_bounding_box(&k, &mut b);
        if pass_on {
            assert!(
                matches!(
                    r,
                    Err(SolidError::InvalidInput {
                        operation: "query::face::p1::calculate_bounding_box",
                        ..
                    })
                ),
                "{r:?}"
            );
        } else {
            let bbox = r.expect("dropped status");
            assert_eq!(bbox.max(), Point3::new(1.0, 2.0, 3.0));
        }
    }
}

/// fork 60a8a51 (`SmBrep::Transform` of a matrix, `SER((svRow0 * svRow1).AngleBetween(svRow2, ..))`):
/// a zero first row has no angle. Upstream reads the angle as 0 and goes on with a
/// rigid transform built from the zero row; the fork returns the angle's error.
#[test]
fn matrix_transform_passes_on_a_failed_angle() {
    let m = [
        [0.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    for pass_on in [false, true] {
        let mut k = kernel(pass_on);
        let mut b = unit_box(&mut k);
        let r = transform_matrix(&mut k, &mut b, &m, true);
        if pass_on {
            assert!(
                matches!(
                    r,
                    Err(SolidError::InvalidInput {
                        operation: "geom::vector3d::Vec3::angle_between",
                        ..
                    })
                ),
                "{r:?}"
            );
        } else {
            assert!(r.is_ok(), "{r:?}");
        }
    }
}
