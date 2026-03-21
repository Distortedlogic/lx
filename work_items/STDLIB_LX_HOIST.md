# Goal

Hoist pure-logic and Store-replaceable Rust stdlib modules into lx packages (`pkg/`), making the language more self-hosting. Two small Rust primitives (`try` builtin, `resolve_handler` builtin) are added first to close the one feature gap (propagated error catching from `^` in user callbacks), then 16 modules totaling ~3,200 lines of Rust are replaced with ~1,900 lines of idiomatic lx. `agent_errors.rs` stays as an internal Rust helper (still called by 6 non-hoisted modules) but its lx-visible exports move to `pkg/core/agent_errors.lx`. 19 concrete tasks.

# Why

- 20 Rust stdlib files implement pure logic (string matching, record manipulation, list operations, arithmetic, control flow) that has zero dependency on Rust-specific APIs (no DashMap, no threading, no async, no FFI). This logic is expressible in lx using existing builtins and should live in lx to reduce the Rust surface area and make the stdlib self-hosting. One additional file (`agent_errors.rs`) has its lx-visible exports moved to lx but stays as an internal Rust helper for non-hoisted callers
- `std/audit` (378 lines across 2 files) does string pattern matching (hedging detection, refusal detection, keyword overlap, repetition detection) — all expressible with lx's `lower`, `contains?`, `split`, `len`, `trim`
- `std/plan` (275 lines across 2 files) is a dependency-aware step executor — pure loop with record field access, set membership checks, and callback invocations
- `std/saga` (252 lines) is compensating transactions — a try/undo loop with retry delegation, expressible as lx control flow
- `std/retry` (214 lines) is backoff computation (constant/linear/exponential with jitter) plus a retry loop — pure arithmetic and `time.sleep`
- `std/budget` (275 lines across 2 files) uses DashMap for state that maps directly to a Store-backed Class — spend tracking, percentage math, and parent propagation
- 13 agent extension files (~2,000 lines total) implement reconciliation strategies, negotiation protocols, dispatch rules, mock agents, format adapters, handoff protocols, capability advertisement, and dialogue persistence — all pure logic or Store-replaceable state
- The Rust implementations of plan, saga, and retry catch `LxError::Propagate` (errors thrown by user callbacks via `^`), which lx code cannot currently catch — this is the one feature gap, solvable by adding a `try` builtin (~20 lines of Rust)
- `agent.intercept` dynamically resolves reloaded handlers via the `AGENT_HANDLERS` DashMap — lx code cannot do this lookup, solvable by adding a `resolve_handler` builtin (~15 lines of Rust)

# What changes

**Rust primitives (Tasks 1-2):** Add `try` builtin that wraps a function call and converts `LxError::Propagate` back to `Err` values. Add `resolve_handler` builtin that looks up the current handler for an agent in the `AGENT_HANDLERS` store, enabling lx-level intercept to work with hot-reloaded agents.

**Standalone module hoists (Tasks 3-7):** Replace `std/audit`, `std/retry`, `std/plan`, `std/saga`, and `std/budget` with lx packages in `pkg/core/`. Each task creates the lx package, updates imports in consumer files, removes the Rust module registration, and deletes the Rust source files.

**Agent error type hoist (Task 8):** Create lx type definitions in `pkg/core/agent_errors.lx` mirroring the 11 AgentErr variants. Remove the `tagged_ctors()` exports from `agent.rs` so lx code imports constructors from `pkg/core/agent_errors` instead of `std/agent`. Keep `agent_errors.rs` as an internal Rust helper — 6 non-hoisted Rust modules (agent_ipc.rs, agent_route.rs, agent.rs, introspect.rs, mcp.rs, mcp_typed.rs) still call its builder functions.

**Agent extension hoists (Tasks 9-16):** Replace 10 agent extension files with lx packages across `pkg/core/` and `pkg/agents/`. Each task creates the lx package, removes the extension from `agent.rs`'s build map, updates consumer imports from `use std/agent {X}` to `use pkg/.../X`, and deletes the Rust source files.

**Context file updates (Task 19):** Update INVENTORY.md, STDLIB.md, and REFERENCE.md to reflect the new package locations.

