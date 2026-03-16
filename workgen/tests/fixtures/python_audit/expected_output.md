# Goal

Fix Python code quality violations: dynamic attribute access patterns, swallowed errors, mutable default arguments, string literals used as enum values, free functions that should be methods, intermediate list materializations, dicts used instead of structured types, and bare except clauses.

# Why

The codebase has multiple quality issues: `dict` used for structured data with known keys instead of a dataclass or Pydantic model, string literals "active"/"inactive"/"pending" representing a fixed set of statuses that should be a StrEnum, free functions `store_item` and `fetch_item` that take a DataStore as first argument and should be methods, `sum([x["value"] for x in items])` materializes an intermediate list where a generator expression suffices, a mutable default argument `items=[]` that is shared across calls, a bare `except: pass` that swallows all errors silently, and `data.get(key, None)` where None masks a real absence.

# What changes

- Replace `dict` return type in `load_config` and `process_items` with a dataclass or Pydantic model
- Replace string literals "active", "inactive", "pending" with a StrEnum
- Move `store_item` and `fetch_item` into DataStore as methods
- Replace `sum([x["value"] for x in items])` with `sum(x["value"] for x in items)` — remove intermediate list
- Replace mutable default `items=[]` with `items=None` and create inside function body
- Replace bare `except: pass` with specific exception handling — do not swallow errors
- Replace `data.get(key, None)` with explicit handling of missing keys

# Files affected

- src/service.py — dict as structured type, string literal enum values, free functions on DataStore, intermediate list in sum(), mutable default argument, swallowed error via bare except, inappropriate defaulting

# Task List

## Task 1: Define structured types

Replace raw `dict` returns with dataclasses. Define a `Config` dataclass for `load_config` return. Define a `ProcessResult` dataclass for `process_items` return.

```
just fmt
git add src/service.py
git commit -m "fix: replace dict with dataclasses for structured data"
```

## Task 2: Replace string literals with StrEnum

Define a `Status` StrEnum with `ACTIVE`, `INACTIVE`, `PENDING` variants. Replace string comparisons in `get_status`.

```
just fmt
git add src/service.py
git commit -m "fix: replace status string literals with StrEnum"
```

## Task 3: Convert free functions to methods

Move `store_item` and `fetch_item` into the `DataStore` class as methods.

```
just fmt
git add src/service.py
git commit -m "fix: move free functions into DataStore as methods"
```

## Task 4: Fix error handling and defaults

Replace bare `except: pass` with `except ValueError` and log or propagate. Replace mutable default `items=[]` with `items=None`. Remove intermediate list in `sum()`. Fix `.get(key, None)` defaulting.

```
just fmt
git add src/service.py
git commit -m "fix: proper error handling, remove mutable default and intermediate list"
```

## Task 5: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify python audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No raw dicts for structured data — use dataclasses or Pydantic
- No string literals as enum values — use StrEnum
- No mutable default arguments
- No bare except clauses

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
