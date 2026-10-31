//! tangency-labs/solidcore#1818: an inward shell of the valid square-to-circle loft
//! (40 x 40 at z = 0 to circle R15 at z = 60) returned `Ok` with a non-manifold,
//! zero-volume wall when only `Kernel::fixes.offset_extended_edges_mated` was on.
//! `create_offset_solid` is true there, so this is not the intended early open-sheet
//! return (`p2.rs` returns the intermediate non-solid only when it is false). The final
//! `shell_difference` is now checked before `Ok` when the mated-offset flag is on.
//!
//! The mated mixed test pins the new refusal at `shell_brep`'s final-wall guard
//! (`SM_ERR`/1001, operation `shell_brep`), not at an earlier unrelated failure, and
//! pins the ownership boundary: the guard runs after `shell_difference` consumed the
//! input, so `retained_input` is `None` and `input_ownership_transferred()` is true.
//! The default mixed test records the pre-existing refusal without the flag (the
//! pinned path is unchanged). Volume is diagnostic only for the mixed results: the
//! guard checks manifoldness, and a closed manifold zero-volume wall would be a
//! different defect. The circle controls (mated and default) keep the closed-form
//! volume check.
//!
//! An open-sheet control with this same mixed loft and `create_offset_solid = false`
//! does not reach the early open return: the intermediate is manifold, the final
//! `shell_difference` runs, and the new guard refuses its non-manifold result. The
//! early return is therefore not exercised by this input; no parameter search was done.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // test helpers

#[path = "support/offset_fixture.rs"]
mod offset_fixture;

use core::f64::consts::PI;

use offset_fixture::{planar_faces_at, volume, Notifications};
use solidcore::curve::curve::Curve;
use solidcore::curve::ellipse::Ellipse;
use solidcore::error::SM_ERR;
use solidcore::geom::axis2_placement::Axis2Placement;
use solidcore::geom::extent1d::Extent1d;
use solidcore::math::Vec3;
use solidcore::offset::offset_executive::p1::OffsetExecutive;
use solidcore::offset::offset_executive::p2::{OffsetOptions, ShellOptions, ShellOutcome};
use solidcore::primitive::api_curves::create_line_segment;
use solidcore::query::api_queries::brep_is_manifold_solid;
use solidcore::sweep::api_surfaces::create_skin_primitive;
use solidcore::{Brep, Kernel, SolidError};

/// The operation string `shell_brep` uses for its own `SM_ERR` refusals.
const OP_SHELL: &str = "offset::offset_executive::p2::shell_brep";

/// The #1818 loft: a square 40 x 40 (or circle R15) at z = 0 to a circle R15 at
/// z = 60; four curves per section, capped, linear skin.
fn loft(k: &mut Kernel, square: bool) -> Brep {
    let mut curves = Vec::new();
    let xy = [[-20.0, -20.0], [20.0, -20.0], [20.0, 20.0], [-20.0, 20.0]];
    for section in 0..2 {
        for (i, point) in xy.iter().enumerate() {
            let c = if square && section == 0 {
                Curve::from(
                    create_line_segment(
                        Vec3::new(point[0], point[1], 0.0),
                        Vec3::new(xy[(i + 1) % 4][0], xy[(i + 1) % 4][1], 0.0),
                    )
                    .expect("square section edge"),
                )
            } else {
                let start = (-135.0 + f64::from(u32::try_from(i).expect("curve index")) * 90.0)
                    * PI
                    / 180.0;
                let frame = Axis2Placement::new(
                    Vec3::new(0.0, 0.0, f64::from(section) * 60.0),
                    Vec3::new(start.cos(), start.sin(), 0.0),
                    Vec3::new(-start.sin(), start.cos(), 0.0),
                );
                Curve::from(
                    Ellipse::create_canonical_circle(
                        k,
                        &frame,
                        15.0,
                        Some(&Extent1d::from_min_max(0.0, 90.0)),
                        None,
                    )
                    .expect("circle section arc"),
                )
            };
            curves.push(c);
        }
    }
    let refs: Vec<_> = curves.iter().map(Some).collect();
    let mut part = create_skin_primitive(k, &[4, 4], &refs, 1, true)
        .expect("square-to-circle skin")
        .brep;
    part.refresh_cached_topology();
    part
}

/// Shells `part` 1 mm inward, open at its z = 60 face, as SolidCraft calls it.
fn shell_outcome_at_top(k: &mut Kernel, mut part: Brep, create_offset_solid: bool) -> ShellOutcome {
    let top = planar_faces_at(k, &part, 2, 60.0);
    assert_eq!(top.len(), 1, "one open top face at z = 60");
    let mut notifications = Notifications::new(&mut part);
    OffsetExecutive::shell_brep(
        k,
        part,
        ShellOptions {
            offset: OffsetOptions {
                distance: -1.0,
                extend_convex_edges: true,
                merge_results: true,
            },
            create_offset_solid,
            merge_sheet_side_walls: false,
        },
        &top,
        &mut notifications.context(),
    )
}

