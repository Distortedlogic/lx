# Goal

Remediate all violations identified in the Rust codebase quality audit across the `lx` and `lx-cli` crates. The audit found 15 categories of violations: 22 files exceeding the 300-line limit, inline `crate::` import paths at call sites, swallowed errors via `let _ =` and `eprintln`-only handling, Cargo dependencies not hoisted to workspace or using string shorthand, string literals used where enums should exist, duplicate/mergeable code across agent and git stdlib modules, duplicate store-ID/handle patterns, a missing prelude for the `lx` crate, single-callsite extracted functions, free functions that should be methods, repeated `Arc<IndexMap>` + `Value::Record` construction boilerplate, `&Arc<RuntimeCtx>` parameters where `&RuntimeCtx` suffices, and an intermediate `collect::<Vec<_>>().join()` that can use itertools.

# Why

- 22 `.rs` files violate the hard 300-line cap from CLAUDE.md, making navigation and maintenance difficult and preventing future splits from compounding
- Inline `crate::` paths at 39 call sites obscure what types are in scope and make refactoring harder
- 34 `let _ =` occurrences and multiple `eprintln`-only error paths silently discard failures in cleanup, flush, kill/wait, and compensation callbacks — masking bugs in production-like test runs
- 8 Cargo dependencies in crate-level `Cargo.toml` files specify versions directly instead of using `workspace = true`, and 10 dependencies across all `Cargo.toml` files use string shorthand instead of object notation — fragmenting version management
- 7 stdlib modules match on string literals for fixed variant sets (backoff strategy, supervision strategy, restart type, timeout behavior, quorum mode, markdown node types, prompt sections, log levels) — no exhaustiveness checking, no typo prevention, unnecessary allocation
- 5 agent modules (`agents_auditor`, `agents_reviewer`, `agents_grader`, `agents_planner`, `agents_router`) share near-identical `build()`, `extract_fields()`, `build_system_prompt()`, `build_user_prompt()`, `parse_llm_result()` scaffolding — every new agent copies the same boilerplate
- 5 git modules (`git_log`, `git_status`, `git_branch`, `git_ops`, `git_diff`) repeat identical error-handling, argument extraction, and record-building patterns 30+ times
- `trace.rs`/`trace_query.rs`/`trace_progress.rs` and `profile.rs`/`profile_io.rs`/`profile_strategy.rs` duplicate `store_id()`/`profile_id()` extraction and `LazyLock<DashMap>` state storage
- The `lx` crate exports 13+ public modules with no prelude, forcing `lx-cli` to import each type individually
- 12+ private functions across agent modules are called from exactly one site — unnecessary indirection
- Free functions in `circuit.rs` (`breaker_id`, `trip_check`) and across `agents_*` modules operate on struct parameters instead of being methods
- 100+ call sites manually build `IndexMap::new()` + `insert()` + `Value::Record(Arc::new(...))` with no convenience constructor
- Every builtin/stdlib function signature takes `&Arc<RuntimeCtx>` even when the function never clones the Arc — passing `&RuntimeCtx` would be correct for the majority
- `agents_auditor.rs` collects into a `Vec` only to immediately call `.join()` — an iterator-based join avoids the allocation

# What changes

## Cargo dependency hygiene (Finding 4, 5)

Hoist `miette`, `num-bigint`, `num-integer`, `num-traits`, `thiserror`, `indexmap` from `crates/lx/Cargo.toml` and `clap`, `miette` from `crates/lx-cli/Cargo.toml` into `[workspace.dependencies]` in the root `Cargo.toml`. Convert all string shorthand dependencies (`dep = "version"`) to object notation (`dep = { version = "version" }`) across all three `Cargo.toml` files. Reference each dependency with `dep.workspace = true` in crate-level files.

## Inline import path cleanup (Finding 2)

Add `use` statements at the top of each affected file and replace all 39 `crate::` inline paths with short names. Affected files: `interpreter/modules.rs`, `interpreter/agents.rs`, `interpreter/eval.rs`, `interpreter/mod.rs`, `interpreter/apply.rs`, `interpreter/patterns.rs`, `builtins/call.rs`, `builtins/hof.rs`, `builtins/hof_extra.rs`, `backends/user.rs`, `parser/pattern.rs`, `parser/paren.rs`, `checker/mod.rs`, `checker/synth.rs`, `stdlib/diag.rs`, `stdlib/agent_dialogue.rs`, `error.rs`, `value.rs`.

## Swallowed error remediation (Finding 3)

Replace all `let _ =` that discard `Result` values with explicit error propagation or logging that surfaces the error. For `child.kill()`/`child.wait()` in `agent.rs` and `agent_supervise.rs`, propagate the error via `?` or return it. For `saga.rs` on_compensate callback, propagate the error. For `lx-cli/src/main.rs` `stdout().flush()`, this is already handled correctly (returns early on error). For checker `let _ = self.synth(...)` and interpreter `let _ = span` / `let _ = name` / `let _ = exported`, determine if these are intentional unused bindings vs actual error swallowing and handle accordingly.

## String-to-enum conversion (Finding 6)

Define enums for each fixed variant set and parse the string into the enum at the boundary:
- `Backoff` enum already exists in `retry.rs` — no new type needed, just ensure the parse happens at the boundary
- `SupervisionStrategy` enum with variants `OneForOne`, `OneForAll`, `RestForOne` in `agent_supervise.rs`
- `RestartType` enum with variants `Permanent`, `Transient`, `Temporary` in `agent_supervise.rs`
- `TimeoutPolicy` enum with variants `Abort`, `Approve`, `Reject` in `agent_gate.rs`
- `ReconcileStrategy` enum already exists — verify it is used correctly
- `Quorum` enum already exists — verify it is used correctly
- `MdNodeType` enum with variants `Heading`, `Para`, `Code`, `List`, `Ordered`, `Table`, `Blockquote`, `Hr`, `Link`, `Raw` in `md_build.rs`
- `PromptSection` enum with variants `System`, `Constraints`, `Instructions`, `Examples` in `prompt.rs`
- `StatusLevel` enum with variants `Info`, `Warn`, `Error`, `Success` in `backends/user.rs`

