#### WR-441. `SmPlane::IsNurbSurfacePlane` offsets the origin before correcting its axes (#1056)
- **Severity:** Wrong result
- **Area:** Surfaces and volumes
- **Location:** `source/SMLib/src/SmPlane.cpp:2836-2854`, `SmPlane::IsNurbSurfacePlane` (pinned source `c9c5979`).
- **What happens:** The recognised plane uses an origin calculated with the original Y axis, then constructs a plane with the perpendicularised Y axis. Its evaluated point at the input surface's lower UV corner is displaced along X by approximately `-v_min * v_scale * shear` for a small X component in the Y tangent. The correction can also change the residual error across the rest of the patch; it does not make a sheared parameterisation exactly representable by an orthogonal plane.
- **Trigger:** A nearly perpendicular planar bilinear surface with a nonzero V-domain minimum. For a 40 mm V span, shear `3.2e-8`, and V minimum 4, the analytic corner displacement is `-5.12e-6` mm. A V minimum of -5 gives `+6.4e-6` mm.
- **How found:** Read during porting and reported in SolidCore issue #1056, based on a SolidCraft swept-groove regression.
- **Confirmed:** no; the pinned C++ source shows this operation order, but the local C++ reference and Rust tests have not run in this worktree.
