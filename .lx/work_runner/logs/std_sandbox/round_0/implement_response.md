The lx crate now has 0 warnings. The remaining 3 warnings are in lx-mobile (unrelated).

All 5 tasks from the work item are already implemented:

1. **Task 1** - `crates/lx/src/backends/restricted.rs` - Deny and Restricted backend wrappers (DenyShellBackend, DenyHttpBackend, DenyAiBackend, DenyPaneBackend, DenyEmbedBackend, RestrictedShellBackend)
2. **Task 2** - `crates/lx/src/stdlib/sandbox.rs` + `sandbox_policy.rs` - Policy data structures, presets, introspection functions (policy, describe, permits, merge, attenuate)
3. **Task 3** - `crates/lx/src/stdlib/sandbox_scope.rs` - Scoped RuntimeCtx restriction with thread-local policy stack
4. **Task 4** - `crates/lx/src/stdlib/sandbox_exec.rs` - Sandboxed shell exec and agent spawn
5. **Task 5** - Module registered in `stdlib/mod.rs`, test in `tests/102_sandbox.lx` passes

The only change I made was fixing a pre-existing clippy warning (collapsible `if`) in `defaults.rs`. The sandbox tests pass, `just diagnose` is clean for the lx crate, and 105/106 tests pass (the one failure is in 103_search.lx, unrelated to sandbox).