Each enum gets a `from_lx_str` method that strips the leading `:` and parses. Match arms then use the enum variant directly.

## File splits for 300-line violations (Finding 1)

Split each of the 22 oversize files into logical sub-files. The splits are:
- `parser/statements.rs` (589) → `statements.rs` + `statements_protocol.rs` + `statements_use.rs`
- `ast.rs` (500) → `ast.rs` + `ast_types.rs` + `ast_protocol.rs`
- `interpreter/mod.rs` (475) → `mod.rs` + `keyword_hints.rs` + `exec.rs`
- `interpreter/eval.rs` (474) → `eval.rs` + `eval_ops.rs` + `eval_str.rs`
- `interpreter/agents.rs` (470) → `agents.rs` + `agents_protocol.rs`
- `parser/prefix.rs` (467) → `prefix.rs` + `prefix_collections.rs`
- `parser/mod.rs` (441) → `mod.rs` + `helpers.rs`
- `interpreter/apply.rs` (425) → `apply.rs` + `apply_record.rs`
- `stdlib/memory.rs` (417) → `memory.rs` + `memory_ops.rs`
- `stdlib/tasks.rs` (379) → `tasks.rs` + `tasks_ops.rs`
- `stdlib/context.rs` (375) → `context.rs` + `context_ops.rs`
- `value.rs` (372) → `value.rs` + `value_types.rs`
- `builtins/coll.rs` (406) → `coll.rs` + `coll_ops.rs`
- `builtins/mod.rs` (348) → `mod.rs` + `builtins_log.rs` + `builtins_io.rs`
- `stdlib/diag_walk.rs` (355) → `diag_walk.rs` + `diag_walk_visit.rs`
- `lexer/mod.rs` (353) → `mod.rs` + `operators.rs`
- `stdlib/audit.rs` (350) → `audit.rs` + `audit_checks.rs`
- `stdlib/budget.rs` (345) → `budget.rs` + `budget_ops.rs`
- `stdlib/prompt.rs` (332) → `prompt.rs` + `prompt_ops.rs`
- `stdlib/agents_grader.rs` (325) → `agents_grader.rs` + `agents_grader_prompts.rs`
- `stdlib/md.rs` (314) → `md.rs` + `md_parse.rs`
- `lx-cli/src/main.rs` (306) → `main.rs` + `run.rs`

## Free functions to methods (Finding 12)

Move `breaker_id` and `trip_check` in `circuit.rs` into `impl Breaker`. Move `extract_fields` in each `agents_*` module into an `impl` on the corresponding fields struct (`AuditFields`, `ReviewFields`, etc.). Move `collect_exports` in `interpreter/modules.rs` to an appropriate impl block.

## Intermediate collect elimination (Finding 15)

In `agents_auditor.rs`, replace `.collect::<Vec<_>>().join("; ")` with `itertools::Itertools::join` on the iterator, or a manual `fold`-based string builder. Add `itertools` to workspace dependencies if not already present, or use fold.

## `&Arc<RuntimeCtx>` audit (Finding 14)

This is a large-scale signature change affecting 450+ call sites. It should be a separate work item due to scope. Flag it here but do not include tasks for it.

## Duplicate code extraction — agent modules (Finding 7)

This is a large structural refactor affecting 5 modules with shared scaffolding. It should be a separate work item. Flag it here but do not include tasks for it.

## Duplicate code extraction — git modules (Finding 8)

This is a large structural refactor affecting 5 modules with shared error handling. It should be a separate work item. Flag it here but do not include tasks for it.

## Duplicate store-ID pattern (Finding 9)

This is a moderate refactor affecting 6 modules. It should be a separate work item. Flag it here but do not include tasks for it.

## Prelude creation (Finding 10)

This depends on understanding the full import graph across crates. It should be a separate work item. Flag it here but do not include tasks for it.

## Value::Record builder (Finding 13)

Creating a builder/macro for `Value::Record` construction affects 100+ sites. It should be a separate work item. Flag it here but do not include tasks for it.

## Single-callsite function inlining (Finding 11)

Inlining functions at their sole call sites should happen as part of the file-split refactoring and the agent-module deduplication work items. Flag it here but defer to those work items.

# Files affected

**Cargo.toml files:**
- `Cargo.toml` — add workspace dependencies, convert string shorthand to object notation
- `crates/lx/Cargo.toml` — replace direct versions with `workspace = true`, convert string shorthand
- `crates/lx-cli/Cargo.toml` — replace direct versions with `workspace = true`

