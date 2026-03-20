# Goal

Add `std/sandbox` — capability-based sandboxing for agent spawns, shell commands, and scoped execution. Deny-by-default policies restrict what code can do at the lx runtime level (RuntimeCtx backend restriction) and optionally at the OS level (Landlock + seccomp). Full spec: `spec/stdlib-sandbox.md`.

# Why

- Every serious agentic tool sandboxes execution: Codex CLI (Landlock+seccomp), Cursor (container), Devin (VM). lx has `agent.eval_sandbox` for one narrow case but nothing for process-level restriction or scoped capability attenuation.
- LLM-generated workflows with `agent.spawn` hand child processes full system access. Tool-generating agents calling `$` can run arbitrary shell commands. No guardrails.
- The existing RuntimeCtx backend architecture already supports swapping backends — sandbox leverages this directly by wrapping existing backends with deny/restrict variants.

# What Changes

**New file `crates/lx/src/backends/restricted.rs` — Deny and Restricted backend wrappers:**

`DenyShellBackend` — returns `Err("shell access denied by sandbox policy")` for both `exec` and `exec_capture`.
`DenyHttpBackend` — returns `Err("network access denied by sandbox policy")` for `request`.
`DenyAiBackend` — returns `Err("AI access denied by sandbox policy")` for `prompt`.
`DenyPaneBackend` — returns `Err("pane access denied by sandbox policy")` for all methods.
`DenyEmbedBackend` — returns `Err("embedding access denied by sandbox policy")` for `embed`.
`RestrictedShellBackend` — wraps inner `ShellBackend`, checks command against allowlist before delegating.

**New file `crates/lx/src/stdlib/sandbox.rs` — module entry, policy creation, introspection:**

Policy is a Record stored in a global DashMap. Preset policies (`:pure`, `:readonly`, `:local`, `:network`, `:full`). Custom policies from config Records. `sandbox.policy`, `sandbox.describe`, `sandbox.permits`, `sandbox.merge`, `sandbox.attenuate`.

**New file `crates/lx/src/stdlib/sandbox_scope.rs` — scope enforcement:**

`sandbox.scope` pushes a policy onto a thread-local stack, creates a child RuntimeCtx with restricted backends swapped in, evaluates the body, pops the policy on exit. Nested scopes intersect (inner can only narrow, never widen).

**New file `crates/lx/src/stdlib/sandbox_exec.rs` — sandboxed shell and spawn:**

`sandbox.exec` runs a shell command under policy restrictions (Layer 1 only — lx-level restriction). `sandbox.spawn` wraps `agent.spawn` with restricted backends. OS-level enforcement (Landlock+seccomp) deferred to a follow-up — Layer 1 (RuntimeCtx restriction) covers the critical path.

# Files Affected

- `crates/lx/src/backends/restricted.rs` — New file: Deny* and Restricted* backends
- `crates/lx/src/backends/mod.rs` — Add `mod restricted; pub use restricted::*;`
- `crates/lx/src/stdlib/sandbox.rs` — New file: module entry, policy, introspection
- `crates/lx/src/stdlib/sandbox_scope.rs` — New file: scope enforcement
- `crates/lx/src/stdlib/sandbox_exec.rs` — New file: sandboxed exec/spawn
- `crates/lx/src/stdlib/mod.rs` — Register module
- `tests/102_sandbox.lx` — New test file

# Task List

### Task 1: Create restricted backend wrappers

**Subject:** Create Deny and Restricted backend wrappers in backends/restricted.rs

**Description:** Create `crates/lx/src/backends/restricted.rs`.

