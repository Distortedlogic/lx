# Goal

Add supervision tree primitives to lx so agents can declare restart policies and be automatically managed by a supervisor. Currently agents are manually spawned and killed — the language has no opinions about what happens when an agent crashes. Supervision closes the gap between lx's ambition ("Terraform for agentic programming") and its current fire-and-forget agent model.

# Why

- Every non-trivial agentic workflow has agents that crash — network failures, LLM timeouts, OOM. Without supervision, every caller manually wraps spawn/ask/kill in retry logic.
- The research in `research/concurrency/design-patterns.md` covers Erlang/OTP supervision trees (one-for-one, one-for-all, rest-for-one strategies, intensity thresholds) — the theory is done, the implementation isn't.
- `pkg/core/retry.lx` and `pkg/core/circuit.lx` are user-space workarounds for what should be a language-level primitive.
- Supervision composes — a supervised group of agents can itself be supervised, creating a hierarchy that isolates failure domains.

# What Changes

## New stdlib module: `std/supervise`

New file `crates/lx/src/stdlib/supervise/mod.rs` implementing 4 functions:

**`supervise.one_for_one specs opts -> SupervisorHandle`** — Spawns a set of child agents from `specs` (list of child spec records). If one child crashes, only that child is restarted. Opts: `max_restarts` (Int, default 3), `window_secs` (Int, default 60), `on_max` (Func called when restart limit exceeded).

**`supervise.one_for_all specs opts -> SupervisorHandle`** — If one child crashes, all children are killed and restarted. Same opts.

**`supervise.rest_for_one specs opts -> SupervisorHandle`** — If one child crashes, that child and all children started after it are restarted. Order is the order in `specs`.

**`supervise.stop handle -> Result`** — Gracefully stops all children and the supervisor.

## Child spec format

Each child spec is a record:
```
{
  id: "worker-1"
  command: "lx"
  args: ["run" "worker.lx"]
  restart: :permanent | :transient | :temporary
  shutdown: 5000
}
```

- `:permanent` — always restart on exit
- `:transient` — restart only on abnormal exit (error/crash, not clean return)
- `:temporary` — never restart

## SupervisorHandle

A record with `{id: Str, children: [AgentHandle], stop: Func}` that can be used with `supervise.stop` or inspected for child status.

## Runtime changes

The supervisor runs as a tokio task that monitors child process exit codes. On child exit:
1. Check restart policy
2. Check restart intensity (count within window)
3. If within limits, respawn. If exceeded, call `on_max` or propagate error.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/supervise/mod.rs` — supervisor strategies + child spec handling
- `crates/lx/src/stdlib/supervise/strategy.rs` — one_for_one, one_for_all, rest_for_one logic
- `tests/70_supervise.lx` — unit tests for supervision

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — register `mod supervise;`, add to `get_std_module`

# Task List

### Task 1: Create child spec parsing and supervisor state types

**Subject:** Define ChildSpec and SupervisorState structs for supervision

**Description:** Create `crates/lx/src/stdlib/supervise/mod.rs`. Define internal structs: `ChildSpec { id, command, args, restart_policy, shutdown_ms }` and `SupervisorState { children, max_restarts, window_secs, restart_log }`. Implement `parse_child_spec(val: &LxVal) -> Result<ChildSpec, LxError>` to extract fields from an lx record value. Implement `parse_opts(val: &LxVal) -> SupervisorOpts` for supervisor-level options. Register the module in `stdlib/mod.rs`. Create the `build()` function returning an empty `IndexMap` for now.

Run `just diagnose`.

**ActiveForm:** Defining supervisor state types

---

### Task 2: Implement one_for_one supervisor

**Subject:** Core supervision loop with single-child restart strategy

**Description:** In `crates/lx/src/stdlib/supervise/mod.rs`, implement `bi_one_for_one(specs, opts)`. This function:
1. Parses child specs and supervisor opts from the lx values.
2. Spawns each child via the shell backend's process spawning (or directly via `tokio::process::Command`).
3. Spawns a supervisor tokio task that `select!`s on all child process exit signals.
4. On child exit: check restart policy (permanent/transient/temporary), check restart intensity (track restart timestamps, count within window), respawn if allowed, call `on_max` if limit exceeded.
5. Returns a `SupervisorHandle` record containing the supervisor task handle and child list.

Add `"one_for_one"` to the `build()` map.

Run `just diagnose`.

**ActiveForm:** Implementing one_for_one supervision

---

### Task 3: Implement one_for_all and rest_for_one strategies

**Subject:** Add remaining supervision strategies

**Description:** Create `crates/lx/src/stdlib/supervise/strategy.rs`. Factor the restart logic from Task 2 into a `Strategy` enum with variants `OneForOne`, `OneForAll`, `RestForOne`. Implement `bi_one_for_all` and `bi_rest_for_one`:
- `one_for_all`: on any child exit, kill all children (send shutdown signal, wait `shutdown_ms`, then force kill), then restart all in order.
- `rest_for_one`: on child N exit, kill children N+1..end, then restart N..end in order.

Add `"one_for_all"` and `"rest_for_one"` to the `build()` map.

Run `just diagnose`.

**ActiveForm:** Implementing remaining supervision strategies

---

### Task 4: Implement supervise.stop and write tests

**Subject:** Graceful shutdown and test suite for supervision

**Description:** Implement `bi_stop(handle)` — sends shutdown signal to all children, waits for graceful exit, then cancels the supervisor task. Add `"stop"` to the `build()` map.

Create `tests/70_supervise.lx` with tests:
1. Spawn a supervisor with one child, verify it starts, stop it, verify clean exit.
2. Spawn with `:temporary` child, simulate child exit, verify no restart.
3. Verify restart intensity limit — set `max_restarts: 1, window_secs: 60`, trigger 2 crashes, verify supervisor calls `on_max`.

Run `just diagnose` and `just test`.

**ActiveForm:** Implementing stop and writing supervision tests

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
mcp__workflow__load_work_item({ path: "work_items/SUPERVISION_TREES.md" })
```

Then call `next_task` to begin.