**Inline import path fixes (18 files):**
- `crates/lx/src/interpreter/modules.rs` — add `use` for `stdlib`, `lexer::lex`, `parser::parse`
- `crates/lx/src/interpreter/agents.rs` — add `use` for `stdlib::agent`, `stdlib::mcp`, `value::BuiltinFunc`
- `crates/lx/src/interpreter/eval.rs` — add `use` for `builtins::call_value`
- `crates/lx/src/interpreter/mod.rs` — add `use` for `builtins::register`
- `crates/lx/src/interpreter/apply.rs` — add `use` for `value::ValueKey`
- `crates/lx/src/interpreter/patterns.rs` — add `use` for `ast::StrPart`
- `crates/lx/src/builtins/call.rs` — add `use` for `interpreter::Interpreter`
- `crates/lx/src/builtins/hof.rs` — add `use` for `builtins::call_value`
- `crates/lx/src/builtins/hof_extra.rs` — add `use` for `value::ValueKey`
- `crates/lx/src/backends/user.rs` — add `use` for `stdlib::json_conv::json_to_lx`
- `crates/lx/src/parser/pattern.rs` — add `use` for `ast::ListElem`
- `crates/lx/src/parser/paren.rs` — add `use` for `ast::Literal`
- `crates/lx/src/checker/mod.rs` — add `use` for `ast::ProtocolEntry`
- `crates/lx/src/checker/synth.rs` — add `use` for `ast::UnaryOp`, `ast::ListElem`, `span::Span`
- `crates/lx/src/stdlib/diag.rs` — add `use` for `lexer::lex`, `parser::parse`
- `crates/lx/src/stdlib/agent_dialogue.rs` — add `use` for `builtins::call_value`
- `crates/lx/src/error.rs` — add `use` for `value::Value`
- `crates/lx/src/value.rs` — add `use` for `backends::RuntimeCtx`

**Swallowed error fixes (8 files):**
- `crates/lx/src/stdlib/agent.rs` — propagate kill/wait errors
- `crates/lx/src/stdlib/agent_supervise.rs` — propagate kill/wait errors
- `crates/lx/src/stdlib/saga.rs` — propagate on_compensate error
- `crates/lx/src/interpreter/eval.rs` — remove `let _ = span`
- `crates/lx/src/interpreter/mod.rs` — address `let _ = name`, `let _ = exported`
- `crates/lx/src/checker/synth.rs` — address `let _ = self.synth(...)` returns
- `crates/lx/src/checker/mod.rs` — address `let _ = named_to_type(...)` returns
- `crates/lx/src/stdlib/env.rs` — address `let _ = &args[0]` patterns
- `crates/lx/src/stdlib/cron.rs` — address `let _ = &args[0]` patterns
- `crates/lx/src/stdlib/time.rs` — address `let _ = &args[0]` patterns
- `crates/lx/src/stdlib/git.rs` — address `let _ = &args[0]` patterns
- `crates/lx/src/stdlib/git_status.rs` — address `let _ = &args[0]` patterns
- `crates/lx/src/stdlib/git_branch.rs` — address `let _ = &args[0]` patterns
- `crates/lx/src/stdlib/md.rs` — address `let _ = stack.pop()` pattern

**String-to-enum conversion (7 files):**
- `crates/lx/src/stdlib/agent_supervise.rs` — add `SupervisionStrategy` and `RestartType` enums
- `crates/lx/src/stdlib/agent_gate.rs` — add `TimeoutPolicy` enum
- `crates/lx/src/stdlib/md_build.rs` — add `MdNodeType` enum
- `crates/lx/src/stdlib/prompt.rs` — add `PromptSection` enum
- `crates/lx/src/backends/user.rs` — add `StatusLevel` enum

**Free function to method moves (2 files):**
- `crates/lx/src/stdlib/circuit.rs` — move `breaker_id` and `trip_check` into `impl Breaker`
- `crates/lx/src/stdlib/agents_auditor.rs` — move `extract_fields` into `impl AuditFields`

**Intermediate collect fix (1 file):**
- `crates/lx/src/stdlib/agents_auditor.rs` — replace collect-then-join with fold

**File splits (22 files → ~47 files):**
- All 22 oversize files listed in finding 1 — each split into 2-3 sub-files

# Task List

## Task 1: Hoist Cargo dependencies to workspace and convert string shorthand

**Files:** `Cargo.toml`, `crates/lx/Cargo.toml`, `crates/lx-cli/Cargo.toml`

In root `Cargo.toml` under `[workspace.dependencies]`, add these entries in object notation:
- `miette = { version = "7", features = ["fancy"] }`
- `num-bigint = { version = "0.4" }`
- `num-integer = { version = "0.1" }`
- `num-traits = { version = "0.2" }`
- `thiserror = { version = "2" }`
- `indexmap = { version = "2" }`
- `clap = { version = "4", features = ["derive"] }`

Convert existing string-shorthand entries in root `Cargo.toml` to object notation:
- `pulldown-cmark = "0.12"` → `pulldown-cmark = { version = "0.12" }`
- `chrono = "0.4"` → `chrono = { version = "0.4" }`
- `dashmap = "6"` → `dashmap = { version = "6" }`
- `fastrand = "2"` → `fastrand = { version = "2" }`
- `parking_lot = "0.12"` → `parking_lot = { version = "0.12" }`

In `crates/lx/Cargo.toml`, replace:
- `miette = { version = "7", features = ["fancy"] }` → `miette.workspace = true`
- `num-bigint = "0.4"` → `num-bigint.workspace = true`
- `num-integer = "0.1"` → `num-integer.workspace = true`
- `num-traits = "0.2"` → `num-traits.workspace = true`
- `thiserror = "2"` → `thiserror.workspace = true`
- `indexmap = "2"` → `indexmap.workspace = true`

In `crates/lx-cli/Cargo.toml`, replace:
- `clap = { version = "4", features = ["derive"] }` → `clap.workspace = true`
- `miette = { version = "7", features = ["fancy"] }` → `miette.workspace = true`

**Verify:** `just diagnose` passes with no errors.

Run: `just fmt` then `git add Cargo.toml crates/lx/Cargo.toml crates/lx-cli/Cargo.toml` then `git commit -m "hoist all cargo deps to workspace, convert string shorthand to object notation"`

