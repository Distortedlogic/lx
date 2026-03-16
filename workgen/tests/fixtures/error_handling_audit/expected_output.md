# Goal

Eliminate all swallowed errors, silent fallbacks, and string-typed errors from the codebase, replacing them with typed error enums and explicit propagation.

# Why

The codebase systematically hides failures: .unwrap_or_default() silently returns empty data when I/O fails, let _ = discards write errors, .ok() converts failures to None without logging, and string-typed errors lose structure. Callers receive empty collections with no indication that loading failed, leading to silent data loss.

# What changes

- Remove all .unwrap_or_default() calls — propagate errors to callers instead of returning empty defaults
- Remove all let _ = patterns on Result values — handle or propagate the error
- Remove all .ok() calls that swallow errors — use ? or match instead
- Replace String error types with typed error enums using thiserror
- Add error context to every I/O operation describing what file and operation failed
- Make failed loop iterations collect errors instead of silently skipping via .ok()

# Files affected

- src/service.rs — .unwrap_or_default(), let _ =, .ok() on errors, silent empty fallbacks
- src/api.rs — .unwrap_or_default(), String error returns, swallowed file I/O errors

# Task List

## Task 1: Create error types

Define a typed error enum with thiserror for service and API errors. Remove all String-typed error returns.

```
just fmt
git add src/
git commit -m "fix: add typed error enum, remove String errors"
```

## Task 2: Fix src/service.rs error handling

Remove .unwrap_or_default() — return Result instead of empty collections. Remove let _ = on Result values — propagate or log. Remove .ok() calls — use ? operator. Add context to error messages describing what operation failed.

```
just fmt
git add src/service.rs
git commit -m "fix: stop swallowing errors in service.rs"
```

## Task 3: Fix src/api.rs error handling

Remove .unwrap_or_default() calls — propagate errors. Replace String error returns with typed errors. Fix swallowed file I/O errors — propagate with context. Ensure no silent fallback to empty collections.

```
just fmt
git add src/api.rs
git commit -m "fix: stop swallowing errors in api.rs"
```

## Task 4: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: final verification after error handling audit"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- Never swallow errors with .ok(), let _ =, or .unwrap_or_default()
- Every I/O error must include context about what file and operation failed

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit. After all tasks, run `just test` and `just diagnose` to verify.
