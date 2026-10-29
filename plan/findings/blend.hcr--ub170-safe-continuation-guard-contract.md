# blend.hcr: UB-170 safe continuation disagrees with the native guard (2026-10-29)

Unit: `blend.hcr`. PR: #1487. Base: `16af20891a` (claim commit).

The proposed bounded edge iterator completes the focused two-vertex, one-edge
regression that previously returned `InvalidInput`. It preserves the vertex-count
limit when that limit is in bounds. The native reference and its UB guards are
unchanged, as are comparison tolerances and generated cases.

Reproduce with `cargo run --profile difftest -p difftest --features cpp -- --unit blend.hcr`.
The default 400 generated seeds plus fixtures give 434 cases: 417 pass, 17 fail,
0 deferred. Each failure has native status -2 and no record (the shim refuses an
undefined native path), versus Rust status 0 and a nonempty classification or
visibility record. These are guard-contract differences, not successful native
comparisons of the corrected continuation. No cases have been parked or removed.

| Case | Generated seed | Native record length | Rust record length |
| --- | --- | --- | --- |
| gen0014 | 10620584185765354333 | 0 | 35 |
| gen0062 | 10620584185765354381 | 0 | 24 |
| gen0114 | 10620584185765354433 | 0 | 18 |
| gen0128 | 10620584185765354447 | 0 | 18 |
| gen0163 | 10620584185765354482 | 0 | 18 |
| gen0175 | 10620584185765354494 | 0 | 18 |
| gen0183 | 10620584185765354502 | 0 | 18 |
| gen0197 | 10620584185765354516 | 0 | 24 |
| gen0237 | 10620584185765354556 | 0 | 24 |
| gen0251 | 10620584185765354570 | 0 | 18 |
| gen0256 | 10620584185765354575 | 0 | 18 |
| gen0279 | 10620584185765354598 | 0 | 18 |
| gen0289 | 10620584185765354608 | 0 | 24 |
| gen0294 | 10620584185765354613 | 0 | 18 |
| gen0306 | 10620584185765354625 | 0 | 18 |
| gen0318 | 10620584185765354637 | 0 | 24 |
| gen0367 | 10620584185765354686 | 0 | 18 |

Suspected cause: `icws_undefined` in `harness/cpp/shim/blend_hcr.cpp`
returns status -2 when the vertex query is larger than the edge query. The Rust
runner previously mapped the kernel's UB-170 `InvalidInput` to the same sentinel.
The corrected kernel now completes that path. The harness needs an explicit,
reviewed way to distinguish an undefined native comparison from a Rust failure;
changing or removing the C++ crash guard is not safe. Until that is resolved,
this proposed change is blocked by the existing gate and must not be called green.

The affected sweep caller passes: 703 total, 694 passed with values, 0 failed,
0 deferred, 2 rejected and 7 already parked ND-491 cases; max relative error
3.62089958464889e-14. This does not establish the complete bottle-sweep repair:
the private corrected native probe still produced an invalid bottle result, as
recorded in `plan/upstream-issues/UB-170--addendum-2026-10-29-bottle-sweep.md`.