## Task 2: Fix inline import paths in interpreter crate modules

**Files:** `crates/lx/src/interpreter/modules.rs`, `crates/lx/src/interpreter/agents.rs`, `crates/lx/src/interpreter/eval.rs`, `crates/lx/src/interpreter/mod.rs`, `crates/lx/src/interpreter/apply.rs`, `crates/lx/src/interpreter/patterns.rs`

In `interpreter/modules.rs`: add `use crate::stdlib::{std_module_exists, get_std_module};`, `use crate::lexer::lex;`, `use crate::parser::parse;` at the top. Replace all 4 inline `crate::stdlib::std_module_exists`, `crate::stdlib::get_std_module`, `crate::lexer::lex`, `crate::parser::parse` call sites with short names.

In `interpreter/agents.rs`: add `use crate::stdlib::agent::{send_subprocess, ask_subprocess};`, `use crate::stdlib::mcp::{register_tool_defs, typed_call};`, `use crate::value::BuiltinFunc;` at the top. Replace all 5 inline `crate::stdlib::agent::send_subprocess`, `crate::stdlib::agent::ask_subprocess`, `crate::stdlib::mcp::register_tool_defs`, `crate::stdlib::mcp::typed_call`, `crate::value::BuiltinFunc` call sites with short names.

In `interpreter/eval.rs`: add `use crate::builtins::call_value;` at the top. Replace the inline `crate::builtins::call_value` call site with the short name.

In `interpreter/mod.rs`: add `use crate::builtins;` at the top (if not present). Replace `crate::builtins::register(...)` with `builtins::register(...)`.

In `interpreter/apply.rs`: add `use crate::value::ValueKey;` at the top. Replace `crate::value::ValueKey(...)` with `ValueKey(...)`.

In `interpreter/patterns.rs`: add `use crate::ast::StrPart;` at the top. Replace `crate::ast::StrPart::Text` and `crate::ast::StrPart::Interp` with `StrPart::Text` and `StrPart::Interp`.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/interpreter/` then `git commit -m "fix inline crate:: import paths in interpreter modules"`

## Task 3: Fix inline import paths in builtins, backends, parser, checker, stdlib, error, value

**Files:** `crates/lx/src/builtins/call.rs`, `crates/lx/src/builtins/hof.rs`, `crates/lx/src/builtins/hof_extra.rs`, `crates/lx/src/backends/user.rs`, `crates/lx/src/parser/pattern.rs`, `crates/lx/src/parser/paren.rs`, `crates/lx/src/checker/mod.rs`, `crates/lx/src/checker/synth.rs`, `crates/lx/src/stdlib/diag.rs`, `crates/lx/src/stdlib/agent_dialogue.rs`, `crates/lx/src/error.rs`, `crates/lx/src/value.rs`

In `builtins/call.rs`: add `use crate::interpreter::Interpreter;` at the top. Replace `crate::interpreter::Interpreter::with_env(...)` with `Interpreter::with_env(...)`.

In `builtins/hof.rs`: the `crate::builtins::call_value` reference is to the parent module — add `use super::call::call_value;` or `use crate::builtins::call_value;` as a `use` at the top if not present, then use the short name. (Check which form is already in use.)

In `builtins/hof_extra.rs`: add `use crate::value::ValueKey;` at the top. Replace `crate::value::ValueKey(...)` with `ValueKey(...)`. Also fix the `indexmap::IndexMap` inline path if present — add `use indexmap::IndexMap;` if needed.

In `backends/user.rs`: add `use crate::stdlib::json_conv::json_to_lx;` at the top. Replace `crate::stdlib::json_conv::json_to_lx(...)` with `json_to_lx(...)`.

In `parser/pattern.rs`: add `use crate::ast::ListElem;` at the top. Replace `crate::ast::ListElem::Single` and `crate::ast::ListElem::Spread` with `ListElem::Single` and `ListElem::Spread`.

In `parser/paren.rs`: add `use crate::ast::Literal;` at the top (check if already imported from the existing `use crate::ast::{...}` line — if so, add `Literal` to that import). Replace `crate::ast::Literal::Unit` with `Literal::Unit`. Also add `SPattern` to imports if `crate::ast::SPattern` is used inline.

In `checker/mod.rs`: add `use crate::ast::ProtocolEntry;` at the top. Replace `crate::ast::ProtocolEntry::Field(f)` with `ProtocolEntry::Field(f)`.

In `checker/synth.rs`: add `use crate::ast::{UnaryOp, ListElem};` and `use crate::span::Span;` at the top (or add to existing imports). Replace all `crate::ast::UnaryOp::Neg`, `crate::ast::UnaryOp::Not`, `crate::ast::ListElem::Single`, `crate::ast::ListElem::Spread`, `crate::span::Span` with short names.

In `stdlib/diag.rs`: add `use crate::lexer::lex;` and `use crate::parser::parse;` at the top. Replace inline call sites.

In `stdlib/agent_dialogue.rs`: add `use crate::builtins::call_value;` at the top. Replace inline call site.

In `error.rs`: add `use crate::value::Value;` at the top (if not present). Replace `crate::value::Value` references with `Value`.

In `value.rs`: add `use crate::backends::RuntimeCtx;` at the top (if not present). Replace `crate::backends::RuntimeCtx` references with `RuntimeCtx`.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/builtins/ crates/lx/src/backends/ crates/lx/src/parser/ crates/lx/src/checker/ crates/lx/src/stdlib/diag.rs crates/lx/src/stdlib/agent_dialogue.rs crates/lx/src/error.rs crates/lx/src/value.rs` then `git commit -m "fix inline crate:: import paths in builtins, backends, parser, checker, stdlib, error, value"`

