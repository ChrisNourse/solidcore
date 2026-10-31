Updates: WR-100

#### WR-100 (update, 2026-10-31). The V degeneracy test is now confirmed by a measured native witness

- **Severity:** Wrong result
- **Area:** Intersection
- **Location:** `source/SMLib/src/SmSurfaceDropLib.cpp:4250` `SmSurface::DropCurve`:
  `bDegenSurfV = sUVDomain.XLength() < SM_EFF_ZERO` (the whole block is lines
  4244-4269; the line above, 4249, sets `bDegenSurfU` the same way).
- **What happens:** A drop domain degenerate in V is not reported
  (`SM_ERR_DEGENERATE_SURFACE`, `SM_CTC_DEGENSURFACE_FAIL`) and the drop goes on;
  `m_bDegenSurfV` of the failure record copies the U test.
- **Trigger:** Only a natural domain degenerate in V reaches it (a given domain shorter
  than `SM_EFF_ZERO` in either direction is replaced by the natural one), e.g. a
  B-spline surface whose V knot span is collapsed by `sm_FixupKnotVector`.
- **How found:** read during porting; this addendum records a native witness.
- **Confirmed:** yes, real C++ and Rust default outputs are identical on the three cases
  below (the C++ defect is faithfully ported, not a Rust divergence).

##### Measured witness (2026-10-31)

- **Tested binary:** `.slim/worktrees/new20-copy-faces-geometry` HEAD `6ca9deed1`
  (base `df64e8281`), its prebuilt `bin/debug/difftest` (04:23) and shim/static C++
  artifacts. This is the tested head, not `944`.
- **Source-of-truth worktree:** `.slim/worktrees/new20-face-points-domain` HEAD
  `9d8996e27` (base `9441664ac`). The 14 relevant semantic files (`surface_drop_lib`
  p1/p2/p3, `nurbs_crv`, `srf_basic`, `b_spline_surface/p1`, `surface/surface/p2`,
  `math`, both FFI p2/p3, both shims p2/p3, both difftest runners p2/p3) are byte
  identical between `6ca9deed1` and `9d8996e27`, so the prebuilt `6ca` binary is a
  valid proxy for the `944` source on this path.
- **Pin used by the witness:** `.slim/worktrees/reference-v101` at
  `3fd5b9548d19aa8c12c29bc9637a96d42a06f8a5`; line numbers above are from that pin.
  This implementation lane uses the separate sparse `reference-canonical/upstream`
  checkout at the same commit; it does not replace the witness's reference provenance.
- **Time:** witness artifacts written 2026-10-31 between 05:12 and 05:25 local; the
  full-output run was at about 05:16.
- **Rust toolchain:** the existing `bin/debug` rlibs were built by rustc `1.99.1`;
  `1.98.0` rejects them as incompatible, so `1.99.1` linked the temporary witness
  (a `rustc` build outside the repo). No crate, shim or C++ rebuild.

##### Inputs

Three JSON cases generated in the approved temp dir (all in
`.../T/opencode/wr100-proof/`), SHA-256 from `MANIFEST.sha256`:

| case | file SHA-256 |
|---|---|
| `vtiny_far.json` | `611e09cc927369284f813f3b36575d8e2b615995ee5e4097270d98e250d164b3` |
| `utiny_far.json` | `d909b15de68441dc5423670d45823caa0843aefcb45cdfda9f71fcea04625098` |
| `normal_far.json` | `3bc53fd525570e265ead25584eb8ea58a90c1ecc5166676f6999960fbaa79ab2` |
| `main.rs` (witness) | `f983a883ab4351a772870a10afe3bf10f828a39729856569ae0225385971ddd1` |
| `summary.json` | `08ccd75ad589d0e9a70f231e3743d48ac4b999fef94565a6441305464b01135d` |

Geometry: a non-rational degree-2x2 B-spline surface over a 3x3 net (all weights
`NOW = -f64::MAX`), normal U knots `[0,0,0,1,1,1]`; the V-tiny case uses
`[0,0,0,1e-15,1e-15,1e-15]` and the U-tiny case swaps them. The curve is a degenerate
(point) degree-1 B-spline with two coincident control points at `(10,10,10)`, interval
`[0,1]`, `tol=1e-4`, `keep_all=true`, `want_record=true`. The tiny knot span is
collapsed by `sm_FixupKnotVector` (`SM_EFF_ZERO_PARAM = 1.2e-8`) in both constructors,
so the measured natural length is exactly 0, not `1e-15`.

##### Bounds and limits

- Each case ran in its own process: `subprocess.Popen(..., start_new_session=True)`,
  `communicate(timeout=10)`, and `os.killpg(os.getpgid(pid), SIGKILL)` only on timeout.
  All three exited 0; no timeout or signal occurred and no background child remained.
  The first cold run took 3.1 s (497 MB static binary); a warm V-tiny re-run under a
  strict 5 s timeout took 0.105 s, and the existing `--fuzz-exec` binary took 0.123 s.
- Limits: only these three far-away point-curve cases; no near-surface/long-trace case
  and no optional U/V-swap case. The C++ natural UV domain is inferred to equal the Rust
  one (same fixup, identical outputs); it was not printed directly from C++. The scout's
  predicted `1032` path was not observed; the measured V-tiny path is `1020 / 19`.

##### Measured results (C++ == Rust exactly in all three)

| case | natural UV (public Rust) | expected degen (u,v) | measured degen (u,v) | fail_status | path | contained |
|---|---|---|---|---|---|---|
| `vtiny_far` | `[0,0,1,0]` (xlen 1, ylen 0) | (0, **1**) | (0, **0**) | 1020 | 19 FirstDropUncontainedFail | 1 |
| `utiny_far` | `[0,0,0,1]` (xlen 0, ylen 1) | (**1**, 0) | (**1**, **1**) | 1034 | 17 DegenSurfaceFail | 1 |
| `normal_far` | `[0,0,1,1]` (xlen 1, ylen 1) | (0, 0) | (0, 0) | 1020 | 19 FirstDropUncontainedFail | 1 |

The V-tiny case also measured `closed_v=1 (true)`, `srf_closure=V` and
`approx_singularities=15`: the drop went on instead of taking the early `1034` exit.

##### Rust port

The port reproduces the pinned behaviour by default
(`Surface::drop_curve_base` in `intersect/surface_drop_lib/p3.rs`). Per AGENTS §4b the
result-changing correction is opt-in: `Kernel::fixes.drop_curve_v_domain_degeneracy`
(`crates/solidcore/src/kernel.rs`), off by default. With it on, `degen_surf_v` tests
`s_uv_domain.y_length()` instead of `x_length()`. The V-tiny case then returns the
existing outer `SM_ERR` (1001) with record `fail_status = 1034` /
`DegenSurfaceFail` and flags `(degen_u, degen_v) = (0, 1)`; the U-only case keeps
`1034 / 17` but its V flag becomes 0; the normal-domain control is unchanged
(`1020 / 19`, flags `(0, 0)`). Regression tests:
`intersect::surface_drop_lib::p3::tests::wr100_*`; default-parity fixtures:
`fixtures/intersect.surface_drop_lib_p3/natural_domain_degeneracy.json`.