# Files affected

**New lx packages:**
- `pkg/core/audit.lx` — text quality checks (is_empty, is_hedging, is_refusal, etc.)
- `pkg/core/retry.lx` — backoff computation and retry loop
- `pkg/core/plan.lx` — dependency-aware step execution
- `pkg/core/saga.lx` — compensating transactions
- `pkg/core/budget.lx` — Budget Class with spend tracking
- `pkg/core/agent_errors.lx` — AgentErr union type (Timeout, RateLimited, etc.)
- `pkg/core/handoff.lx` — Handoff Trait + as_context formatter
- `pkg/core/adapter.lx` — Trait format adaptation + coerce
- `pkg/core/reconcile.lx` — 6 reconciliation strategies
- `pkg/core/negotiate_fmt.lx` — Trait format negotiation with structural matching
- `pkg/core/capability.lx` — Capabilities Trait + advertise
- `pkg/agents/negotiate.lx` — N-party iterative consensus
- `pkg/agents/dispatch_rules.lx` — pattern-based message dispatch
- `pkg/agents/mock.lx` — mock agents with Store-backed call recording
- `pkg/agents/dialogue_persist.lx` — dialogue save/load/list/delete via std/fs

**New Rust builtins:**
- `crates/lx/src/builtins/mod.rs` or `crates/lx/src/builtins/register.rs` — `try` and `resolve_handler` builtins

**Deleted Rust files (20 files):**
- `crates/lx/src/stdlib/audit.rs`, `audit_score.rs`
- `crates/lx/src/stdlib/retry.rs`
- `crates/lx/src/stdlib/plan.rs`, `step_deps.rs`
- `crates/lx/src/stdlib/saga.rs`
- `crates/lx/src/stdlib/budget.rs`, `budget_report.rs`
- `crates/lx/src/stdlib/agent_handoff.rs`
- `crates/lx/src/stdlib/agent_adapter.rs`
- `crates/lx/src/stdlib/agent_reconcile.rs`, `agent_reconcile_strat.rs`, `agent_reconcile_score.rs`
- `crates/lx/src/stdlib/agent_negotiate.rs`
- `crates/lx/src/stdlib/agent_dispatch.rs`
- `crates/lx/src/stdlib/agent_negotiate_fmt.rs`
- `crates/lx/src/stdlib/agent_mock.rs`
- `crates/lx/src/stdlib/agent_capability.rs`
- `crates/lx/src/stdlib/agent_dialogue_persist.rs`

**Kept as internal Rust helper (not deleted):**
- `crates/lx/src/stdlib/agent_errors.rs` — still called by 6 non-hoisted Rust modules (agent_ipc.rs, agent_route.rs, agent.rs, introspect.rs, mcp.rs, mcp_typed.rs); only the lx-visible exports via `tagged_ctors()` are removed

**Modified Rust files:**
- `crates/lx/src/stdlib/mod.rs` — remove module declarations and registrations for hoisted modules
- `crates/lx/src/stdlib/agent.rs` — remove build() entries for hoisted agent extensions, remove `tagged_ctors()` loop

**Modified lx files:** all consumer .lx files that import hoisted modules (tests, flows, brain, pkg)

**Updated context files:** `agent/INVENTORY.md`, `agent/STDLIB.md`, `agent/REFERENCE.md`

# Task List

## Task 1: Add `try` builtin

**Subject:** Add a `try` builtin that catches propagated errors from function calls
**ActiveForm:** Adding try builtin

Add a `try` builtin function to the builtins registration. `try` takes a function and an argument, calls the function, and converts any `LxError::Propagate` into a `Value::Err`. This enables lx-level orchestrators (plan, saga, retry) to catch errors that user callbacks throw via `^`.

Implementation: Register `try` as a sync builtin with arity 2 in `crates/lx/src/builtins/register.rs`. The implementation calls `call_value_sync(f, arg, span, ctx)`. On `Ok(v)` return `Ok(v)`. On `Err(LxError::Propagate { value, .. })` return `Ok(Value::Err(value))`. On any other `Err(e)` return `Err(e)`.