## Task 4: Remediate swallowed errors — agent kill/wait and saga compensation

**Files:** `crates/lx/src/stdlib/agent.rs`, `crates/lx/src/stdlib/agent_supervise.rs`, `crates/lx/src/stdlib/saga.rs`

In `agent.rs` (lines 261-266): replace the `eprintln`-only handling of `child.kill()` and `child.wait()` failures. The kill function should propagate these errors via `?` or return them as `LxError::runtime`. Change the `if let Err(e)` blocks to use `map_err` and `?` propagation.

In `agent_supervise.rs` (lines 250-255): same pattern — replace `eprintln`-only handling of `child.kill()` and `child.wait()` failures with error propagation. If the function returns `Result`, use `?`. If it returns nothing, change the return type to `Result<(), LxError>` and propagate.

In `saga.rs` (lines 121-127): the `on_compensate` callback error is silently logged. Propagate the error — if `on_compensate` fails, that is a saga failure that should be surfaced. Return the error via `?` or accumulate it into the compensation result.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/agent.rs crates/lx/src/stdlib/agent_supervise.rs crates/lx/src/stdlib/saga.rs` then `git commit -m "propagate errors from agent kill/wait and saga compensation callbacks"`

## Task 5: Remediate swallowed errors — let _ = patterns in interpreter and checker

**Files:** `crates/lx/src/interpreter/eval.rs`, `crates/lx/src/interpreter/mod.rs`, `crates/lx/src/checker/synth.rs`, `crates/lx/src/checker/mod.rs`

Read each file and determine the context of every `let _ =` occurrence:

In `interpreter/eval.rs` line 60: `let _ = span;` — this is suppressing an unused variable warning. Remove the line entirely and prefix the parameter with `_` if it is unused, or use the span value.

In `interpreter/mod.rs` lines 229, 404, 451: `let _ = name;` and `let _ = exported;` — these suppress unused variable warnings. Remove these lines. If the variables are genuinely unused, prefix them with `_` at their binding site. If they should be used, use them.

In `checker/synth.rs` lines 35, 40, 83, 87: `let _ = self.synth(left);` — these discard the type-checking result. The `synth` method performs side effects (recording diagnostics). If the return value is genuinely unneeded, convert to a statement call without `let _ =` — just `self.synth(left);`. But verify the return type is not `Result` — if it is, propagate with `?`.

In `checker/mod.rs` lines 135, 144, 146: `let _ = named_to_type(...)` and `let _ = resolve_mcp_output(...)` — these discard type resolution results. Same approach: if these return `Result`, propagate with `?`. If they return a non-Result type and the value is unneeded, just call the function as a bare statement.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/interpreter/eval.rs crates/lx/src/interpreter/mod.rs crates/lx/src/checker/synth.rs crates/lx/src/checker/mod.rs` then `git commit -m "remediate let _ = patterns in interpreter and checker modules"`

## Task 6: Remediate swallowed errors — let _ = &args[0] patterns in stdlib

**Files:** `crates/lx/src/stdlib/env.rs`, `crates/lx/src/stdlib/cron.rs`, `crates/lx/src/stdlib/time.rs`, `crates/lx/src/stdlib/git.rs`, `crates/lx/src/stdlib/git_status.rs`, `crates/lx/src/stdlib/git_branch.rs`, `crates/lx/src/stdlib/md.rs`

Read each file. The pattern `let _ = &args[0];` appears to be intentionally ignoring the first argument (perhaps a module self-reference or unused context). For each occurrence:

If `args[0]` is genuinely unused and the function signature requires it for API consistency, remove the `let _ = &args[0];` line entirely — the unused argument in `args` does not need explicit suppression since `args` is a slice.

In `md.rs` line 105: `let _ = stack.pop();` discards a `pop()` result. The return value of `pop()` is `Option<T>` — if it is intentionally unused (we just want the side effect of removing the top), this is acceptable. Remove the `let _ =` and just write `stack.pop();` as a bare statement.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/env.rs crates/lx/src/stdlib/cron.rs crates/lx/src/stdlib/time.rs crates/lx/src/stdlib/git.rs crates/lx/src/stdlib/git_status.rs crates/lx/src/stdlib/git_branch.rs crates/lx/src/stdlib/md.rs` then `git commit -m "remove let _ = &args[0] and let _ = stack.pop() patterns in stdlib"`

## Task 7: Define enums for string literal variant sets — agent_supervise and agent_gate

**Files:** `crates/lx/src/stdlib/agent_supervise.rs`, `crates/lx/src/stdlib/agent_gate.rs`

In `agent_supervise.rs`:

Define `SupervisionStrategy` enum with variants `OneForOne`, `OneForAll`, `RestForOne`. Add a `fn from_lx_str(s: &str) -> Result<Self, LxError>` method that strips a leading `:` and matches. Default is `OneForOne`. Replace the string-based strategy parsing at line 76-80 with `SupervisionStrategy::from_lx_str(...)`. Replace the `match strategy.as_str()` block at line 195-199 with a match on the enum.

Define `RestartType` enum with variants `Permanent`, `Transient`, `Temporary`. Add a `fn from_lx_str(s: &str) -> Result<Self, LxError>` method. Default is `Permanent`. Replace the string-based restart parsing at line 140-144 with `RestartType::from_lx_str(...)`. Replace any downstream string matching on restart type with enum matching.

In `agent_gate.rs`:

Define `TimeoutPolicy` enum with variants `Abort`, `Approve`, `Reject`. Add a `fn from_lx_str(s: &str) -> Result<Self, LxError>` method. Default is `Abort`. Replace the string-based on_timeout parsing at line 158-162 with `TimeoutPolicy::from_lx_str(...)`. Replace the `match on_timeout { "approve" => ..., "reject" => ..., _ => ... }` block at lines 122-142 with a match on the enum.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/agent_supervise.rs crates/lx/src/stdlib/agent_gate.rs` then `git commit -m "replace string literal matching with enums in agent_supervise and agent_gate"`

