Updates: UB-252

PR: https://github.com/tangency-labs/solidcore/pull/644

### UB-252 follow-up / issue #458: temporary radius constructor propagation

The port already checks the law evaluation in `SetUp`, but
`TemporaryRadiusChange::new` discarded that error and the scope wrapper could
enter filleting without a valid temporary radius. The constructor now propagates
all setup failures. Fixed in tangency-labs/smlib 9a12ad7; the port carries it.
Direct regressions cover failed construction/body exclusion and unchanged offsets
after a failed law evaluation; native uninitialised-value behavior is not invoked.

## Review update (2026-10-26)

This follow-up is limited to the generic constructor and its scope wrapper.
`Kernel::fixes.temporary_radius_setup_errors` is default-off, supplied through
additive `new_with_fixes` and `with_temporary_radius_change_with_fixes` methods.
Existing call shapes and ordinary-status handling stay faithful to pinned C++.
The runner regression exercises both policies; the offset-law regression retains
the unconditional UB-252 protection already on main. `DirectTemporaryRadiusChange`
is outside this PR's scope and retains main's documented status behavior.
Earlier validation above belongs to the previous implementation and bases.
