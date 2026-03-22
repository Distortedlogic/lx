# Goal

Add checkpoint-based durable execution to lx workflows so long-running agent programs can resume from the last successful step after a crash. Currently a multi-step saga that fails on step 7 of 10 must re-execute all 7 completed steps on retry — wasting LLM tokens, time, and producing non-deterministic results for already-completed work.

# Why

- The research in `research/workflow-dsls/design-patterns.md` covers Temporal and Restate's durable execution model extensively — deterministic replay, memoization, exactly-once semantics. This is the feature that separates a scripting language from a workflow orchestration language.
- LLM calls are expensive and non-deterministic. Re-executing a completed AI reasoning step produces different output, potentially invalidating downstream steps that were correct.
- `pkg/core/saga.lx` already implements compensation (undo on failure) but not replay (skip on retry). Durable execution complements sagas.
- The `Store` type already provides persistent key-value storage with optional file-backed persistence — checkpoint storage follows a similar file-based pattern but stores individual JSON files per step rather than using the Store abstraction.

# What Changes

## New stdlib module: `std/checkpoint`

New file `crates/lx/src/stdlib/checkpoint.rs` implementing 3 functions:

**`checkpoint.scope name opts body -> Result`** — Creates a durable execution scope. `name` is the workflow ID (used as the storage key prefix). `opts` contains `store_path` (Str, default `.lx-checkpoints/`). `body` is a function that receives a `checkpoint` function. Returns the body's return value, or replays from storage if all checkpoints are already completed.

**`checkpoint.step name body -> Result`** — Inside a `checkpoint.scope`, marks a checkpoint. On first execution, evaluates `body`, serializes the result to the checkpoint store, and returns it. On subsequent executions (replay), deserializes and returns the stored result without calling `body`. `name` must be unique within the scope.

**`checkpoint.clear name -> ()`** — Clears all checkpoint state for a given scope name, forcing full re-execution.

## Usage pattern

```
use std/checkpoint

checkpoint.scope "long_task" {:} (step) {
  data = step "fetch" { http.get "https://api.example.com/data" }
  analysis = step "analyze" { ai.ask "Analyze: {data}" }
  result = step "transform" { transform analysis }
  result
}
```

On first run, all 3 steps execute and checkpoint. If the process crashes after "analyze" completes, on retry "fetch" and "analyze" return stored results, only "transform" re-executes.

## Serialization

Checkpoint values are serialized via `serde_json`. `LxVal` implements `Serialize` and `Deserialize` manually in `crates/lx/src/value/serde_impl.rs` (not via derive). However, serialization is lossy for non-data variants:
- `Func`/`BuiltinFunc`/`Trait`/`Class` serialize as the string `"<{type_name}>"` via the catch-all arm without erroring — `checkpoint.step` must explicitly check the return value type and return an error if it contains a Func, rather than relying on serde to fail.
- `Ok(v)` and `Some(v)` serialize as their inner value (unwrapped, no marker key) — they lose their wrapper on round-trip.
- `Err(v)` serializes with `__err` marker, `Tagged` with `__tag`/`__values`, `Store` with `__store`, `Object` with `__object`/`__class` — but deserialization (via `serde_json::Value -> LxVal`) does not reconstruct these marker-keyed types. They round-trip as plain Records.
- `Range` serializes with `start`/`end`/`inclusive` keys but likewise deserializes as a plain Record.
- For simple data checkpoints (Int, Float, Bool, Str, List, Record) round-tripping is lossless.
- Functions and closures cannot be checkpointed — `checkpoint.step` must return an error if the body returns a value containing `Func` or `BuiltinFunc`.

## Storage