## Task 8: Define enums for string literal variant sets — md_build, prompt, user

**Files:** `crates/lx/src/stdlib/md_build.rs`, `crates/lx/src/stdlib/prompt.rs`, `crates/lx/src/backends/user.rs`

In `md_build.rs`:

Define `MdNodeType` enum with variants: `Heading`, `Para`, `Code`, `List`, `Ordered`, `Table`, `Blockquote`, `Hr`, `Link`, `Raw`. Add a `fn from_str(s: &str) -> Option<Self>` method. Replace the `match t.as_str() { "heading" => ..., "para" => ..., ... }` block at lines 169-230 with: parse the string into `MdNodeType` first, then match on the enum. Handle the `None` case (unknown type) by doing nothing (matching the current `_ => {}` arm).

In `prompt.rs`:

Define `PromptSection` enum with variants: `System`, `Constraints`, `Instructions`, `Examples`, `Custom(String)`. Add a `fn from_str(s: &str) -> Self` method. Replace the `match name.as_str() { "system" => ..., "constraints" => ..., ... }` block at lines 323-330 with a match on the enum.

In `backends/user.rs`:

Define `StatusLevel` enum with variants: `Info`, `Warn`, `Error`, `Success`. Add a `fn from_lx_str(s: &str) -> Self` method that strips leading `:` and parses, defaulting to using the raw string as the tag for unknown values. Replace the `match level.trim_start_matches(':') { "info" => ..., ... }` block at lines 112-117 with a match on the enum. The enum's Display impl or a `tag()` method returns the uppercase tag string.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/md_build.rs crates/lx/src/stdlib/prompt.rs crates/lx/src/backends/user.rs` then `git commit -m "replace string literal matching with enums in md_build, prompt, and user backend"`

## Task 9: Move free functions to methods — circuit.rs and agents_auditor.rs

**Files:** `crates/lx/src/stdlib/circuit.rs`, `crates/lx/src/stdlib/agents_auditor.rs`

In `circuit.rs`:

Move `breaker_id` (lines 44-53) into an `impl` block. Since it takes `&Value` and extracts a breaker ID, it is best as a standalone helper on `Breaker` that takes a `&Value` — add it as `fn from_value(v: &Value, span: Span) -> Result<u64, LxError>` on an appropriate associated function, or keep as a module-level function if Breaker does not own Value. Actually, since it extracts the ID from a Value, make it `Breaker::id_from_value(v: &Value, span: Span) -> Result<u64, LxError>`. Update all call sites from `breaker_id(v, span)` to `Breaker::id_from_value(v, span)`.

Move `trip_check` (lines 95-125) into `impl Breaker` as `fn trip_check(&mut self)`. Update all call sites from `trip_check(&mut b)` to `b.trip_check()`.

In `agents_auditor.rs`:

Move `extract_fields` (lines 30-64) into `impl AuditFields` as `fn from_args(args: &[Value], span: Span) -> Result<Self, LxError>`. Update all call sites from `extract_fields(args, span)` to `AuditFields::from_args(args, span)`.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/circuit.rs crates/lx/src/stdlib/agents_auditor.rs` then `git commit -m "move free functions to methods on Breaker and AuditFields"`

## Task 10: Eliminate intermediate collect in agents_auditor.rs

**File:** `crates/lx/src/stdlib/agents_auditor.rs`

At lines 103-107, replace:
```
let feedback = failures.iter().map(|(_, reason)| reason.as_str()).collect::<Vec<_>>().join("; ");
```

With a fold-based approach that avoids the intermediate Vec allocation:
```
let feedback = failures.iter().map(|(_, reason)| reason.as_str()).fold(String::new(), |mut acc, s| { if !acc.is_empty() { acc.push_str("; "); } acc.push_str(s); acc });
```

Or use `itertools::Itertools::join` if itertools is already a dependency. Check `Cargo.toml` for itertools — if not present, use the fold approach.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/agents_auditor.rs` then `git commit -m "eliminate intermediate collect-then-join in agents_auditor"`

## Task 11: Split oversize files — parser/statements.rs (589 lines)

**File:** `crates/lx/src/parser/statements.rs`

Split into three files:
- `statements.rs` — keep `try_parse_binding`, `is_typed_binding`, `skip_type_tokens`, `skip_default_expr` and any other core binding/statement parsing methods. Target ≤ 200 lines.
- `statements_protocol.rs` — extract all `protocol`-related parsing: `parse_protocol_stmt`, `parse_protocol_fields`, `parse_protocol_union`, `parse_mcp_tool_decl`, and any helper methods related to protocol/mcp parsing. Add `mod statements_protocol;` to `parser/mod.rs` (or the appropriate parent).
- `statements_use.rs` — extract `parse_use_stmt` and any use-statement helpers. Add `mod statements_use;` to the parent.