/// The issue's input checks: intact, manifold, positive volume.
fn check_input(k: &Kernel, part: &mut Brep, label: &str) -> f64 {
    assert!(
        solidcore::topology::integrity::check(part).is_ok(),
        "{label} input integrity"
    );
    assert!(
        brep_is_manifold_solid(part).expect("input manifold query"),
        "{label} input is a manifold solid"
    );
    let input_volume = volume(k, part).expect("input volume");
    assert!(input_volume > 0.0, "{label} input volume {input_volume}");
    eprintln!("INPUT {label}: integrity=Ok manifold=true volume={input_volume}");
    input_volume
}

/// The new final-wall refusal, distinguished from an earlier unrelated error.
fn assert_final_wall_refusal(error: &SolidError) {
    match error {
        SolidError::Upstream {
            operation,
            code,
            entity,
        } => {
            assert_eq!(
                *operation, OP_SHELL,
                "refusal must come from shell_brep's final-wall guard"
            );
            assert_eq!(*code, SM_ERR, "final-wall refusal must be SM_ERR/1001");
            assert_eq!(*entity, None);
        }
        other => panic!("expected the final-wall SM_ERR refusal, got {other:?}"),
    }
}

#[test]
fn issue_1818_mated_mixed_refuses_at_final_wall_guard() {
    let mut k = Kernel::default();
    k.fixes.offset_extended_edges_mated = true;
    let mut part = loft(&mut k, true);
    check_input(&k, &mut part, "mixed square-to-circle (mated)");

    let outcome = shell_outcome_at_top(&mut k, part, true);
    let retained = outcome.retained_input.is_some();
    let transferred = outcome.input_ownership_transferred();
    match outcome.result {
        Err(e) => {
            eprintln!(
                "RESULT mated mixed: Err({e:?}) retained_input={retained} input_ownership_transferred={transferred}"
            );
            assert_final_wall_refusal(&e);
            assert!(
                !retained,
                "the final-wall refusal must not retain the consumed input"
            );
            assert!(
                transferred,
                "the final-wall refusal must report ownership transferred"
            );
        }
        Ok(mut wall) => {
            wall.refresh_cached_topology();
            let manifold = brep_is_manifold_solid(&wall);
            panic!(
                "expected the final-wall refusal, got Ok: manifold={manifold:?} faces={} edges={} vertices={} retained_input={retained} input_ownership_transferred={transferred}",
                wall.face_list().len(),
                wall.edge_list().len(),
                wall.vertex_list().len()
            );
        }
    }
}

#[test]
fn issue_1818_default_mixed_refuses_before_final_wall_guard() {
    let mut k = Kernel::default();
    let mut part = loft(&mut k, true);
    check_input(&k, &mut part, "mixed square-to-circle (default)");

    let outcome = shell_outcome_at_top(&mut k, part, true);
    let retained = outcome.retained_input.is_some();
    let transferred = outcome.input_ownership_transferred();
    match outcome.result {
        Err(e) => {
            eprintln!(
                "RESULT default mixed: Err({e:?}) retained_input={retained} input_ownership_transferred={transferred}"
            );
            assert!(
                matches!(
                    &e,
                    SolidError::Upstream {
                        operation: "boolean::merge::p2::make_faceuse",
                        code: SM_ERR,
                        ..
                    }
                ),
                "default must retain its existing make_faceuse refusal: {e:?}"
            );
            assert!(retained, "the earlier default refusal retains its input");
            assert!(
                !transferred,
                "the default refusal does not consume its input"
            );
        }
        Ok(mut wall) => {
            wall.refresh_cached_topology();
            let manifold = brep_is_manifold_solid(&wall);
            panic!(
                "default mixed expected the pinned refusal, got Ok: manifold={manifold:?} faces={}",
                wall.face_list().len()
            );
        }
    }
}

fn check_circle_control(k: &mut Kernel, label: &str) {
    let mut part = loft(k, false);
    check_input(k, &mut part, label);

    let mut wall = shell_outcome_at_top(k, part, true)
        .result
        .unwrap_or_else(|e| panic!("{label} shell: {e:?}"));
    wall.refresh_cached_topology();
    let integrity = solidcore::topology::integrity::check(&wall);
    let manifold = brep_is_manifold_solid(&wall);
    let got = volume(k, &mut wall);
    let analytic = PI * (15.0 * 15.0 * 60.0 - 14.0 * 14.0 * 59.0);
    eprintln!(
        "RESULT {label}: integrity={integrity:?} manifold={manifold:?} faces={} edges={} vertices={} volume={got:?} analytic={analytic}",
        wall.face_list().len(),
        wall.edge_list().len(),
        wall.vertex_list().len()
    );
    assert!(integrity.is_ok(), "{label} wall topology: {integrity:?}");
    assert_eq!(manifold, Ok(true), "{label} wall is not a manifold solid");
    let got = got.unwrap_or_else(|e| panic!("{label} volume: {e:?}"));
    assert!(
        (got - analytic).abs() <= 1e-6 * analytic,
        "{label} volume {got} vs analytic {analytic}"
    );
}

#[test]
fn issue_1818_circle_control_mated_is_a_manifold_solid() {
    let mut k = Kernel::default();
    k.fixes.offset_extended_edges_mated = true;
    check_circle_control(&mut k, "circle R15 control (mated)");
}

#[test]
fn issue_1818_circle_control_default_is_a_manifold_solid() {
    let mut k = Kernel::default();
    check_circle_control(&mut k, "circle R15 control (default)");
}
