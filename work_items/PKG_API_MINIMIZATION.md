# Goal

Achieve the minimal optimal API covering of the lx package ecosystem by eliminating cross-layer redundancies, removing duplicate functions, correcting layer violations, fixing bugs, and hoisting domain-specific packages out of the generic library layer. 17 concrete fixes across std/, pkg/, and flows/.

# Why

- `std/agents/monitor` and `pkg/agents/guard` both detect injection patterns and stuck loops — two implementations of the same feature at different layers
- `std/agents/planner`, `std/agents/router`, `std/agents/reviewer` are pure prompt compositions implemented in Rust that have identical lx equivalents in `pkg/ai/` — maintaining both doubles the maintenance surface
- `std/ctx` provides 7 functions (`empty`/`get`/`set`/`keys`/`merge`/`remove`/`save`/`load`) that `Store` already provides natively except `merge` — an entire std/ module exists for one missing Store method
- `pkg/agents/catalog.route` and `pkg/ai/router.quick_route` implement the same keyword-overlap scoring algorithm independently
- `pkg/agents/dialogue` exports 3 functions (`open_dialogue`/`dialogue_turn`/`close_dialogue`) that are 1-line pass-throughs over `std/agent` with zero added logic
- `pkg/ai/quality.refine_response` and `refine_code` are 40-line functions with 90% identical structure — only differ in grader function, prompt text, and threshold
- `pkg/kit/context_manager.pressure_level` reimplements the classification already done by `pkg/data/context.ContextWindow.pressure`
- `pkg/kit/context_manager.pressure` returns Float while `pkg/data/context.ContextWindow.pressure` returns Str — same name, different types, different packages
- `pkg/agents/guard.check` and `guard.check_safety` are identical logic differing only in which pattern list is used
- `pkg/ai/router.route` calls `json.parse` without importing `std/json` — runtime crash
- `pkg/agents/monitor` is 267 lines with 23 exports mixing health/budget monitoring with introspection/analysis — two distinct concerns in one package
- `pkg/data/tieredmem` mixes single-MemoryStore helpers (`create`/`seed`/`daily`/`weekly`) with multi-store composition (`init`/`remember`/`recall`) — two abstraction levels in one package
- `pkg/connectors/*` (6 files), `pkg/infra/github`, and `pkg/data/training` are domain-specific code in a generic library layer — only consumed by flows/
- `pkg/kit/tool_executor.execute_single` hardcodes dispatch to only "Bash" and "Read", making the "generic" executor not generic
- `pkg/agents/monitor.suggest_strategy` passes 2 args to `Inspector.similar_actions` which takes 1 — arity mismatch bug

# What changes

**std/ deprecations (Tasks 1-3):** Remove `std/agents/monitor` (subsumed by `pkg/agents/guard`). Mark `std/agents/planner`, `std/agents/router`, `std/agents/reviewer` as deprecated — new code uses `pkg/ai/planner`, `pkg/ai/router`, `pkg/ai/reviewer`. Add `merge` method to `std/store` and deprecate `std/ctx` (all other ctx functions are Store methods).

**pkg/ deduplication (Tasks 4-9):** Delete `pkg/agents/catalog.route` (callers use `pkg/ai/router.quick_route`). Delete `pkg/agents/dialogue.open_dialogue/dialogue_turn/close_dialogue` (callers use `std/agent` directly). Merge `pkg/ai/quality.refine_response` and `refine_code` into single `refine_work`. Delete `pkg/kit/context_manager.pressure_level` (use `win.pressure()` directly). Rename `pkg/kit/context_manager.pressure` to `pressure_pct`. Merge `pkg/agents/guard.check` and `check_safety` into one function with optional patterns parameter.

**pkg/ bug fixes (Tasks 10-11):** Add `use std/json` to `pkg/ai/router.lx`. Fix `pkg/agents/monitor.suggest_strategy` to pass correct arity to `Inspector.similar_actions`.

**pkg/ structural splits (Tasks 12-13):** Split `pkg/agents/monitor` — move introspection functions (`self_assess`/`detect_doom_loop`/`strategy_analysis`/`time_pressure`/`generate_status`/`should_pivot`/`narrate_thinking`/`suggest_pivot`) to `pkg/core/introspect` as module-level functions. Move `pkg/data/tieredmem` single-store helpers (`tiers`/`thresholds`/`create`/`seed`/`daily`/`weekly`) to `pkg/data/memory`.

