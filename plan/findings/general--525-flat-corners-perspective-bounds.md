# General: #525 flat corners and perspective bounds (2026-10-25, PR #684)

- **Rust defects reproduced before the fix.** Both flat-corner regression tests
  failed: requesting points changed masks `1` and `15` to `0`. The perspective
  regression failed because a point inside the second box was culled. These are
  correctness fixes that intentionally differ from the unchanged C++ reference.
- **Flat corners (WR-55).** The optional point loop now tests `flags & bit` without
  modifying the accumulated mask. A bilinear net `(0,0), (1,0); (2,0), (1,1)` has
  only the first flat corner; reversing either net direction moves that corner to
  each of the other bits. A collinear net `S(u,v)=(u+2v,0,0)` has all four corners.
  With/without-points results agree; only flagged points are evaluated, and regular
  plane corners remain undefined. This supersedes the historical “quirks kept”
  note above. C++ returns mask zero whenever points are requested and skips later
  corner points. Its unchanged no-points results agree with Rust.
- **Perspective bounds (WR-260).** Each sphere now uses its own radius and center.
  Reading the vector projection API also exposed an extra center addition on both
  spheres: `PerspectiveProjectToPlane` expects a radius **vector**, and itself adds
  the base point. The port now passes the offset instead of `center + offset`.
  The four sampling directions, tolerance and comparisons are unchanged. This is
  not a claim that the inherited four-sample approximation is a general exact
  projected-sphere bound.
- **Small analytic reproduction.** Eye `(0,0,0)`, plane `z=1`, point
  `(1.75,0,2)`, and the box segment `[(0,0,2),(2,0,2)]`. The point lies inside the
  second box, and their projections overlap at `x=.875`; C++ culls it and returns
  no node solutions, Rust retains it and returns one. Swapped-tree, coincident-point
  and separated-point/small-box controls are included.
- **Existing generated branch evidence.** `geom.global_solver/gen0207`, seed
  `8717372228313210103`, changes from 52 C++ leaf combinations to 50 Rust ones.
  Only pairs `(38,1)` and `(38,2)` change cull decisions. Independently projecting
  all eight bounding-box corners gives viewing-plane z gaps `2.235484266590634`
  and `2.0536994213256827`, both greater than the supplied tolerance `1`. Thus these
  two pairs are actually separated; the old incorrect bounds kept unnecessary work.
- **Actual curve caller.** `curve.curve_p9/gen0415`, seed `7577090626449490212`,
  previously compared an empty C++ result to one Rust intersection. A new direct
  Rust regression recreates that input and checks that solutions exist, both
  parameters stay inside their requested intervals, and the two evaluated points
  perspective-project within the caller's unchanged `0.01` tolerance. Thus the
  extra solution is validated geometrically, not accepted merely for disagreeing
  with C++. It is the caller's one known reference mismatch; the unit stays partial.
- **Native validation:** `surface.surface_p7`: 3738 / 3725 exact / 13 known;
  `geom.global_solver`: 628 / 626 exact / 2 known; `curve.curve_p9`: 3126 /
  3125 exact / 1 known. `heal.face_props`: 1528 / 1431 exact / 97 pre-existing
  dependency deferrals. No unexpected failures; all comparisons retain their
  existing tolerances. Sixteen reference differences in total stay visible.
- **Local gates on BCC main plus the fix:** fmt, default and native workspace
  Clippy `-D warnings`, 2077 kernel tests (two existing primitive integration tests
  ignored), wasm32 check, and 46 difftest tests plus its float-roundtrip regression
  passed. Positive controls detect perturbations in all five no-points mask cases
  and both separated-point projection cases. Full macOS `--green` is not claimed:
  the previously documented reference worker/reader defects (#448–450) remain;
  canonical CI needs maintainer approval for the fork PR.
- **Revalidated after rebasing onto main `94abd96b5` (PR #680):** all Rust gates
  above pass again with 2077 tests. A freshly rebuilt native runner retains all
  four unit counts above, and the newly landed `intersect.curve_class_p7` also
  passes all 1516 cases exactly. Both perturb checks remain sensitive (5/5 and
  2/2). This checks the actual rebased production source, not an older binary.
- **Comparison evidence.** The following exact case differences remain individually
  listed in `crates/difftest/known_mismatches/<unit id>.toml`. None is counted as a pass, no tolerance or
  reference output was modified, and both units are partial.

| Unit | Case | First field | C++ | Rust |
|---|---|---|---|---|
| `curve.curve_p9` | `gen0415` | `values.out.len` | `1` (no solutions) | `9` (one solution) |
| `surface.surface_p7` | `flat_mask_1_points_1` | `values.out[1]` | `0e0` | `1e0` |
| `surface.surface_p7` | `flat_mask_2_points_1` | `values.out[1]` | `0e0` | `2e0` |
| `surface.surface_p7` | `flat_mask_4_points_1` | `values.out[1]` | `0e0` | `4e0` |
| `surface.surface_p7` | `flat_mask_8_points_1` | `values.out[1]` | `0e0` | `8e0` |
| `surface.surface_p7` | `flat_mask_all_points_1` | `values.out[1]` | `0e0` | `1.5e1` |
| `surface.surface_p7` | `gen0191` | `values.out[1]` | `0e0` | `1e0` |
| `surface.surface_p7` | `gen0796` | `values.out[1]` | `0e0` | `1e0` |
| `surface.surface_p7` | `gen1186` | `values.out[1]` | `0e0` | `1.5e1` |
| `surface.surface_p7` | `gen1584` | `values.out[1]` | `0e0` | `1.5e1` |
| `surface.surface_p7` | `gen2327` | `values.out[1]` | `0e0` | `1e0` |
| `surface.surface_p7` | `gen2351` | `values.out[1]` | `0e0` | `1.5e1` |
| `surface.surface_p7` | `gen2373` | `values.out[1]` | `0e0` | `1e0` |
| `surface.surface_p7` | `gen2405` | `values.out[1]` | `0e0` | `1.5e1` |
| `geom.global_solver` | `perspective_unequal_containing_swap_0` | `values.floats.len` | `3` | `25` |
| `geom.global_solver` | `gen0207` | `values.floats.len` | `1147` | `1103` |
