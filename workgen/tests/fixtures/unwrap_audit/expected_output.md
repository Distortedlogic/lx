# Goal

Eliminate all unwrap calls, panics, and unsafe error handling patterns from the codebase, replacing them with proper Result propagation and contextual error messages.

# Why

The codebase uses .unwrap() extensively in both application and library code, risking runtime panics on invalid input, missing files, or lock contention. A mutable static Mutex compounds this by making state management fragile. Every failure path must return a descriptive Result instead.

# What changes

- Replace all .unwrap() calls with ? operator or .map_err() with context
- Replace panic!() in db.rs with Result return type
- Replace mutable static Mutex with dependency injection or OnceLock
- Make all public functions return Result when they can fail
- Add error context describing what operation failed and why

# Files affected

- src/main.rs — 7 unwrap calls across load_config, get_port, read_users
- src/db.rs — 3 unwrap calls on Mutex::lock, 1 panic!(), mutable static DB

# Task List

## Task 1: Fix src/main.rs error handling

Replace all .unwrap() calls in load_config, get_port, and read_users with ? propagation. Change return types to Result. Add error context for file I/O: include the file path and operation in error messages.

```
just fmt
git add src/main.rs
git commit -m "fix: replace unwrap with Result propagation in main.rs"
```

## Task 2: Fix src/db.rs error handling

Replace .unwrap() calls on Mutex::lock with proper error handling. Replace panic!("index out of bounds") with returning an Err. Remove mutable static DB — accept the data store as a parameter or use OnceLock. Make all public functions return Result with descriptive error context.

```
just fmt
git add src/db.rs
git commit -m "fix: replace unwrap/panic with Result in db.rs, remove mutable static"
```

## Task 3: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: final verification after unwrap audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- Never use .unwrap() — always propagate with ? or handle explicitly
- Error messages must say what failed and why

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit. After all tasks, run `just test` and `just diagnose` to verify.
