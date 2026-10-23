# blend.fillet_executive_p2: propagate temporary radius setup errors (#458, 2026-10-25)

PR: https://github.com/tangency-labs/solidcore/pull/644

### Issue #458 follow-up: do not discard temporary radius setup errors

The law evaluations in `VariableRadiusFs::setup_offset_values` and
`TemporaryRadiusChange::set_up` already propagate errors, and StepSolve's first
setup call is already fallible (UB-252). The remaining gap was the constructor:
`TemporaryRadiusChange::new` passed `set_up` through `ignored`, suppressing every
ordinary failure and letting `with_temporary_radius_change` execute its body.
It now returns the setup failure, so the body never runs with an unestablished
radius. This completes the error propagation at this entry point, carrying the
safety intent of tangency-labs/smlib 9a12ad7.

A Rust-only regression in the existing executive runner checks the constructor
and wrapper against a failing adapter and verifies the body and law are untouched.
A kernel regression evaluates a failed law with real offset surfaces and checks
both distances remain unchanged. No differential tolerance or fixture is relaxed.
The second StepSolve setup remains behind its documented unported variable-radius
continuation; no new end-to-end variable fillet capability is claimed.

Validation for #458 on macOS/Apple Clang, Rust 1.98.0: 2,063 kernel tests, the
new executive runner regression (with `cpp`), fmt, plain/native workspace Clippy
with warnings denied, and wasm32 pass. Optimized differential checks: executive
p2 275 cases / 259 exact passes / 16 deferred / zero failed; standard solver p1
3839 cases / 3828 exact passes / 11 rejected / zero deferred or failed.
The broad macOS suite is not green; control comparisons without #432 reproduce
its seven NURBS and one 1D-solver failures. No comparison tolerance was changed.

## Review update (2026-10-26)

This follow-up is limited to the generic constructor and its scope wrapper.
`Kernel::fixes.temporary_radius_setup_errors` is default-off, supplied through
additive `new_with_fixes` and `with_temporary_radius_change_with_fixes` methods.
Existing call shapes and ordinary-status handling stay faithful to pinned C++.
The runner regression exercises both policies; the offset-law regression retains
the unconditional UB-252 protection already on main. `DirectTemporaryRadiusChange`
is outside this PR's scope and retains main's documented status behavior.
Earlier validation above belongs to the previous implementation and bases.