All extracted methods remain `impl super::Parser` methods (using `impl super::Parser` in the new files). Ensure each file is under 300 lines.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/parser/` then `git commit -m "split parser/statements.rs into three sub-modules"`

## Task 12: Split oversize files — ast.rs (500 lines)

**File:** `crates/lx/src/ast.rs`

Split into three files:
- `ast.rs` — keep core expression and statement types (`Expr`, `Stmt`, `SExpr`, `SStmt`, `Literal`, `BinOp`, `UnaryOp`, `Pattern`, `MatchArm`, `FieldKind`, `Binding`, `BindTarget`, `Param`, `Program`). Target ≤ 200 lines.
- `ast_types.rs` — extract type annotation types (`TypeExpr`, `SType`, `TypeField`) and any type-system-related AST nodes.
- `ast_protocol.rs` — extract protocol/MCP types (`ProtocolEntry`, `ProtocolField`, `ProtocolUnionDef`, `McpToolDecl`, `McpOutputType`, `AgentMethod`).

Add `mod ast_types;` and `mod ast_protocol;` in `lib.rs` or re-export from `ast.rs`. Ensure all types remain publicly accessible at their current paths (add `pub use` re-exports in `ast.rs` if needed for backward compatibility — but since we do not care about backward compatibility, just update import paths at all usage sites).

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/ast.rs crates/lx/src/ast_types.rs crates/lx/src/ast_protocol.rs crates/lx/src/lib.rs` then `git commit -m "split ast.rs into ast, ast_types, ast_protocol"`

## Task 13: Split oversize files — interpreter/mod.rs (475 lines)

**File:** `crates/lx/src/interpreter/mod.rs`

Split into three files:
- `mod.rs` — keep `Interpreter` struct definition, `new()`, `run()`, core `eval_stmts` dispatch, and the `ModuleExports` type. Target ≤ 200 lines.
- `keyword_hints.rs` — extract the `keyword_hint` function and all its match arms (this is ~50 lines but logically distinct).
- `exec.rs` — extract statement execution methods that are currently in `mod.rs` (e.g., `eval_stmt` match arms for `Let`, `Expr`, `AgentDef`, `TypeDef`, etc.) into a separate file as `impl Interpreter` methods.

Add `mod keyword_hints;` and `mod exec;` in `mod.rs`.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/interpreter/` then `git commit -m "split interpreter/mod.rs into mod, keyword_hints, exec"`

## Task 14: Split oversize files — interpreter/eval.rs (474 lines)

**File:** `crates/lx/src/interpreter/eval.rs`

Split into three files:
- `eval.rs` — keep `eval`, `eval_literal`, `eval_expr`, core dispatch, and `dedent_string`. Target ≤ 200 lines.
- `eval_ops.rs` — extract binary/unary operation evaluation (`eval_binop`, `eval_unary`, coalesce, etc.).
- `eval_str.rs` — extract string interpolation, template evaluation, and related helpers.

Add `mod eval_ops;` and `mod eval_str;` in the parent `mod.rs`.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/interpreter/` then `git commit -m "split interpreter/eval.rs into eval, eval_ops, eval_str"`

## Task 15: Split oversize files — interpreter/agents.rs (470 lines) and interpreter/apply.rs (425 lines)

**Files:** `crates/lx/src/interpreter/agents.rs`, `crates/lx/src/interpreter/apply.rs`

In `interpreter/agents.rs`: split into `agents.rs` (send/ask/spawn, agent handler resolution) and `agents_protocol.rs` (protocol validation, MCP tool handling, protocol field checking). Add `mod agents_protocol;` in parent `mod.rs`. Target each file ≤ 250 lines.

In `interpreter/apply.rs`: split into `apply.rs` (core `apply_func`, function application, currying) and `apply_record.rs` (record field access, method dispatch on records, named argument handling). Add `mod apply_record;` in parent `mod.rs`. Target each file ≤ 250 lines.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/interpreter/` then `git commit -m "split interpreter/agents.rs and apply.rs into sub-modules"`

## Task 16: Split oversize files — parser/prefix.rs (467 lines) and parser/mod.rs (441 lines)

**Files:** `crates/lx/src/parser/prefix.rs`, `crates/lx/src/parser/mod.rs`

In `parser/prefix.rs`: split into `prefix.rs` (core prefix dispatch, literals, identifiers, unary ops) and `prefix_collections.rs` (list, map, block/record, and shell parsing). Add `mod prefix_collections;` in parent `mod.rs`. Target each file ≤ 250 lines.

In `parser/mod.rs`: split into `mod.rs` (Parser struct, `parse()`, `parse_program`, core helpers like `advance`, `peek`, `expect_kind`, `skip_semis`) and `helpers.rs` (utility methods like `is_at_expr_start`, binding-power tables, precedence helpers). Add `mod helpers;` in `mod.rs`. Target each file ≤ 250 lines.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/parser/` then `git commit -m "split parser/prefix.rs and parser/mod.rs into sub-modules"`

## Task 17: Split oversize files — stdlib batch 1 (memory, tasks, context, diag_walk)

**Files:** `crates/lx/src/stdlib/memory.rs` (417), `crates/lx/src/stdlib/tasks.rs` (379), `crates/lx/src/stdlib/context.rs` (375), `crates/lx/src/stdlib/diag_walk.rs` (355)

For each file, split into a primary file and an `_ops` companion:

- `memory.rs` → `memory.rs` (build, store/retrieve core) + `memory_ops.rs` (search, list, delete, similarity operations)
- `tasks.rs` → `tasks.rs` (build, spawn, status core) + `tasks_ops.rs` (task group operations, join, cancel)
- `context.rs` → `context.rs` (build, push/pop core) + `context_ops.rs` (merge, query, formatting operations)
- `diag_walk.rs` → `diag_walk.rs` (walker struct, core walk) + `diag_walk_visit.rs` (visitor methods for each AST node type)