Imports: `std::sync::Arc`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`, `super::*`.

Implement 5 deny backends — each returns a descriptive `Value::Err`:

`pub struct DenyShellBackend;` — impl `ShellBackend`, both `exec` and `exec_capture` return `Ok(Value::Err(Box::new(Value::Str(Arc::from("shell access denied by sandbox policy")))))`.

`pub struct DenyHttpBackend;` — impl `HttpBackend`, `request` returns network denied error.

`pub struct DenyAiBackend;` — impl `AiBackend`, `prompt` returns AI denied error.

`pub struct DenyPaneBackend;` — impl `PaneBackend`, all 4 methods return pane denied error (open returns Err, update/close return LxError, list returns Err).

`pub struct DenyEmbedBackend;` — impl `EmbedBackend`, `embed` returns embedding denied error.

Implement 1 restricted backend:

`pub struct RestrictedShellBackend { pub inner: Arc<dyn ShellBackend>, pub allowed_cmds: Vec<String> }`. Impl `ShellBackend`: for `exec` and `exec_capture`, extract the first word of the command string (split on whitespace, take first). If it's in `allowed_cmds`, delegate to `self.inner.exec(cmd, span)`. Otherwise return `Ok(Value::Err(Box::new(Value::Str(Arc::from(format!("command '{}' not allowed by sandbox policy", first_word))))))`.

Add `mod restricted;` and `pub use restricted::*;` to `crates/lx/src/backends/mod.rs`.

**ActiveForm:** Creating restricted backend wrappers

---

### Task 2: Create sandbox.rs with policy creation and introspection

**Subject:** Create sandbox.rs with policy data structures, presets, and introspection functions

**Description:** Create `crates/lx/src/stdlib/sandbox.rs`.

Imports: `std::sync::{Arc, LazyLock, atomic::{AtomicU64, Ordering}}`, `dashmap::DashMap`, `indexmap::IndexMap`, `num_bigint::BigInt`, `crate::backends::RuntimeCtx`, `crate::builtins::mk`, `crate::error::LxError`, `crate::record`, `crate::span::Span`, `crate::value::Value`.

Define `pub(super) struct Policy`:
- `fs_read: Vec<String>` — allowed read paths
- `fs_write: Vec<String>` — allowed write paths
- `net_allow: Vec<String>` — allowed network destinations
- `shell: ShellPolicy` — enum `Deny | Allow | AllowList(Vec<String>)`
- `agent: bool`
- `mcp: bool`
- `ai: bool`
- `embed: bool`
- `pane: bool`
- `max_time_ms: u64` — 0 = unlimited

Static: `pub(super) static POLICIES: LazyLock<DashMap<u64, Policy>> = ...;`, `static NEXT_ID: ...;`.

`pub(super) fn policy_id(v: &Value, span: Span) -> Result<u64, LxError>`: extract `__policy_id` from Record.

`fn make_preset(name: &str) -> Policy`: match on name:
- `"pure"` → all deny, no fs, no net, no shell, no agent/mcp/ai/embed/pane
- `"readonly"` → fs_read: `["."]`, rest deny
- `"local"` → fs_read/write: `["."]`, shell: Allow, rest deny
- `"network"` → fs_read/write: `["."]`, net_allow: `["*"]`, ai: true, rest deny
- `"full"` → everything allowed

`fn parse_policy(config: &Value, span: Span) -> Result<Policy, LxError>`: extract fields from a Record config. `fs.read` and `fs.write` as lists of strings. `net.allow` as list. `shell` as bool or Record with `allow` list. `agent`, `mcp`, `ai`, `embed`, `pane` as bools. `max_time_ms` as Int.

`pub fn build() -> IndexMap<String, Value>`: register:
- `"policy"` → `bi_policy` arity 1
- `"describe"` → `bi_describe` arity 1
- `"permits"` → `bi_permits` arity 3
- `"merge"` → `bi_merge` arity 1
- `"attenuate"` → `bi_attenuate` arity 2
- `"scope"` → `super::sandbox_scope::bi_scope` arity 2
- `"exec"` → `super::sandbox_exec::bi_exec` arity 2
- `"spawn"` → `super::sandbox_exec::bi_spawn` arity 2

`bi_policy`: args[0] is either a Symbol (`:pure`, etc.) or a config Record. For Symbol, call `make_preset`. For Record, call `parse_policy`. Store in POLICIES, return handle Record.

`bi_describe`: args[0] is policy handle. Look up policy. Return a descriptive Record with `fs_read`, `fs_write`, `net`, `shell`, `agent`, `mcp`, `ai` fields.

`bi_permits`: args[0] is policy handle, args[1] is capability Symbol (`:fs_read`, `:fs_write`, `:shell`, `:net`, `:ai`, `:agent`, `:mcp`), args[2] is target Str. Check the policy for the given capability against the target. Return `Value::Bool`.

`bi_merge`: args[0] is List of policy handles. Intersection: for each field, take the most restrictive value.

`bi_attenuate`: args[0] is parent policy handle, args[1] is overrides Record. Parse overrides, intersect with parent. Error if overrides try to widen (grant capability parent doesn't have).

**ActiveForm:** Creating sandbox.rs with policy and introspection

---

### Task 3: Create sandbox_scope.rs with scope enforcement

**Subject:** Create sandbox_scope.rs with scoped RuntimeCtx restriction

**Description:** Create `crates/lx/src/stdlib/sandbox_scope.rs`.

Imports: `std::sync::Arc`, `std::cell::RefCell`, `crate::backends::*`, `crate::builtins::call_value_sync`, `crate::error::LxError`, `crate::span::Span`, `crate::value::Value`, `super::sandbox::{POLICIES, Policy, policy_id}`.

Thread-local policy stack: `thread_local! { static POLICY_STACK: RefCell<Vec<u64>> = RefCell::new(Vec::new()); }`.

`pub(super) fn current_policy_id() -> Option<u64>`: peek the stack.

`fn build_restricted_ctx(base: &Arc<RuntimeCtx>, policy: &Policy) -> Arc<RuntimeCtx>`: Create a new RuntimeCtx copying all fields from base, then swap backends based on policy:
- If `!policy.ai` → replace `ai` with `Arc::new(DenyAiBackend)`
- If `!policy.pane` → replace `pane` with `Arc::new(DenyPaneBackend)`
- If `!policy.embed` → replace `embed` with `Arc::new(DenyEmbedBackend)`
- If `policy.net_allow.is_empty()` → replace `http` with `Arc::new(DenyHttpBackend)`
- Match `policy.shell`:
  - `ShellPolicy::Deny` → replace `shell` with `Arc::new(DenyShellBackend)`
  - `ShellPolicy::AllowList(cmds)` → replace `shell` with `Arc::new(RestrictedShellBackend { inner: base.shell.clone(), allowed_cmds: cmds.clone() })`
  - `ShellPolicy::Allow` → keep base shell

Construct the new RuntimeCtx with all fields set. Wrap in `Arc::new`.

`pub fn bi_scope(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is policy handle, args[1] is body function. Get policy_id. Look up policy in POLICIES. Build restricted ctx. Push policy_id onto POLICY_STACK. Call `call_value_sync(&args[1], Value::Unit, span, &restricted_ctx)`. Pop from POLICY_STACK. Return the result (propagate errors).

**ActiveForm:** Creating sandbox_scope.rs with scope enforcement

---

### Task 4: Create sandbox_exec.rs with sandboxed exec and spawn

**Subject:** Create sandbox_exec.rs with sandboxed shell execution and agent spawn

**Description:** Create `crates/lx/src/stdlib/sandbox_exec.rs`.

Imports: `std::sync::Arc`, `crate::backends::*`, `crate::error::LxError`, `crate::span::Span`, `crate::value::Value`, `super::sandbox::{POLICIES, policy_id}`.

`pub fn bi_exec(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is policy handle, args[1] is command string. Get policy. Check `policy.shell`:
- `ShellPolicy::Deny` → return `Ok(Value::Err(...))`
- `ShellPolicy::AllowList(cmds)` → extract first word of command, check against list
- `ShellPolicy::Allow` → proceed

If allowed, delegate to `ctx.shell.exec(cmd, span)`.

`pub fn bi_spawn(args: &[Value], span: Span, ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError>`: args[0] is policy handle, args[1] is spawn config Record (same format as agent.spawn). Get policy. If `!policy.agent`, return `Ok(Value::Err(...))`. Otherwise, this delegates to the existing agent spawn mechanism but with a restricted RuntimeCtx. For now, return `Ok(Value::Err(Box::new(Value::Str(Arc::from("sandbox.spawn: OS-level sandboxing not yet implemented — use sandbox.scope for lx-level restriction")))))`. OS-level enforcement (Landlock+seccomp) is a follow-up.

**ActiveForm:** Creating sandbox_exec.rs with sandboxed execution

---

### Task 5: Register std/sandbox and write tests

**Subject:** Register sandbox module in mod.rs and write integration tests

**Description:** Edit `crates/lx/src/stdlib/mod.rs`:

Add `mod sandbox;`, `mod sandbox_scope;`, `mod sandbox_exec;`.

In `get_std_module`, add: `"sandbox" => sandbox::build(),`.

In `std_module_exists`, add `| "sandbox"`.

Create `tests/102_sandbox.lx`:

```
use std/sandbox

-- Preset policies
pure = sandbox.policy :pure
readonly = sandbox.policy :readonly
local = sandbox.policy :local
full = sandbox.policy :full

-- Describe
desc = sandbox.describe pure
assert (desc.ai == false) "pure denies ai"
assert (desc.shell == false) "pure denies shell"

desc_full = sandbox.describe full
assert (desc_full.ai == true) "full allows ai"

-- Permits
assert (sandbox.permits readonly :fs_read ".") "readonly permits fs_read"
assert (not (sandbox.permits readonly :fs_write ".")) "readonly denies fs_write"
assert (not (sandbox.permits pure :shell "ls")) "pure denies shell"

-- Scope: pure blocks shell
result = sandbox.scope pure () {
  $echo "should not run"
}
assert (type_of result == "Err") "pure scope blocks shell"

-- Scope: local allows shell
result2 = sandbox.scope local () {
  $^echo "hello"
}
assert (result2 | trim == "hello") "local scope allows shell"

-- Custom policy with shell allowlist
custom = sandbox.policy {
  shell: {allow: ["echo" "cat"]}
  ai: false
}
result3 = sandbox.scope custom () {
  $^echo "allowed"
}
assert (result3 | trim == "allowed") "allowlist permits echo"

-- Merge: intersection
merged = sandbox.merge [local full]
desc_merged = sandbox.describe merged
assert (desc_merged.ai == false) "merge intersects — local has no ai"

-- Attenuate: can narrow
narrow = sandbox.attenuate full {ai: false} ^
desc_narrow = sandbox.describe narrow
assert (desc_narrow.ai == false) "attenuate narrows"

log.info "102_sandbox: all passed"
```

Run `just diagnose` to verify compilation. Run `just test` to verify tests pass.

**ActiveForm:** Registering sandbox module and writing tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/STD_SANDBOX.md" })
```

Then call `next_task` to begin.