**pkg/ → flows/ hoists (Tasks 14-16):** Move `pkg/connectors/*` (6 files) to `flows/connectors/`. Move `pkg/infra/github` to `flows/lib/github.lx`. Move `pkg/data/training` to `flows/lib/training.lx`. Update all consumer imports.

**pkg/ generalization (Task 17):** Make `pkg/kit/tool_executor.execute` accept an optional `dispatch_fn` parameter for non-builtin tools, defaulting to the current Err fallback.

# Files affected

- `crates/lx/src/stdlib/mod.rs` — Remove `agents_monitor` from `get_std_module` and `std_module_exists`
- `crates/lx/src/stdlib/store_dispatch.rs` — Add `merge` method to Store dot-dispatch
- `crates/lx/src/stdlib/store.rs` — Add `merge` to Store module build
- `tests/39_agents_monitor.lx` — Update to use `pkg/agents/guard`
- `flows/examples/defense_layers.lx` — Already migrated off `std/agents/monitor`
- `pkg/agents/catalog.lx` — Remove `route` function
- `pkg/agents/dialogue.lx` — Remove `open_dialogue`, `dialogue_turn`, `close_dialogue`
- `pkg/ai/quality.lx` — Replace `refine_response`/`refine_code` with `refine_work`
- `pkg/kit/context_manager.lx` — Remove `pressure_level`, rename `pressure` to `pressure_pct`
- `pkg/agents/guard.lx` — Merge `check`/`check_safety` into one with optional patterns param
- `pkg/ai/router.lx` — Add `use std/json`
- `pkg/agents/monitor.lx` — Move 8 introspection functions out, fix `similar_actions` call
- `pkg/core/introspect.lx` — Receive 8 introspection functions as module-level exports
- `pkg/data/tieredmem.lx` — Remove `tiers`/`thresholds`/`create`/`seed`/`daily`/`weekly`
- `pkg/data/memory.lx` — Receive single-store helper functions
- `pkg/connectors/*.lx` (6 files) — Move to `flows/connectors/`
- `pkg/infra/github.lx` — Move to `flows/lib/github.lx`
- `pkg/data/training.lx` — Move to `flows/lib/training.lx`
- `pkg/kit/tool_executor.lx` — Add `dispatch_fn` parameter to `execute`
- `brain/lib/context_mgr.lx` — Update `pressure` → `pressure_pct` call
- `brain/orchestrator.lx` — Update if uses removed dialogue functions
- All consumer files importing moved packages — Update import paths

# Task List

## Task 1: Remove std/agents/monitor

**Subject:** Delete std/agents/monitor from Rust stdlib
**ActiveForm:** Removing std/agents/monitor

Remove `agents_monitor` from `get_std_module` match and `std_module_exists` match in `crates/lx/src/stdlib/mod.rs`. Do not delete the Rust source file yet — just unregister it so `use std/agents/monitor` no longer resolves. Update `tests/39_agents_monitor.lx` to import `pkg/agents/guard` and test `guard.full_scan` instead of `monitor.check`/`monitor.scan_actions`.

Verify: `just diagnose` passes. `just test` passes (test 39 uses guard).

## Task 2: Add merge method to std/store

**Subject:** Add Store.merge method to eliminate std/ctx dependency
**ActiveForm:** Adding merge to Store

In `crates/lx/src/stdlib/store_dispatch.rs`, add a `"merge"` arm to the store method dispatch that takes another Store or Record and copies all its entries into the target Store. In `crates/lx/src/stdlib/store.rs`, add `merge` to the module build if needed. This makes `std/ctx` fully redundant — all 7 ctx functions (`empty`=`Store()`, `get`=`.get`, `set`=`.set`, `keys`=`.keys`, `merge`=new, `remove`=`.remove`, `save`/`load`=`.save`/`.load`) are now Store operations.

Verify: `just diagnose` passes. Add a merge assertion to `tests/78_store.lx`.

## Task 3: Deprecate std/ctx

**Subject:** Mark std/ctx as deprecated
**ActiveForm:** Deprecating std/ctx

Add a deprecation notice to `std/ctx` — when loaded, emit a log.warn "std/ctx is deprecated, use Store() directly". Update any consumer in brain/, flows/, workgen/ that uses `std/ctx` to use `Store()` + dot methods + the new `.merge` method instead. If the only consumers are brain/orchestrator.lx and brain/lib/cognitive_saga.lx using `ctx.empty()`/`ctx.set`/`ctx.get`, replace with Store operations.

