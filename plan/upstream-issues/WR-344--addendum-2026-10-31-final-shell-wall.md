Updates: WR-344

#### WR-344 (update, 2026-10-31). The mated-edge opt-in also refuses a non-solid final shell wall

`Kernel::fixes.offset_extended_edges_mated` (default `false`, upstream behaviour) now
also refuses (`SM_ERR`/1001, operation `offset::offset_executive::p2::shell_brep`) the
final `shell_difference` result of a manifold input when it is not a manifold solid.
This is a refinement of the port's opt-in, not a new pinned-C++ defect: the pinned C++
1.0.1 already refused the #1818 input with 1001, and the guard prevents the existing
relaxation flag from turning that refusal into a broken body. The check is
`offset_extended_edges_mated && input_is_manifold && !brep_is_manifold_solid(&shell)`
after `make_shell_edges_exact`; it does not test `create_offset_solid`, so it covers the
final difference result in both the solid-wall and closed-shell calls. It runs after
`shell_difference` has consumed the input, so the refusal reports `retained_input =
None` and `input_ownership_transferred() == true`; the early open-sheet return
(`create_offset_solid = false`, non-manifold intermediate) and the sheet-thickening
branch are unchanged.

Measured on base `9112d3a0ea7a72a090ff812df95839f35cafaba6` with
`crates/solidcore/tests/issue_1818_final_shell_wall.rs`: square 40 x 40 at z = 0 to
circle R15 at z = 60, four curves each, `create_skin_primitive` (6 faces, 12 edges,
8 vertices, integrity `Ok`, manifold, volume `67644.02049209057`); shell -1 inward, top
open at z = 60, `extend_convex_edges = true`, `merge_results = true`,
`create_offset_solid = true`. With only `offset_extended_edges_mated` on, the final
wall was `Ok` with 10 faces, 16 edges, 16 vertices, integrity `Ok`, `manifold = false`
and volume `0.0`; with the guard it is the `SM_ERR` refusal above. With the flag off,
the same base refuses in `boolean::merge::p2::make_faceuse` (1001) and retains the
input (`retained_input = Some`, `input_ownership_transferred() == false`); the guard is
gated by the flag and leaves that path unchanged. The circle-to-circle R15 control
stays a manifold solid at `6082.125217` mm3 (analytic `6082.123377`, within `1e-6`
relative) with the flag on and off.

This does not repair the mixed-loft geometry, the missing manifold/volume validity of
the underlying shell, or a closed manifold zero-volume wall; those remain out of scope.
Test `crates/solidcore/tests/issue_1818_final_shell_wall.rs`; existing mated-only
canaries `issue_469_extended_offset_mated_edges_not_intersected_again`,
`issue_1795_extended_edges_mated`, and
`issue_1476_swept_bar_shells_with_solidcraft_fixes` must stay green. Fixes
tangency-labs/solidcore#1818.
