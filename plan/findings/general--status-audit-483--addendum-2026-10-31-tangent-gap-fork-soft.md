Updates: plan/findings/general--status-audit-483.md

## `offset_tangent_gap_statuses` now follows the fork (#2009 group A)

The row for `HaveTangentGaps` said the first failed `NormalizedEvaluate` or
`ClosestPoint` propagates. The SolidCraft fork of SMLib (tangency-labs/smlib 4803d6d)
chose otherwise: `HaveTangentGaps` returns a boolean, so a sample it cannot evaluate is
skipped (upstream measured the unset point), and `ClosestPoint`'s status stays dropped
because its `bSuccess` says whether a closest point was found. SolidCraft held the flag
off because it errored where the fork's C++ carries on.

The flag was never turned on by a caller (no fix is shipped), so it changes meaning
under the same name rather than through a new one: with
`Kernel::fixes.offset_tangent_gap_statuses` set, `tangent_gap_samples`
(`crates/solidcore/src/offset/offset_geometry_creation.rs`) skips a sample whose
evaluation fails and ignores the closest-point status, as the fork does. Flag off is
unchanged (upstream: the unset point is measured). Tests:
`offset::offset_geometry_creation::tests::tangent_gaps_skip_a_sample_that_cannot_be_evaluated`
(an unset point that reads as a gap upstream shows none with the flag) and the updated
`audit_483_tangent_gaps_preserve_errors_and_no_solution`.