Verify: `just test` passes. No `use std/ctx` remains in brain/, flows/, workgen/.

## Task 4: Delete catalog.route

**Subject:** Remove duplicate route function from catalog
**ActiveForm:** Removing catalog.route

Delete the `+route` function from `pkg/agents/catalog.lx`. Search all .lx files for `catalog.route` calls. If any exist, replace with `router.quick_route {prompt: ... catalog: catalog.entries}` using `pkg/ai/router`. Update `flows/lib/specialists.lx` if it references `catalog.route`.

Verify: `just test` passes. No `catalog.route` calls remain.

## Task 5: Delete dialogue pass-through wrappers

**Subject:** Remove trivial dialogue wrappers
**ActiveForm:** Removing dialogue pass-throughs

Delete `+open_dialogue`, `+dialogue_turn`, and `+close_dialogue` from `pkg/agents/dialogue.lx`. Search all .lx files for these function calls. If any exist, replace with direct `std/agent` calls: `agent.dialogue worker {role context max_turns: 10} ^`, `agent.dialogue_turn session message ^`, `agent.dialogue_end session`.

Verify: `just test` passes.

## Task 6: Merge refine_response and refine_code

**Subject:** Unify duplicate refinement functions in quality
**ActiveForm:** Merging refine_response and refine_code

In `pkg/ai/quality.lx`, replace `+refine_response` and `+refine_code` with a single `+refine_work` that takes `(initial work task grader_fn revise_system threshold)`. The grader_fn parameter replaces the hardcoded `grade_response`/`grade_code` call. The revise_system parameter replaces the hardcoded system prompt string. The threshold parameter replaces the hardcoded 80/85. Add `+refine_response` and `+refine_code` as thin wrappers calling `refine_work` with their specific parameters so existing callers still work.

Verify: `just test` passes. `refine_response` and `refine_code` still callable.

## Task 7: Fix context_manager pressure functions

**Subject:** Remove duplicate pressure classification and fix naming
**ActiveForm:** Fixing context_manager pressure API

In `pkg/kit/context_manager.lx`: delete `+pressure_level` (callers should use `win.pressure()` directly — ContextWindow already classifies into "critical"/"high"/"moderate"/"low"). Rename `+pressure` to `+pressure_pct` to distinguish from ContextWindow.pressure which returns a string. Update `brain/lib/context_mgr.lx` to call `cm.pressure_pct` instead of `cm.pressure`.

Verify: `just test` passes.

## Task 8: Merge guard.check and guard.check_safety

**Subject:** Unify duplicate guard check functions
**ActiveForm:** Merging guard check functions

In `pkg/agents/guard.lx`, change `+check` to accept an optional second parameter `patterns` defaulting to `default_patterns`. Delete `+check_safety`. Update any caller of `check_safety` to call `check text safety_patterns` instead.

Verify: `just test` passes. No `check_safety` calls remain except through `check`.

## Task 9: Fix router.lx missing json import

**Subject:** Add missing std/json import to router
**ActiveForm:** Fixing router import

Add `use std/json` to the imports in `pkg/ai/router.lx`. Line 1378 calls `json.parse` without this import.

Verify: `just diagnose` passes.

## Task 10: Fix monitor similar_actions arity bug

**Subject:** Fix wrong argument count in monitor.suggest_strategy
**ActiveForm:** Fixing similar_actions call

In `pkg/agents/monitor.lx`, the `suggest_strategy` function calls `state.inspector.similar_actions (state.inspector.actions ()) 3` — passing 2 arguments (actions list and window size 3). But `Inspector.similar_actions` takes 1 argument (window_size) and reads `self.actions` internally. Fix the call to `state.inspector.similar_actions 3`.

Verify: `just test` passes.

## Task 11: Split monitor introspection functions to introspect

**Subject:** Move introspection functions from monitor to introspect
**ActiveForm:** Splitting monitor package

Move these 8 functions from `pkg/agents/monitor.lx` to `pkg/core/introspect.lx` as module-level exported functions (not methods on Inspector): `self_assess`, `detect_doom_loop`, `strategy_analysis`, `time_pressure`, `generate_status`, `should_pivot`, `narrate_thinking`, `suggest_pivot`. Each takes `state` as first parameter (where `state` has an `.inspector` field). Update any caller importing these from `pkg/agents/monitor` to import from `pkg/core/introspect` instead. `pkg/agents/monitor.lx` should drop below 150 lines after this.

