# Goal

Unwind five topology knots in `workgen/tests/fixtures/topology_audit/src/main.rs` identified by the topology simplification audit: eliminate the `DataHandle` proxy type, delete the `TransferPayload` shuttle type, inline the `run_inner` forwarding function, fix repeated lock acquisition in `analyze`, and replace config parameter threading with a `&Config` reference.

# Why

- `DataHandle` wraps `Arc<RwLock<Vec<f64>>>` behind five methods that all follow the same lock-and-forward pattern, adding no invariant enforcement — callers pay lock overhead per call and cannot batch reads under a single guard
- `TransferPayload` is constructed at one site and consumed at one site where only the `values` field is used; the `label` field is never read, making the struct pure transfer overhead
- `run_inner` is called from exactly one site (`process`) and does nothing but forward all four parameters to `do_work` — a pure forwarding hop with no logic
- `analyze` acquires the `RwLock` read guard four separate times in sequence (via `handle.len()`, `handle.sum()`, `handle.avg()`, `handle.max()`) when a single acquisition suffices
- All four `Config` fields are destructured at the call site in `main` and threaded individually through `process` → `run_inner` → `do_work`, bloating every signature with four parameters that originate from one struct

# What changes

**Delete `DataHandle` proxy type (lines 10–31):** Remove the `DataHandle` struct and its entire `impl` block. Replace the `handle` usage in `analyze` with direct access to the `Arc<RwLock<Vec<f64>>>`. The `analyze` function acquires the read lock once and computes len, sum, avg, and max from the single guard.

**Delete `TransferPayload` shuttle type (lines 33–36):** Remove the struct definition. Change `compute` to accept `values: Vec<f64>` directly instead of `TransferPayload`. Update the call site in `main` to pass the `Vec<f64>` directly, eliminating the struct construction and the dead `label` field.

**Inline `run_inner` into `process` (lines 49–51):** Delete `run_inner` entirely. Replace the call in `process` with a direct call to `do_work`.

**Replace config parameter threading with `&Config` (lines 42–55, 65–72):** Change `process` and `do_work` to accept `&Config` instead of four individual parameters. Update `main` to pass `&config` instead of destructuring four fields. Remove the now-inlined `run_inner` from this chain (already handled above).

**Fix repeated lock acquisition in `analyze` (lines 57–63):** After `DataHandle` is removed, `analyze` takes `&Arc<RwLock<Vec<f64>>>` and acquires the read guard once at the top, then computes all four statistics from that single guard.

# Files affected

- `workgen/tests/fixtures/topology_audit/src/main.rs` — all changes are in this single file: delete `DataHandle` struct and impl, delete `TransferPayload` struct, inline `run_inner`, change `process`/`do_work` signatures to take `&Config`, rewrite `analyze` to take `&Arc<RwLock<Vec<f64>>>` and lock once

# Task List

## Task 1: Delete DataHandle proxy and rewrite analyze with single lock acquisition

**File:** `workgen/tests/fixtures/topology_audit/src/main.rs`

Remove the `DataHandle` struct (lines 10–12) and its entire `impl` block (lines 14–31). Change the `analyze` function signature to accept `data: &Arc<RwLock<Vec<f64>>>` instead of `handle: &DataHandle`. Inside `analyze`, acquire the read guard once with `let data = data.read().unwrap();` and compute all four values (len, sum, avg, max) directly from that single guard using iterator methods. Update any call site of `analyze` in `main` — if `main` currently constructs a `DataHandle`, construct the `Arc<RwLock<Vec<f64>>>` directly and pass a reference to `analyze`. If `main` does not call `analyze`, add a call that creates sample data in an `Arc<RwLock<Vec<f64>>>` and passes it.

After completing the implementation:
- Run `just fmt`
- Stage changes with `git add`
- Commit with a descriptive message

## Task 2: Delete TransferPayload shuttle and pass Vec directly to compute

**File:** `workgen/tests/fixtures/topology_audit/src/main.rs`

Remove the `TransferPayload` struct definition. Change `compute` signature from `fn compute(payload: TransferPayload) -> f64` to `fn compute(values: Vec<f64>) -> f64`. Update the body to call `values.iter().sum()` directly. In `main`, remove the `TransferPayload` construction and pass `vec![1.0, 2.0, 3.0]` directly to `compute`.

After completing the implementation:
- Run `just fmt`
- Stage changes with `git add`
- Commit with a descriptive message

## Task 3: Inline run_inner and replace config parameter threading with &Config

**File:** `workgen/tests/fixtures/topology_audit/src/main.rs`

Delete the `run_inner` function entirely. Change `process` signature to `fn process(config: &Config)`. Inside `process`, access fields as `config.threshold`, `config.max_items`, `config.batch_size`, `config.verbose`. Replace the call to `run_inner(...)` with a direct call to `do_work(config)`. Change `do_work` signature to `fn do_work(_config: &Config)`. In `main`, replace the destructured call `process(config.threshold, config.max_items, config.batch_size, config.verbose)` with `process(&config)`.

After completing the implementation:
- Run `just fmt`
- Stage changes with `git add`
- Commit with a descriptive message

## Task 4: Verification

Run the full verification suite to confirm all changes compile and pass:

- Run `just test`
- Run `just diagnose`
- Run `just fmt`

Fix any issues found before marking complete.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

# Task Loading Instructions

To begin execution, run:

```
mcp__workflow__load_work_item({ path: "work_items/TOPOLOGY_SIMPLIFICATION_AUDIT.md" })
```

Then call `mcp__workflow__next_task` to get the first task and begin implementation.