Add corresponding `mod` declarations in `stdlib/mod.rs`. Target each file ≤ 250 lines.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/` then `git commit -m "split stdlib memory, tasks, context, diag_walk into sub-modules"`

## Task 18: Split oversize files — stdlib batch 2 (audit, budget, prompt, agents_grader, md)

**Files:** `crates/lx/src/stdlib/audit.rs` (350), `crates/lx/src/stdlib/budget.rs` (345), `crates/lx/src/stdlib/prompt.rs` (332), `crates/lx/src/stdlib/agents_grader.rs` (325), `crates/lx/src/stdlib/md.rs` (314)

For each file, split into a primary file and a companion:

- `audit.rs` → `audit.rs` (build, core check dispatch) + `audit_checks.rs` (individual check implementations)
- `budget.rs` → `budget.rs` (build, allocate/spend core) + `budget_ops.rs` (query, report, rebalance operations)
- `prompt.rs` → `prompt.rs` (build, PromptState struct, core operations) + `prompt_ops.rs` (section manipulation, render, without)
- `agents_grader.rs` → `agents_grader.rs` (build, extract_fields, main grading logic) + `agents_grader_prompts.rs` (prompt construction and result parsing helpers)
- `md.rs` → `md.rs` (build, core markdown parsing) + `md_parse.rs` (pulldown-cmark event handling, block extraction)

Add corresponding `mod` declarations in `stdlib/mod.rs`. Target each file ≤ 250 lines.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/stdlib/` then `git commit -m "split stdlib audit, budget, prompt, agents_grader, md into sub-modules"`

## Task 19: Split oversize files — value.rs, builtins/coll.rs, builtins/mod.rs, lexer/mod.rs, lx-cli/main.rs

**Files:** `crates/lx/src/value.rs` (372), `crates/lx/src/builtins/coll.rs` (406), `crates/lx/src/builtins/mod.rs` (348), `crates/lx/src/lexer/mod.rs` (353), `crates/lx-cli/src/main.rs` (306)

- `value.rs` → `value.rs` (Value enum, core methods, Display delegation) + `value_types.rs` (LxFunc, BuiltinFunc, BuiltinFn, ValueKey, McpToolDef, McpOutputDef, ProtoFieldDef, helper types). Add `mod value_types;` in `lib.rs` and `pub use value_types::*;` in `value.rs`.

- `builtins/coll.rs` → `coll.rs` (first, last, contains, get, len, keys, values, sort, reverse, unique, flatten) + `coll_ops.rs` (merge, group_by, chunk, window, frequencies, set operations). Add `mod coll_ops;` in `builtins/mod.rs`.

- `builtins/mod.rs` → `mod.rs` (register function, core builtins like type_of, to_str, print, assert) + `builtins_log.rs` (all log-level builtins: log_info, log_warn, log_err, log_debug, make_log_builtin) + `builtins_io.rs` (input, prompt, sleep, and other I/O builtins). Add `mod builtins_log;` and `mod builtins_io;` in `mod.rs`.

- `lexer/mod.rs` → `mod.rs` (Lexer struct, lex function, advance/peek, skip_whitespace, main token dispatch) + `operators.rs` (operator token recognition, multi-character operator parsing). Add `mod operators;` in `mod.rs`.

- `lx-cli/src/main.rs` → `main.rs` (arg parsing, main function, CLI dispatch) + `run.rs` (the `run_file` / `run_repl` / execution logic). Add `mod run;` in `main.rs`.

Target each file ≤ 250 lines.

**Verify:** `just diagnose` passes.

Run: `just fmt` then `git add crates/lx/src/value.rs crates/lx/src/value_types.rs crates/lx/src/builtins/ crates/lx/src/lexer/ crates/lx/src/lib.rs crates/lx-cli/src/` then `git commit -m "split value.rs, builtins/coll.rs, builtins/mod.rs, lexer/mod.rs, lx-cli/main.rs"`

## Task 20: Final verification

Run the full verification suite:

1. `just fmt` — ensure all formatting is correct
2. `just diagnose` — ensure `cargo check` + `cargo clippy -- -D warnings` passes with no errors or warnings
3. `just test` — ensure all `.lx` suite tests pass

Verify no `.rs` file under `crates/` exceeds 300 lines: `find crates/ -name '*.rs' -exec awk 'END { if (NR > 300) print FILENAME, NR }' {} \;`

Verify no `crate::` inline import paths remain at call sites (excluding `use` statements and macro bodies): search for `crate::` in non-`use` positions.

Verify no `let _ =` patterns remain that discard `Result` values.

If any check fails, fix the issue and re-run all three verification commands.

Run: `just fmt` then `just diagnose` then `just test`

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To load this work item into the workflow pipeline, run:

```
mcp__workflow__load_work_item({ path: "work_items/RUST_CODEBASE_QUALITY_AUDIT.md" })
```

Then call `mcp__workflow__next_task` to begin execution. After completing each task's implementation, call `mcp__workflow__complete_task` which will automatically format, commit, and run diagnostics. On the final task, `complete_task` also runs the full test suite and cleans up the work item file.

**Deferred to separate work items (out of scope for this audit):**
- Finding 7: Agent module deduplication (agents_auditor, agents_reviewer, agents_grader, agents_planner, agents_router shared scaffolding)
- Finding 8: Git module deduplication (git_log, git_status, git_branch, git_ops, git_diff shared error handling)
- Finding 9: Store-ID/handle pattern deduplication (trace and profile modules)
- Finding 10: Prelude creation for the `lx` crate
- Finding 11: Single-callsite function inlining (deferred to file-split and deduplication work items)
- Finding 13: Value::Record builder macro/method (100+ call sites)
- Finding 14: `&Arc<RuntimeCtx>` → `&RuntimeCtx` parameter audit (450+ call sites)