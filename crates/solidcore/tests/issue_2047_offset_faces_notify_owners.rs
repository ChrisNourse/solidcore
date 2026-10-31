//! tangency-labs/solidcore#2047: `OffsetFacesExecutive::offset_faces` on a
//! `NamingScheme::Notify` body (a primitive, or a `brep_copy` of one) came back with
//! every face's surface owned under the face's arena identity while the body stayed
//! `Notify`, so `check_brep` failed `SmFace` test #35 on every face. The local
//! operation's copy back into the body now names the faces by the body's own scheme.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)]

use solidcore::geom::attribute::{AttributeStore, ObjectKey};
use solidcore::mass::integrator::properties::{brep_volume, PropertyAttributes};
use solidcore::offset::offset_faces::OffsetFacesExecutive;
use solidcore::primitive::api_primitives::create_box;
use solidcore::topology::brep::p1;
use solidcore::topology::brep::p3::brep_copy;
use solidcore::topology::brep::validation::{check_brep, CheckLevel};
use solidcore::topology::{Brep, NamingScheme};
use solidcore::{Kernel, Point3};

/// Moves the first face of `b` 2 into the body and checks the 10 x 10 x 8 result.
fn move_first_face_in(k: &mut Kernel, mut b: Brep) {
    assert_eq!(b.naming(), NamingScheme::Notify);
    let before = check_brep(k, &mut b, CheckLevel::Level2).unwrap();
    assert_eq!(before.assert_valid, Some(true));
    let faces = p1::faces(&b, None).unwrap();
    let mut ex = OffsetFacesExecutive::new();
    ex.offset_faces(k, &mut b, &faces[..1], -2.0).unwrap();
    let after = check_brep(k, &mut b, CheckLevel::Level2).unwrap();
    assert_eq!(after.assert_valid, Some(true), "{after:?}");
    let store = AttributeStore::default();
    let attrs = PropertyAttributes {
        store: &store,
        key_of: &|_| ObjectKey(0),
    };
    let volume = brep_volume(k, &mut b, 1e-9, &attrs).unwrap();
    assert!((volume - 800.0).abs() <= 1e-9 * 800.0, "volume {volume}");
}

#[test]
fn offset_faces_on_notify_box_keeps_owners() {
    let mut k = Kernel::default();
    let b = create_box(&mut k, Point3::ZERO, 10.0, 10.0, 10.0).unwrap();
    move_first_face_in(&mut k, b);
}

#[test]
fn offset_faces_on_brep_copy_keeps_owners() {
    let mut k = Kernel::default();
    let b = create_box(&mut k, Point3::ZERO, 10.0, 10.0, 10.0).unwrap();
    let copy = brep_copy(&mut k, &b).unwrap();
    move_first_face_in(&mut k, copy);
}