Verify: `just diagnose` passes. Write a test that defines a function using `^` on an error and confirms `try` catches it: `f = () { Err "boom" ^ }` then `result = try f ()` then `assert (err? result) "try catches propagated error"`.

## Task 2: Add `resolve_handler` builtin

**Subject:** Add a builtin that resolves the current handler for a hot-reloadable agent
**ActiveForm:** Adding resolve_handler builtin

Add a `resolve_handler` builtin that looks up an agent's current handler in the `AGENT_HANDLERS` DashMap (from `agent_reload.rs`). This enables lx-level intercept middleware to work correctly with agents that have been hot-reloaded via `agent.reload`.

Implementation: Register `resolve_handler` as a sync builtin with arity 1 in `crates/lx/src/builtins/register.rs`. The implementation receives an agent Record. Call `agent_reload::handler_id_from_agent(&agent)` — if it returns Some(id), call `agent_reload::lookup_handler(id)` — if that returns Some(handler), return the handler. Otherwise fall back to extracting the `handler` field from the agent Record directly. Return `Value::None` if no handler is found.

Verify: `just diagnose` passes.

## Task 3: Hoist std/audit to pkg/core/audit.lx

**Subject:** Replace Rust std/audit with pure lx implementation
**ActiveForm:** Hoisting std/audit to lx

Create `pkg/core/audit.lx` implementing all 11 exported functions from `audit.rs` and `audit_score.rs` in pure lx:

- `+is_empty` — `(s) { s | trim | len == 0 }`
- `+is_too_short` — `(s min_len) { len s < min_len }`
- `+is_repetitive` — split by `.` and `\n`, trim, lowercase, count duplicates, return `dupes * 2 >= total`
- `+is_hedging` — check lowercase input `contains?` any hedging phrase from a module-level list
- `+is_refusal` — same pattern with refusal phrases
- `+references_task` — keyword overlap: split task into words > 3 chars, check what fraction appear in output, pass if `hits * 3 >= total`
- `+files_exist` — `(paths) { paths | all? (p) fs.exists p }` (import `std/fs`)
- `+has_diff` — check for `"diff --git"`, `"@@"`, or both `"+++"` and `"---"` in string
- `+rubric` — identity function on a list of category records
- `+evaluate` — iterate rubric categories, call each category's `check` function, compute weighted score, compare to threshold, return result record
- `+quick_check` — check output against configurable criteria (empty, min_length, no_hedging, no_refusal, references_task), collect failure reasons

Remove `"audit" => audit::build()` from `get_std_module` in `crates/lx/src/stdlib/mod.rs`. Remove `| "audit"` from `std_module_exists`. Remove `mod audit;` declaration. Delete `crates/lx/src/stdlib/audit.rs` and `crates/lx/src/stdlib/audit_score.rs`. Update all `use std/audit` imports in .lx files to `use pkg/core/audit`.

Verify: `just diagnose` passes. `just test` passes.

## Task 4: Hoist std/retry to pkg/core/retry.lx

**Subject:** Replace Rust std/retry with pure lx implementation
**ActiveForm:** Hoisting std/retry to lx

Create `pkg/core/retry.lx` implementing `+retry` and `+retry_with` in pure lx:

- `+compute_delay` — `(opts attempt)` returns delay in ms. Match `opts.backoff`: `"constant"` → `opts.base_ms`, `"linear"` → `opts.base_ms * attempt`, `"exponential"` → `opts.base_ms * (2 ^ attempt)`. Cap at `opts.max_delay_ms`. Apply jitter if `opts.jitter` is true: random value between `delay/2` and `delay * 1.5`.
- `+retry` — `(f)` calls `retry_with {max: 3 backoff: "exponential" base_ms: 100 max_delay_ms: 30000 jitter: true} f`
- `+retry_with` — `(opts f)` loops up to `opts.max` times. Each iteration: call `try f ()`. If result is `Ok`, return it. If result is `Err`, check `opts.retry_on` predicate (if provided). If not retriable or last attempt, return `Err Exhausted {attempts last_error elapsed_ms}`. Otherwise `time.sleep (compute_delay opts attempt)` and continue.

Import `std/time` for sleep and elapsed tracking.

