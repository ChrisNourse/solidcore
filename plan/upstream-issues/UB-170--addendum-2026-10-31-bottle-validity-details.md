Updates: UB-170

The bottle sweep validity failure recorded in the 2026-10-29 addendum is now
localized further on v1.0.1 (`3fd5b9548d19aa8c12c29bc9637a96d42a06f8a5`). After
the same private edge-array-bound correction, the native sweep succeeds but adds
three validation failures: `SmSurface` tests 1 and 6 (flat corner and partially
coincident boundary), and `SmTrimmingTools` test 2 (V endpoint gap
`0.06984669271518007`, tolerance `0.0069846692715180075`). Three additional reports
are surviving edge/surface gaps already present in the input.

The failing surface is a `SmSurfOfExtrusion` with vector `(0,0,4)`. Its generator
has endpoints `(-0.30118110236220502,0,4.3331526241915306)` and
`(-0.30118297798525001,0,4.2633059315015203)`, over interval
`[0.71410363017993395,0.78395032289511402]`. Eleven samples are nearly linear:
about `1.876e-6` transverse displacement over `0.0698467` vertically. The body
tolerance is `1e-5`.

A separate reduced probe against the unmodified native libraries confirms the
surface defect without HCR: create a line segment from those endpoints, create a
wire edge with interval `[0,1]`, and call `EdgeSweepHigher` with translation
`(0,0,4)` and analytics disabled. It returns 1000, followed by AssertValid false
and exactly surface tests 1 and 6. No private correction is linked into this probe.

Before sweep, `ValidatePointers` returns 1000 and `AssertValid` returns true,
despite eight tolerance-gap reports. After sweep, pointer validation still returns
1000 but `AssertValid` returns false with six reports. A larger sweep epsilon
(`1e-4`) returns 1001 rather than a valid bottle. No successful repair is claimed.
The private bounds correction remains diagnostic only; the unmodified native
reference crashes on this input. See PR #1937 and its Rust investigation record.