Checkpoints are stored as JSON files in `.lx-checkpoints/{scope_name}/{step_name}.json`. This keeps them inspectable, diffable, and git-ignoreable.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/checkpoint.rs` — scope, step, clear implementation
- `tests/79_checkpoint.lx` — unit tests for checkpoint/replay behavior (note: `tests/` directory does not exist yet and must be created; `just test` runs `cargo run -p lx-cli -- test tests/` which discovers `*.lx` files in that directory)

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — add `mod checkpoint;` declaration, add `"checkpoint" => checkpoint::build()` to the `get_std_module` match, and add `"checkpoint"` to the `std_module_exists` `matches!` macro

# Task List

### Task 1: Implement checkpoint storage layer

**Subject:** Create checkpoint file storage with read/write/clear operations

**Description:** Create `crates/lx/src/stdlib/checkpoint.rs`. Implement internal helpers: `checkpoint_dir(store_path, scope_name) -> PathBuf`, `write_checkpoint(dir, step_name, value: &LxVal) -> Result<(), LxError>` (serialize to JSON, write atomically via temp file + rename), `read_checkpoint(dir, step_name) -> Option<LxVal>` (read + deserialize, return None if file doesn't exist), `clear_checkpoints(dir) -> Result<(), LxError>` (remove the directory). Implement `bi_clear(name)` that calls `clear_checkpoints`. Register the module in `stdlib/mod.rs`: add `mod checkpoint;` declaration, add `"checkpoint" => checkpoint::build()` to the `get_std_module` match, and add `"checkpoint"` to the `std_module_exists` `matches!` macro. Add `"clear"` to the `build()` map in checkpoint.rs.

Run `just diagnose`.

**ActiveForm:** Implementing checkpoint storage layer

---

### Task 2: Implement checkpoint.scope and checkpoint.step

**Subject:** Durable execution scope with step replay logic

**Description:** In `crates/lx/src/stdlib/checkpoint.rs`, implement `bi_scope(name, opts, body)`:
1. Parse `store_path` from opts (default `.lx-checkpoints/`).
2. Compute the checkpoint directory.
3. Create a `step` builtin as `LxVal::BuiltinFunc` with arity 3 and the checkpoint directory path pre-applied as the first argument (stored in the `applied` vec). `SyncBuiltinFn` is a plain function pointer (`fn(...)`) — it cannot capture state — so use partial application to pass the directory. The step function `bi_step(args)` receives `args[0]` = dir path (Str), `args[1]` = step_name (Str), `args[2]` = body_fn (Func):
   a. Check if `read_checkpoint(dir, step_name)` returns Some — if so, return the stored value.
   b. Otherwise, call `body_fn()` via `call_value_sync` (from `crate::builtins`), serialize the result, call `write_checkpoint`, return the result.
   c. If the result contains `Func` or `BuiltinFunc`, return an error (serde Serialize won't fail — it silently produces `"<Func>"` — so check explicitly before writing).
4. Call `body(step)` via `call_value_sync`.
5. Return the body's result.

Add `"scope"` and `"step"` to the `build()` map. The `"step"` entry can be a placeholder that errors outside a scope — the real step function is created dynamically inside `bi_scope`.

Run `just diagnose`.

**ActiveForm:** Implementing scope and step functions

---

### Task 3: Write checkpoint tests

**Subject:** Test checkpoint, replay, and clear behavior

**Description:** Create the `tests/` directory if it does not exist, then create `tests/79_checkpoint.lx`. Tests:
1. **Basic checkpoint** — scope with 2 steps, verify both execute and return correct values.
2. **Replay behavior** — scope with 2 steps where step 1 writes a counter file. Run scope twice. On second run, step 1 should replay (counter file not written again). Verify by checking counter.
3. **Clear** — run scope, call `checkpoint.clear`, run scope again, verify all steps re-execute.
4. **Non-serializable error** — step that returns a function, verify error message.

Run `just diagnose` and `just test`.

**ActiveForm:** Writing checkpoint tests

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
mcp__workflow__load_work_item({ path: "work_items/DURABLE_EXECUTION.md" })
```

Then call `next_task` to begin.