Remove `"retry" => retry::build()` from `get_std_module`. Remove `| "retry"` from `std_module_exists`. Remove `mod retry;`. Note: `saga.rs` uses `retry::RetryOpts` and `retry::compute_delay` internally — saga is being hoisted in Task 6 and will import `pkg/core/retry` instead, so also remove `pub(crate)` visibility from `RetryOpts` and `compute_delay` (they will no longer compile without callers). Delete `crates/lx/src/stdlib/retry.rs`. Update all `use std/retry` imports to `use pkg/core/retry`.

Verify: `just diagnose` passes. `just test` passes.

## Task 5: Hoist std/plan to pkg/core/plan.lx

**Subject:** Replace Rust std/plan with pure lx implementation
**ActiveForm:** Hoisting std/plan to lx

Create `pkg/core/plan.lx` implementing all 4 functions plus 2 constants in pure lx:

- Helper `+step_id` — `(step) { step.id ?? "unknown" }`
- Helper `+step_deps` — `(step) { step.depends ?? [] }`
- Helper `+next_ready` — `(remaining completed_ids)` find first step whose deps are all in completed_ids
- `+continue` — `{__action: "continue"}`
- `+skip` — `{__action: "skip"}`
- `+replan` — `(steps) { {__action: "replan" steps: steps} }`
- `+abort` — `(reason) { {__action: "abort" reason: reason} }`
- `+insert_after` — `(after_id steps) { {__action: "insert_after" after: after_id steps: steps} }`
- `+run` — `(steps executor on_step)` main loop: track completed results and IDs. Find next ready step via `next_ready`. Call `try executor (step context)` (using `try` from Task 1). Call `on_step step result plan_state`. Handle action records (continue, skip with successor pruning, abort, replan, insert_after). Return `Ok results` or `Err` on cycle/abort.

The `find_successors` helper: given a step ID and remaining steps, transitively find all steps that depend on it.

Remove `"plan" => plan::build()` from `get_std_module`. Remove `| "plan"` from `std_module_exists`. Remove `mod plan;` and `mod step_deps;`. Delete `crates/lx/src/stdlib/plan.rs` and `crates/lx/src/stdlib/step_deps.rs`. Update all `use std/plan` imports to `use pkg/core/plan`.

Verify: `just diagnose` passes. `just test` passes.

## Task 6: Hoist std/saga to pkg/core/saga.lx

**Subject:** Replace Rust std/saga with pure lx implementation
**ActiveForm:** Hoisting std/saga to lx

Create `pkg/core/saga.lx` implementing all 4 functions in pure lx. Import `pkg/core/retry {compute_delay}`, `std/time`.

- `+run` — `(steps)` calls `run_saga steps [] {max_retries: 0}`
- `+run_with` — `(steps opts)` calls `run_saga steps [] opts`
- `+define` — `(steps) { {__saga: true steps: steps} }`
- `+execute` — `(definition initial)` extracts steps from definition, runs with initial results
- Helper `run_saga` — main loop: track remaining steps, completed triples (id, result, undo_fn), completed_ids. Check timeout via `time.now`. Find next ready step. Extract `do` and `undo` functions. Call `try_step do_fn prev opts.max_retries`. On success, record completion. On failure, call `compensate` (reverse iterate completed, call each undo function, call `opts.on_compensate` if present), return structured Err.
- Helper `try_step` — retry loop: call `try do_fn prev`. On success return. On Err, if not last attempt, sleep with exponential backoff via `compute_delay`. On last attempt, return the error.
- Helper `compensate` — iterate completed in reverse, call each undo_fn with its result, collect compensation errors.

Remove `"saga" => saga::build()` from `get_std_module`. Remove `| "saga"` from `std_module_exists`. Remove `mod saga;`. Delete `crates/lx/src/stdlib/saga.rs`. Update all `use std/saga` imports to `use pkg/core/saga`.

Verify: `just diagnose` passes. `just test` passes.

## Task 7: Hoist std/budget to pkg/core/budget.lx

**Subject:** Replace Rust std/budget with Store-backed lx Class
**ActiveForm:** Hoisting std/budget to lx

