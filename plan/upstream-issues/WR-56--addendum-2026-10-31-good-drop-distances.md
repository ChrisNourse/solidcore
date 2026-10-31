Updates: WR-56

#### WR-56 (update, 2026-10-31). Opt-in comparison of the actual guess and result distances
- **What happens:** The last test in `sm_GoodDropResult` intends to reject a drop whose
  result is farther from the point than the guess, but `dDsqGuess` holds the distance to
  the result point and `dDsqResult` the distance to the guess point. The comparison is
  therefore reversed: a drop that improved on the guess is rejected (and the expensive
  global solve runs), while one that moved away is accepted.
- **Confirmed:** no. The C++ at the pin still has the swap; the Rust test below exercises
  the final comparison in both directions without a C++ run.
- **Rust port:** `Kernel::fixes.surface_drop_checks_actual_guess_distance` (opt-in, off
  by default) uses the actual guess point for `dsq_guess` and the result point for
  `dsq_result`, so only a result farther than the guess is rejected. With the flag off,
  `good_drop_result` keeps upstream's swapped names, numeric operation order and branch
  structure unchanged. The converged-foot part of the fix (the fork's #70) is already
  present at the pin; this addendum covers only the final distance comparison.
- **Test:** `issue_1852_good_drop_compares_the_actual_guess_distance` in
  `crates/solidcore/src/surface/surface/p7.rs`. A plane drop whose result is nearer than
  the guess, one whose guess is nearer than the result, and an equidistant case all reach
  the final comparison (`Fu = 1/2048`, `Fv = 0`, interior parameters). Flag off reproduces
  upstream's accept/reject directions; flag on accepts the nearer result, rejects the
  farther one, and leaves the equidistant case accepted.
- Fixes tangency-labs/solidcore#1852.
