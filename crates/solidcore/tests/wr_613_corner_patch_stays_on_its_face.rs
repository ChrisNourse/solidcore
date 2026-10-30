//! WR-613 (#1836; the fork's tangency-labs/smlib@7fd9fb0, from the issue's diff): the
//! fillet corner patch where a bar's top-edge round meets the concave round at a
//! cylinder the bar runs tangent into crosses the bar's side face. The part is
//! WR-593's peg: a stadium bar 10 high (sides y = -5 and y = 5, x 0..20) whose top meets
//! a cylinder of the bar's end radius standing on its round end at x = 20. The concave
//! half circle where the top meets the cylinder's wall and the bar's front top edge
//! (y = -5, z = 10) are rounded r 2 together, meeting at (20, -5, 10) in a 3x2
//! mixed-convexity corner. One side of its Coons patch lies on the front face, the
//! other on a round; `sm_CreateCoonsSrf` gives their corner the mean of the two twists,
//! which tilts the patch off the face beside that side, so the body's tight box pokes
//! out past y = -5. With `corner_patch_face_side_twist` the corner takes the round's
//! twist and the patch meets the face as the face's own twist has it.
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)] // test helpers
use solidcore::blend::fillet_executive::api::fillet_edges;
use solidcore::boolean::api_brep::boolean_union;
use solidcore::curve::curve::Curve;
use solidcore::math::{Point3, Vec3};
use solidcore::primitive::api_curves as curves;
use solidcore::query::api_queries::{brep_bounding_box, get_edges};
use solidcore::sweep::api_brep::create_linear_sweep;
use solidcore::topology::brep::p2::is_manifold_solid;
use solidcore::topology::{edge, EdgeId};
use solidcore::{Brep, Kernel};

const R: f64 = 5.0;
const L: f64 = 20.0;
const H: f64 = 10.0;
const RADIUS: f64 = 2.0;

fn sweep(k: &mut Kernel, cs: &[Curve], h: f64) -> Brep {
    let refs: Vec<Option<&Curve>> = cs.iter().map(Some).collect();
    create_linear_sweep(k, &refs, Vec3::new(0.0, 0.0, 1.0), h, true).unwrap()
}

/// WR-593's peg: the stadium bar extruded 0..H and the cylinder about x = L, H..2H,
/// made of two half-circle walls whose seams are at the bar's tangent points.
fn peg(k: &mut Kernel) -> Brep {
    let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
    let line = |a: Point3, b: Point3| Curve::from(curves::create_line_segment(a, b).unwrap());
    let arc = |c: Point3, s: f64, e: f64| Curve::from(curves::create_arc(c, R, s, e).unwrap());
    let bar = [
        line(p(0.0, -R, 0.0), p(L, -R, 0.0)),
        arc(p(L, 0.0, 0.0), -90.0, 90.0),
        line(p(L, R, 0.0), p(0.0, R, 0.0)),
        arc(p(0.0, 0.0, 0.0), 90.0, 270.0),
    ];
    let bar = sweep(k, &bar, H);
    let post = [
        arc(p(L, 0.0, H), 90.0, 270.0),
        arc(p(L, 0.0, H), -90.0, 90.0),
    ];
    let post = sweep(k, &post, H);
    boolean_union(k, bar, post).unwrap()
}

/// The edges at z = H whose ends match `ends`.
fn edges_at_top(b: &Brep, ends: impl Fn(Point3, Point3) -> bool) -> Vec<EdgeId> {
    get_edges(b)
        .unwrap()
        .into_iter()
        .filter(|&e| {
            let (Some(s), Some(t)) = edge::start_end_vertices(b, e).unwrap() else {
                return false;
            };
            let (s, t) = (b.vertex(s).unwrap().point, b.vertex(t).unwrap().point);
            (s.z - H).abs() < 1e-6 && (t.z - H).abs() < 1e-6 && ends(s, t)
        })
        .collect()
}

/// The concave half circle from (L, R) to (L, -R) on the bar's side of the cylinder and
/// the bar's front top edge from (0, -R) to (L, -R).
fn rounded_edges(b: &Brep) -> Vec<EdgeId> {
    let arc = edges_at_top(b, |s, t| {
        (s.x - L).abs() < 1e-6 && (t.x - L).abs() < 1e-6 && (s.y - t.y).abs() > R
    });
    assert_eq!(arc.len(), 1, "one concave arc");
    let front = edges_at_top(b, |s, t| {
        (s.y + R).abs() < 1e-6 && (t.y + R).abs() < 1e-6 && (s.x - t.x).abs() > L - 1e-6
    });
    assert_eq!(front.len(), 1, "one front top edge");
    vec![arc[0], front[0]]
}

/// How far the rounded peg's tight box reaches past the front face y = -R.
fn past_front_face(face_side_twist: bool) -> f64 {
    let mut k = Kernel::default();
    k.fixes.fillet_nx1_tangent_side_edge = true;
    k.fixes.fillet_nx1_tangent_corner_edge = true;
    k.fixes.corner_patch_face_side_twist = face_side_twist;
    let mut body = peg(&mut k);
    let edges = rounded_edges(&body);
    fillet_edges(&mut k, &mut body, &edges, RADIUS, 1, 1, 1.0).unwrap();
    assert!(is_manifold_solid(&body).unwrap());
    let x = brep_bounding_box(&k, &body, true).unwrap();
    -R - x.min_at(1)
}

#[test]
fn wr_613_corner_patch_stays_on_the_front_face() {
    let past = past_front_face(true);
    assert!(
        past < 1e-4,
        "the corner patch is {past} past the front face"
    );
}

/// Without the opt-in the mean twist takes the patch past the face, as upstream.
#[test]
fn wr_613_mean_twist_crosses_the_front_face() {
    let past = past_front_face(false);
    assert!(
        past > 1e-3,
        "the corner patch is only {past} past the front face"
    );
}