Create `pkg/core/budget.lx` implementing budget tracking as a Class with Store-backed state. Import `std/time`, `pkg/core/collection {Collection}`.

- `Class Budget : [Collection] = { initial: Store () entries: Store () steps: 0 start_ms: 0 tight_at: 50.0 critical_at: 80.0 parent: None }`
- `+create` — `(opts)` construct a Budget instance. Extract `tight_at`, `critical_at` from opts (defaults 50.0, 80.0). Store numeric fields from opts as initial limits. Initialize used amounts to 0. Record start time via `time.now`.
- `+spend` — `(budget amounts)` increment used values in entries Store. Check each dimension against initial limits. If parent exists, propagate spend upward. Return `Ok ()` if within limits, `Err BudgetExhausted {used limit resource}` if exceeded (import error type from `pkg/core/agent_errors`).
- `+remaining` — `(budget)` compute initial minus used for each dimension. For `wall_time`, subtract elapsed since start.
- `+used` — `(budget)` return current used amounts.
- `+used_pct` — `(budget)` return `(used / initial) * 100` for each dimension.
- `+project` — `(budget steps_remaining)` estimate cost to completion based on per-step average.
- `+status` — `(budget)` return pressure level ("ok"/"tight"/"critical"/"exhausted") per dimension.
- `+slice` — `(budget limits)` create a child Budget with parent linkage for spend propagation.

In `crates/lx/src/stdlib/budget.rs`, the `budget_exhausted` call references `agent_errors.rs`. Inline the tagged value construction directly in budget.rs: replace `super::agent_errors::budget_exhausted(used, limit, resource)` with the literal `Value::Tagged { tag: Arc::from("BudgetExhausted"), values: Arc::new(vec![record! { "used" => Value::Float(used), "limit" => Value::Float(limit), "resource" => Value::Str(Arc::from(resource)) }]) }`. This removes the dependency before deletion.

Remove `"budget" => budget::build()` from `get_std_module`. Remove `| "budget"` from `std_module_exists`. Remove `mod budget;`. Delete `crates/lx/src/stdlib/budget.rs` and `crates/lx/src/stdlib/budget_report.rs`. Update all `use std/budget` imports to `use pkg/core/budget`.

Verify: `just diagnose` passes. `just test` passes.

## Task 8: Hoist agent.errors to pkg/core/agent_errors.lx

**Subject:** Replace Rust agent error constructors with lx type definitions
**ActiveForm:** Hoisting agent errors to lx

Create `pkg/core/agent_errors.lx` defining the AgentErr union type. Field structures must match the constructors used in `tests/73_agent_errors.lx`:

```
type AgentErr
  = Timeout {elapsed_ms: Int deadline_ms: Int}
  | RateLimited {retry_after_ms: Int limit: Str}
  | BudgetExhausted {used: Float limit: Float resource: Str}
  | ContextOverflow {size: Int capacity: Int content: Str}
  | Incompetent {agent: Str task: Str score: Float threshold: Float}
  | Upstream {service: Str code: Int message: Str}
  | PermissionDenied {action: Str resource: Str}
  | TraitViolation {expected: Str got: Str message: Str}
  | Unavailable {agent: Str reason: Str}
  | Cancelled {reason: Str}
  | Internal {message: Str}
```

Export the type and all constructors via `+`.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `tagged_ctors()` insertion loop (line ~181) that exposes constructors to lx. Do NOT remove `mod agent_errors;` from `mod.rs` and do NOT delete `agent_errors.rs` — 6 non-hoisted Rust modules still call its builder functions internally: `agent_ipc.rs` (`unavailable`), `agent_route.rs` (`unavailable` x2), `agent.rs` (`unavailable`), `introspect.rs` (`unavailable`), `mcp.rs` (`upstream`), `mcp_typed.rs` (`upstream`), and `budget.rs` (`budget_exhausted` — hoisted in Task 7, but inline the call there before deleting budget.rs).

