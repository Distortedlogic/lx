## STD_SANDBOX Implementation Report

### File Existence
| File | Status |
|------|--------|
| `crates/lx/src/backends/restricted.rs` | EXISTS |
| `crates/lx/src/stdlib/sandbox.rs` | EXISTS |
| `crates/lx/src/stdlib/sandbox_scope.rs` | EXISTS |
| `crates/lx/src/stdlib/sandbox_exec.rs` | EXISTS |
| `crates/lx/src/stdlib/sandbox_policy.rs` | EXISTS (extra, not in spec) |
| `tests/102_sandbox.lx` | EXISTS |

### Registrations
- `backends/mod.rs`: `mod restricted` + `pub use restricted::*` — REGISTERED
- `stdlib/mod.rs`: `mod sandbox`, `mod sandbox_exec`, `mod sandbox_policy`, `mod sandbox_scope` — REGISTERED
- `get_std_module`: `"sandbox" => sandbox::build()` — REGISTERED
- `std_module_exists`: `| "sandbox"` — REGISTERED

### Task Status

| Task | Status | Evidence |
|------|--------|----------|
| T1: Restricted backends | **DONE** | All 5 Deny backends + RestrictedShellBackend implemented |
| T2: sandbox.rs policy/introspection | **DONE** | `build()`, `Policy`, `POLICIES`, `bi_policy/describe/permits/merge/attenuate` all present |
| T3: sandbox_scope.rs | **DONE** | `POLICY_STACK`, `build_restricted_ctx`, `bi_scope` implemented |
| T4: sandbox_exec.rs | **DONE** | `bi_exec`, `bi_spawn` implemented |
| T5: Registration + tests | **DONE** | Module registered; `102_sandbox.lx` passes |

### Diagnostics
- `just diagnose`: **0 errors, 0 warnings**
- `just test`: `102_sandbox.lx` — **PASS** (note: unrelated `103_search.lx` fails)

### Verdict
**All 5 tasks: DONE.** The work item is fully implemented. One extra file (`sandbox_policy.rs`) exists beyond the spec — likely a refactoring split.