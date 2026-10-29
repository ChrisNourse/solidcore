#### WR-474. `SmCurve::CalculateTightBoundingBox` adds tangent-discontinuous and last knots outside the queried interval
- **Severity:** Wrong result
- **Area:** Curves
- **Location:** `source/SMLib/src/SmCurve.cpp:10277`, `SmCurve::CalculateTightBoundingBox` (def. `:10248`).
- **What happens:** After adding the interval end points, the B-spline branch collects the tangent-discontinuous knots and the last knot to add to the tight box. The loop

  ```cpp
  for (ULONG i=1; i<sKnots.GetSize(); i++) {
      if (i==sKnots.GetSize()-1 || sContinuities[i] <= SM_CT_C1 ) {
          EvaluatePoint( sKnots[i], sPt );
          sTightBox.AddPoint3d( sPt );
      }
  }
  ```

  adds every such knot **regardless of whether the knot value lies inside `crInterval`**. Only the end points are clipped to the interval; the knots are not. A knot that is outside the queried interval is a point the curve never reaches over that interval, so including it enlarges the box to a point the edge does not cover. This is exactly the defect fixed in the Rust port as `CalculateTightBoundingBox holds knots inside the interval only` (issue #1054, PR `fix/calculate-tight-bounding-box-interval-only`): the Rust side keeps only the knots strictly inside the interval (`knot > interval.min() && knot < interval.max()`), which is the correct behaviour.
- **Trigger:** `CalculateTightBoundingBox` of a B-spline / ellipse curve over a sub-interval that excludes an interior tangent-discontinuous knot (or, for a curve whose last knot is outside the interval).
- **How found:** generated difftest case and the `tight_box_*` fixtures of `curve.curve_p9`. The pinned upstream reference (v1.0.1) returns the enlarged box; the corrected Rust port returns the smaller box. Reverting the Rust fix makes `curve.curve_p9` pass 3126/3126 against the reference, confirming the divergence is solely this knot check.
- **Confirmed:** yes, the `tight_box_ellipse_partial`, `tight_box_sub_ellipse_partial`, `tight_box_sub_ellipse_tilted` and `tight_box_degenerate_interval_ellipse` fixtures plus 49 generated cases of `unit.curve.curve_p9` (see `crates/difftest/known_mismatches/curve.curve_p9.toml`).