Update `tests/73_agent_errors.lx` import from `use std/agent {Timeout RateLimited BudgetExhausted ContextOverflow Incompetent Upstream PermissionDenied TraitViolation Unavailable Cancelled Internal}` to `use pkg/core/agent_errors {Timeout RateLimited BudgetExhausted ContextOverflow Incompetent Upstream PermissionDenied TraitViolation Unavailable Cancelled Internal}`. Update any other .lx files that selectively import these error constructors from `std/agent`.

Verify: `just diagnose` passes. `just test` passes.

## Task 9: Hoist agent.handoff to pkg/core/handoff.lx

**Subject:** Replace Rust agent.handoff with lx Trait definition and formatter
**ActiveForm:** Hoisting agent.handoff to lx

Create `pkg/core/handoff.lx`:

- Define `Trait Handoff = { result: Any; tried: []; assumptions: []; uncertainties: []; recommendations: []; files_read: []; tools_used: []; duration_ms: 0 }`
- `+as_context` — `(handoff)` format a Handoff record as markdown: `"## Previous Agent Handoff\n"` followed by `"**Result:** {handoff.result}\n"` and list sections for tried, assumptions, uncertainties, recommendations, files_read, tools_used (each as `"- {item}\n"` bullets). Skip empty lists.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `"Handoff"` and `"as_context"` entries. Remove `mod agent_handoff;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_handoff.rs`. Update all `use std/agent {Handoff as_context}` imports to `use pkg/core/handoff {Handoff as_context}`.

Verify: `just diagnose` passes. `just test` passes.

## Task 10: Hoist agent.adapter to pkg/core/adapter.lx

**Subject:** Replace Rust agent.adapter and agent.coerce with lx implementation
**ActiveForm:** Hoisting agent.adapter to lx

Create `pkg/core/adapter.lx`:

- `+adapter` — `(source_trait target_trait mapping)` validate both args are Traits. Return a function `(msg)` that renames record fields per mapping and validates against target_trait. Mapping is a record `{source_field: "target_field"}`.
- `+coerce` — `(msg target_trait mapping)` one-shot transform: apply mapping to msg, validate against target_trait, return `Ok transformed` or `Err reason`.
- Helper `apply_mapping` — iterate msg record entries, rename fields per mapping, pass through unmapped fields. Validate required target Trait fields are present using `implements`.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `"adapter"` and `"coerce"` entries. Remove `mod agent_adapter;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_adapter.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 11: Hoist agent.reconcile to pkg/core/reconcile.lx

**Subject:** Replace Rust reconcile (3 files) with lx implementation
**ActiveForm:** Hoisting agent.reconcile to lx

Create `pkg/core/reconcile.lx` implementing all 6 strategies plus the main dispatch function:

- `+reconcile` — `(results config)` dispatch on `config.strategy`: `"union"`, `"intersection"`, `"vote"`, `"highest_confidence"`, `"max_score"`, `"merge_fields"`, or a function for custom strategy. Return `{merged sources conflicts dropped rounds dissenting}`.
- `do_union` — key each item via `config.key` function, deduplicate, resolve conflicts via `config.conflict` function.
- `do_intersection` — key each item, keep only items appearing in all result sets.
- `do_vote` — tally votes by `config.vote_field`, apply `config.weight` function per voter, check `config.quorum` (any/majority/unanimous/N), report dissenting indices.
- `do_highest_confidence` — select result with highest `.confidence` field.
- `do_max_score` — apply `config.score` function to each result, select highest. Support `config.early_stop` threshold.
- `do_merge_fields` — merge record fields across results, concatenate list fields, resolve scalar conflicts via `config.conflict`.
- Helpers: `make_result`, `make_conflict_entry`, `flatten_results`.

If the file exceeds 300 lines, split strategies into `pkg/core/reconcile_strat.lx` and import from `pkg/core/reconcile.lx`.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `"reconcile"` entry. Remove `mod agent_reconcile;`, `mod agent_reconcile_strat;`, `mod agent_reconcile_score;` from `mod.rs`. Delete all three Rust files. Update all `use std/agent {reconcile}` imports to `use pkg/core/reconcile {reconcile}`.

Verify: `just diagnose` passes. `just test` passes.

## Task 12: Hoist agent.negotiate to pkg/agents/negotiate.lx

**Subject:** Replace Rust agent.negotiate with lx implementation
**ActiveForm:** Hoisting agent.negotiate to lx

Create `pkg/agents/negotiate.lx`:

- `+negotiate` — `(agents config)` round-based consensus. Extract `config.proposal`, `config.max_rounds` (default 3), `config.converge` function, optional `config.on_round` callback. Loop up to max_rounds: build message `{round proposal positions}`, send to each agent via `agent ~>? msg ^`, collect responses as `{agent: agent.name position: response}`. Call `on_round round responses` if present. Call `converge responses` — if returns `Ok result`, return `Ok {result rounds positions unanimous}`. If returns `"continue"` or anything else, update positions and continue. After all rounds exhausted, return `Err {reason: "no_consensus" rounds positions}`.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `"negotiate"` entry. Remove `mod agent_negotiate;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_negotiate.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 13: Hoist agent.dispatch to pkg/agents/dispatch_rules.lx

