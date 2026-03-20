## Audit Report: `std/sandbox`

### Task 1: Restricted backend wrappers — **DONE**
`backends/restricted.rs` exists. All 5 Deny backends + `RestrictedShellBackend` present. Registered in `backends/mod.rs:4,10`.

### Task 2: sandbox.rs with policy & introspection — **DONE**
`stdlib/sandbox.rs` has `build()` function. Policy struct and `ShellPolicy` enum live in `sandbox_policy.rs` (split from sandbox.rs — acceptable). `POLICIES` DashMap, `policy_id`, presets, `parse_policy`, `intersect_shell`, `describe_shell`, permits logic all present.

### Task 3: sandbox_scope.rs with scope enforcement — **DONE**
`POLICY_STACK` thread-local, `build_restricted_ctx`, `bi_scope` all present. Pushes/pops policy ID around `call_value_sync` with restricted ctx.

### Task 4: sandbox_exec.rs with exec/spawn — **DONE**
`bi_exec` checks shell policy before delegating. `bi_spawn` returns not-yet-implemented error (as spec'd — OS-level deferred).

### Task 5: Registration & tests — **DONE**
`stdlib/mod.rs`: `mod sandbox`, `mod sandbox_exec`, `mod sandbox_policy`, `mod sandbox_scope` registered. `"sandbox" => sandbox::build()` in `get_std_module`. `| "sandbox"` in `std_module_exists`. `tests/102_sandbox.lx` exists and **passes**.

### Diagnostics
- `just diagnose`: **0 errors, 0 warnings**
- `just test`: **102_sandbox.lx PASS** (103_search.lx fails — unrelated)

### Deviation from spec
- Policy struct extracted to `sandbox_policy.rs` (spec said all in `sandbox.rs`) — minor organizational split, no functional impact.

**Verdict: Fully implemented.** All 5 tasks done, compiles clean, tests pass.