Verify: `just test` passes. `pkg/agents/monitor.lx` under 150 lines.

## Task 12: Move tieredmem single-store helpers to memory

**Subject:** Split tieredmem abstraction levels
**ActiveForm:** Moving single-store helpers to memory

Move `+tiers`, `+thresholds`, `+create`, `+seed`, `+daily`, `+weekly` from `pkg/data/tieredmem.lx` to `pkg/data/memory.lx` as module-level exports alongside the MemoryStore Class. These operate on individual MemoryStore instances, not the multi-store `{working episodic semantic persona}` record. Update any caller that imports these from `pkg/data/tieredmem` to import from `pkg/data/memory`.

Verify: `just test` passes.

## Task 13: Move connectors from pkg/ to flows/

**Subject:** Hoist domain-specific MCP connectors to flows layer
**ActiveForm:** Moving connectors to flows/

Create `flows/connectors/` directory. Move all 6 files from `pkg/connectors/` to `flows/connectors/`: `context_engine.lx`, `forgejo.lx`, `gritql.lx`, `langfuse.lx`, `postgresql.lx`, `uptime_kuma.lx`. Update their internal imports (they use `pkg/infra/mcp_session` which stays). Update all consumer imports from `use pkg/connectors/X` to `use flows/connectors/X` — consumers are `flows/agents/gritql.lx`, `flows/agents/context_engine.lx`, `flows/examples/full_pipeline.lx`, `flows/examples/project_setup.lx`, `flows/examples/fine_tuning.lx`. Delete `pkg/connectors/` directory.

Verify: `just test` passes. `pkg/connectors/` directory does not exist.

## Task 14: Move github from pkg/ to flows/

**Subject:** Hoist GitHub client to flows layer
**ActiveForm:** Moving github to flows

Move `pkg/infra/github.lx` to `flows/lib/github.lx`. Update consumer imports: `flows/examples/discovery_system.lx` changes from `use pkg/infra/github` to `use ../lib/github`. `flows/agents/github_search.lx` changes from `use pkg/infra/github` to `use flows/lib/github` (or relative `../../lib/github`). `tests/71_workspace.lx` changes from `use pkg/infra/github` to `use flows/lib/github`.

Verify: `just test` passes.

## Task 15: Move training from pkg/ to flows/

**Subject:** Hoist training pipeline to flows layer
**ActiveForm:** Moving training to flows

Move `pkg/data/training.lx` to `flows/lib/training.lx`. Update consumer import: `flows/examples/fine_tuning.lx` changes from `use pkg/data/training` to `use ../lib/training`.

Verify: `just test` passes.

## Task 16: Generalize tool_executor dispatch

**Subject:** Make tool executor accept custom dispatch function
**ActiveForm:** Generalizing tool_executor

In `pkg/kit/tool_executor.lx`, change `+execute` to accept an optional third parameter `dispatch_fn` that handles tool dispatch for non-builtin tools. Change `execute_single` to call `dispatch_fn request` instead of returning `Err "requires MCP dispatch"` when the tool is not "Bash" or "Read". Default `dispatch_fn` to `(req) Err "tool {req.tool} requires MCP dispatch"` when not provided. Update `brain/lib/tools.lx` if it calls `executor.execute` — pass the existing dispatch pattern.

Verify: `just diagnose` passes.

## Task 17: Deprecate std/agents/planner, router, reviewer

**Subject:** Mark redundant Rust agents as deprecated
**ActiveForm:** Deprecating redundant std agents

In `crates/lx/src/stdlib/agents_planner.rs`, `agents_router.rs`, `agents_reviewer.rs`: add an `eprintln!` deprecation warning in each `bi_*` function that fires once (use `std::sync::Once`), directing callers to use `pkg/ai/planner`, `pkg/ai/router`, `pkg/ai/reviewer` instead. Do not remove the modules — tests still reference them. Update the 3 test files (`tests/36_agents_planner.lx`, `tests/34_agents_router.lx`, `tests/40_agents_reviewer.lx`) to also test the pkg/ equivalents alongside the std/ versions.

Verify: `just test` passes.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