**Subject:** Replace Rust agent.dispatch with lx implementation
**ActiveForm:** Hoisting agent.dispatch to lx

Create `pkg/agents/dispatch_rules.lx`:

- `+dispatch` — `(rules)` return an agent record with a handler that matches incoming messages against rules. Each rule is `{match: pattern to: target transform?: fn}`. Matching: if `match` is `"default"`, always matches. If `match` is a function, call it with msg and check truthiness. If `match` is a record, check all its fields are present and equal in msg. Apply optional `transform` to msg before sending to `to` target via `to ~>? transformed ^` or `to msg ^` if to is a function. Return `Err {type: "no_route" message: msg}` if no rule matches.
- `+dispatch_multi` — same but collects results from ALL matching rules instead of stopping at first.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `"dispatch"` and `"dispatch_multi"` entries. Remove `mod agent_dispatch;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_dispatch.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 14: Hoist agent.negotiate_format to pkg/core/negotiate_fmt.lx

**Subject:** Replace Rust agent.negotiate_format with lx implementation
**ActiveForm:** Hoisting agent.negotiate_format to lx

Create `pkg/core/negotiate_fmt.lx`:

- `+negotiate_format` — `(agent source_trait target_trait)` auto-discover compatible Trait field mappings. Compare source and target Trait fields: exact name matches, structural matches (same type, different name), subset matches. Use Levenshtein distance for fuzzy name matching. Return `Ok {mapping adapter}` where adapter is a reusable transform function, or `Err` if no viable mapping found.
- Helper `levenshtein` — string edit distance computation using a 2-row matrix approach.
- Helper `find_mappings` — iterate source fields, find best match in target fields by exact name, then by type + Levenshtein distance.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `"negotiate_format"` entry. Remove `mod agent_negotiate_fmt;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_negotiate_fmt.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 15: Hoist agent.mock to pkg/agents/mock.lx

**Subject:** Replace Rust agent.mock with Store-backed lx implementation
**ActiveForm:** Hoisting agent.mock to lx

Create `pkg/agents/mock.lx` using Store for call recording:

- `+mock` — `(rules)` create a Store for call history. Return an agent record with a handler that matches messages against rules (same matching logic as dispatch_rules: string equality, record subset, predicate function), records each call `{msg response}` in the Store, and returns the matched response. The `respond` field can be a value (returned directly) or a function (called with msg).
- `+mock_calls` — `(mock_agent)` return all recorded calls from the mock's Store as a list.
- `+mock_assert_called` — `(mock_agent pattern)` check if any recorded call's msg matches the pattern. Return `Ok ()` or `Err "expected call not found"`.
- `+mock_assert_not_called` — `(mock_agent pattern)` check that no recorded call's msg matches the pattern. Return `Ok ()` or `Err "unexpected call found"`.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove `"mock"`, `"mock_calls"`, `"mock_assert_called"`, `"mock_assert_not_called"` entries. Remove `mod agent_mock;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_mock.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 16: Hoist agent.capability to pkg/core/capability.lx

**Subject:** Replace Rust agent.capability with Store-backed lx implementation
**ActiveForm:** Hoisting agent.capability to lx

Create `pkg/core/capability.lx`:

