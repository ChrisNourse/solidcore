# sweep.topology_sweep: bottle validity after UB-170 (2026-10-31)

PR: #1937. Units: `sweep.topology_sweep`, `sweep.trans_sweep_geometry`.
This is an unresolved defect, not a successful repair or a new difftest pass.

Input: pinned v1.0.1 `TestFiles/pt_TestFiles/Fillet/bottle.smb`, whole-body
translation `(0,0,4)`, six null target arrays, stitching and merge disabled,
one repetition, default kernel fixes and sweep epsilon.

The Rust reader and sweep now reproduce the private native diagnostic's result:

| Observation | Rust | Private native diagnostic |
|---|---|---|
| Before pointer validation | 1000 | 1000 |
| Before AssertValid return / reports | true / 8 | true / 8 |
| Sweep status | success | 1000 |
| After pointer validation | 1000 | 1000 |
| After AssertValid return / reports | false / 6 | false / 6 |

Three after-sweep reports are surviving input edge/surface tolerance gaps. The new
reports are surface type 15001 tests 1 (flat corner) and 6 (partially coincident
boundary), and trimming type 12225 test 2 (V endpoint gap). The native V gap is
`0.06984669271518007`, tolerance `0.0069846692715180075`; Rust's formatted report is
`0.069846693`, tolerance `0.0069846693`. No tolerance was increased.

The native reference without the private UB-170 correction crashes. The private
object changes only the `SmHCR.cpp` edge-copy loop bound to the edge-array size.
It is **not** the official parity reference. Rust's merged UB-170 correction uses
the minimum of the vertex and edge counts; the matching bottle results do not
establish equivalence of those two bounds on other inputs.

The reduced wire-edge case was also run directly against the **unmodified** native
libraries, without the private HCR object: `SmLine::CreateLineSegment`,
`SmBrep::CreateWireEdgeFromCurve` with interval `[0,1]`, then
`SmTranslationalSweepGeometry::EdgeSweepHigher` with analytics disabled. It returns
1000 and a surface whose AssertValid returns false with exactly type 15001 tests
1 and 6, matching the reduced Rust test. This case does not invoke HCR.

The failed extrusion's generator goes from
`(-0.30118110236220502,0,4.3331526241915306)` to
`(-0.30118297798525001,0,4.2633059315015203)` over native interval
`[0.71410363017993395,0.78395032289511402]`. Eleven native samples lie nearly on
that line. Its transverse displacement is about `1.876e-6`, below the B-rep's
`1e-5` tolerance. A Rust wire edge made from those endpoints reproduces both
surface failures without the bottle reader or HCR. This localizes the surface
failure to the accepted nearly parallel sweep, but does not prove that rejecting
or omitting the edge would preserve the whole swept solid.

Repeated validation is significant: a second Level0 check of the **unchanged**
Rust result returns pointer status 1001, with trimming tests 2, 3 and 4, and skips
AssertValid. The first check populated derived UV curves. The additional
orientation / zero-area reports identify face 101 in this traversal; the bad
extrusion is face 77, loop 117. These local arena indices are diagnostic, not stable
fixture identifiers. The repeated-check behavior has not been tested in C++.

Two diagnostic-only repairs did not establish a valid bottle:

- Replace the invalid face's support with the exact `y=0` plane and reproject its
  four boundary curves. All four drops report zero distance. With a local origin
  and scaled orthogonal parameters, the replacement face's loop computes `Same`
  orientation and the original UV gap disappears. Whole-body validation still
  rejects face 101's orientation / zero area, also exposed by the unchanged-body
  repeated check. Thus the plane experiment is promising for the surface and UV
  defects, but is not evidence of a valid whole-body repair.
- Call `brep::p5::squeeze_face` on the invalid face using the original body
  tolerance. It returns upstream status 1001 from `topology::brep::p3`, and the
  modified body fails pointer validation. The failures alone do not establish
  damage from squeezing, because repeated validation already exposes more
  failures without an edit; the failed repair status is sufficient to rule out
  claiming this experiment as a successful repair.

No experimental mutation is included in library code. An eventual result-changing
repair must be opt-in and demonstrate valid topology, not just remove surface
reports or return an error. The input already has tolerance-gap reports:
`BrepCheck::is_valid()` and the native AssertValid boolean must not be conflated.

## Reproduction

`crates/solidcore/tests/issue_1937_bottle_sweep.rs` characterizes the current defect.
The reduced test runs normally. The full-body test is ignored only because it
requires the untracked upstream fixture; run it explicitly:

```sh
CARGO_BUILD_JOBS=2 nice -n 10 cargo test --profile difftest -p solidcore \
  --test issue_1937_bottle_sweep -- --include-ignored --nocapture
```

Passing these characterization assertions means the defect is reproduced, not
fixed. The caller supplies distinct topology identities to both the reader and
the borrowed `SweepStore`, following the whole-body difftest runner. The imported
B-rep's own key is assigned the caller's body handle before registration; kernel
identity checks are unchanged.
