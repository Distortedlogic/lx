# Goal

Simplify unnecessary abstraction layers: proxy types, shuttle types, call chain depth exceeding 4 hops, repeated lock acquisition, and configuration parameter threading.

# Why

DataHandle is a proxy type — every method follows the same pattern of acquiring a read/write lock on the inner Vec and forwarding to it. TransferPayload is a shuttle type constructed at one site and immediately consumed by `compute`. The call chain `process` → `run_inner` → `do_work` has 3 forwarding hops where intermediate functions add no logic beyond parameter passing. `analyze` acquires the lock 4 separate times on DataHandle when a single lock acquisition would serve all reads. Config fields are threaded as individual parameters through `process` → `run_inner` → `do_work` instead of passing the Config struct directly.

# What changes

- Replace DataHandle proxy with direct access to `Arc<RwLock<Vec<f64>>>` or a single `with_data(|data| ...)` method
- Remove TransferPayload shuttle type — pass values and label directly to compute
- Inline run_inner into process — it adds no logic
- In analyze: lock once, compute all values from the single lock guard
- Pass Config struct directly instead of threading individual fields as parameters

# Files affected

- src/main.rs — proxy type DataHandle, shuttle type TransferPayload, call chain depth >4 hops, repeated lock acquisition in analyze, config parameter threading

# Task List

## Task 1: Remove proxy type DataHandle

Replace DataHandle with direct access to `Arc<RwLock<Vec<f64>>>`. In analyze, lock once and compute all values.

```
just fmt
git add src/main.rs
git commit -m "fix: remove DataHandle proxy, lock once in analyze"
```

## Task 2: Remove shuttle type and flatten call chain

Remove TransferPayload — pass fields directly. Inline run_inner into process — it only forwards parameters.

```
just fmt
git add src/main.rs
git commit -m "fix: remove shuttle type, inline forwarding function"
```

## Task 3: Fix config parameter threading

Pass the Config struct directly to process and do_work instead of extracting and threading individual fields.

```
just fmt
git add src/main.rs
git commit -m "fix: pass Config struct instead of threading fields"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify topology audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No proxy types that only forward to inner
- No shuttle types with single construction and consumption site
- No call chains >4 hops with forwarding-only intermediaries

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