- Define `Trait Capabilities = { traits: []; tools: []; domains: []; budget_remaining: -1; accepts: []; status: "ready" }`
- Module-level `advertised = Store ()` for capability registration.
- `+capabilities` — `(agent)` send `{type: "capabilities"}` to agent via `agent ~>? {type: "capabilities"} ^`, return the response.
- `+advertise` — `(name caps)` store capabilities in the module-level Store keyed by name.
- `+lookup` — `(name)` retrieve advertised capabilities by name from Store.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove `"Capabilities"`, `"capabilities"`, `"advertise"` entries. Remove `mod agent_capability;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_capability.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 17: Hoist agent.intercept to pkg/agents/intercept.lx

**Subject:** Replace Rust agent.intercept with lx implementation using resolve_handler
**ActiveForm:** Hoisting agent.intercept to lx

Create `pkg/agents/intercept.lx`:

- `+intercept` — `(agent middleware)` create a new agent record with the original agent's fields (minus `__pid` and `__handler_id`) and a new handler. The handler calls `middleware msg next` where `next` is a function that dispatches to the original agent. The `next` function uses `resolve_handler agent` (builtin from Task 2) to get the current handler (supporting hot-reloaded agents), falling back to `agent.handler` if no reload has occurred. For subprocess agents (has `__pid`), `next` sends via `agent ~>? msg ^`.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove the `"intercept"` entry. Remove `mod agent_intercept;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_intercept.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 18: Hoist agent.dialogue_persist to pkg/agents/dialogue_persist.lx

**Subject:** Replace Rust agent.dialogue_persist with lx implementation using std/fs + std/json
**ActiveForm:** Hoisting agent.dialogue_persist to lx

Create `pkg/agents/dialogue_persist.lx` using `std/fs` and `std/json`:

- `+dialogue_save` — `(session)` serialize session state (config + turn history) to JSON via `json.encode_pretty`. Write to `.lx/dialogues/{session.id}.json` via `fs.write`. Create directory if needed via `fs.mkdir`.
- `+dialogue_load` — `(id agent)` read JSON from `.lx/dialogues/{id}.json` via `fs.read`, parse via `json.parse`, bind the loaded session to the given agent.
- `+dialogue_list` — `()` list `.lx/dialogues/` via `fs.ls`, read each file, return metadata records `{id role turns created updated context_preview}`.
- `+dialogue_delete` — `(id)` remove `.lx/dialogues/{id}.json` via `fs.remove`.

In `crates/lx/src/stdlib/agent.rs` `build()`, remove `"dialogue_save"`, `"dialogue_load"`, `"dialogue_list"`, `"dialogue_delete"` entries. Remove `mod agent_dialogue_persist;` from `mod.rs`. Delete `crates/lx/src/stdlib/agent_dialogue_persist.rs`. Update imports.

Verify: `just diagnose` passes. `just test` passes.

## Task 19: Update agent context files

**Subject:** Update INVENTORY.md, STDLIB.md, and REFERENCE.md to reflect hoisted modules
**ActiveForm:** Updating agent context files

In `agent/INVENTORY.md`:
- Under "Stdlib", reduce the Rust module count by the number removed. Add entries for the new pkg/ packages in the appropriate cluster sections.
- Update the "Agent Extensions" section to remove entries for hoisted extensions and add references to their pkg/ locations.

In `agent/STDLIB.md`:
- Remove documentation sections for `std/audit`, `std/retry`, `std/plan`, `std/saga`, `std/budget`. Add corresponding sections for their pkg/ equivalents with import paths.
- Update "Built-in Functions" to include `try` and `resolve_handler`.
- Remove agent extension documentation for hoisted extensions. Add note pointing to pkg/ locations.

In `agent/REFERENCE.md`:
- Update the Codebase Layout tree to reflect removed .rs files and new .lx files.
- Update the "Adding Agent Extensions" section to note that pure-logic extensions should be implemented in pkg/ as lx packages rather than as Rust stdlib modules.
- Update file counts.

Verify: No code changes — context files only. `just diagnose` passes (unchanged).

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
mcp__workflow__load_work_item({ path: "work_items/STDLIB_LX_HOIST.md" })
```

Then call `next_task` to begin.
