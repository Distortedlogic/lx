# Goal

Remediate all audit violations found in `workgen/tests/fixtures/python_audit/src/service.py`. The file has 13 distinct violations spanning dead imports, inappropriate defaults, missing enums, free functions that should be methods, swallowed errors, mutable default arguments, intermediate list materializations, overly specific parameter types, verbose patterns, untyped dicts used where structured types belong, and a CLI entrypoint not using Typer. Each violation is a binary pass/fail — every one must be fully resolved.

# Why

- Dead imports (`os`, `sys`) add noise and trigger linter failures (F401)
- Intermediate list materialization in `sum([...])` wastes memory for no benefit
- Overly specific parameter types (`list`, `dict`) prevent callers from passing other iterables/mappings and break type-checker guidance
- Inappropriate defaults (`.get("users", {})`, `.get(name, "")`, `.get(key, None)`) silently mask malformed data and missing entries, deferring errors to harder-to-debug downstream sites
- String literals `"active"`, `"inactive"`, `"pending"` compared without enum protection allow typos to silently do the wrong thing and provide no exhaustiveness checking
- Free functions `store_item` and `fetch_item` take `DataStore` as first arg and access its internals — they belong as methods on the class
- Bare `except: pass` swallows `KeyboardInterrupt`, `SystemExit`, and every other exception silently
- Mutable default argument `items=[]` is shared across calls, a classic Python bug
- `process_items` returns a dict with fixed keys but is annotated as returning `list` — a dataclass gives type safety, attribute access, and correct annotation
- `load_config` returns raw `dict` accessed with known string keys downstream — a typed model catches schema mismatches at parse time
- `list(filter(lambda ...))` is verbose where a list comprehension is idiomatic
- The `__main__` block is an ad-hoc CLI entrypoint that should use Typer

# What changes

**Dead imports (Finding 1):** Remove `import os` (line 2) and `import sys` (line 3).

**Intermediate list materialization (Finding 2):** Change `sum([x["value"] for x in items])` to `sum(x["value"] for x in items)`.

**Overly specific parameter types (Finding 3):** Change `process_items(items: list)` parameter type to `Iterable[dict[str, Any]]`. Change `get_user(data: dict, ...)` parameter type to `Mapping[str, Any]`. Add imports for `Iterable` and `Mapping` from `collections.abc` and `Any` from `typing`.

**Inappropriate defaulting — get_user (Finding 4):** Replace `data.get("users", {}).get(name, "")` with direct key access `data["users"][name]`, letting `KeyError` propagate to the caller on malformed data or missing user. Remove the `Optional` return type — the function now always returns `str` or raises. Remove the conditional `return result if result else None` — just return the value directly.

**Inappropriate defaulting — fetch_item (Finding 5):** Change `store.data.get(key, None)` to `store.data.get(key)` since `.get()` already returns `None` by default, removing the redundant explicit `None`.

**String literals to enum (Finding 6):** Define a `StatusCode(StrEnum)` with members `ACTIVE = "active"`, `INACTIVE = "inactive"`, `PENDING = "pending"`. Change `get_status` to accept `StatusCode` and compare against enum members. Import `StrEnum` from `enum`.

**Free functions to methods (Finding 7):** Move `store_item` and `fetch_item` into `DataStore` as methods `store` and `fetch`. Update any call sites.

**Swallowed error (Finding 8):** Replace bare `except: pass` with `except ValueError: continue` to catch only the specific exception from `int()` conversion and skip that item explicitly.

**Mutable default argument (Finding 9):** Change `run_pipeline(items=[])` to `run_pipeline(items=None)`, with `if items is None: items = []` inside the function body.

**Dicts to structured types — process_items return (Finding 10):** Define a `ProcessResult` dataclass with fields `total: int`, `names: list[str]`, `filtered: list[dict[str, Any]]`. Change `process_items` return type annotation from `list` to `ProcessResult`. Return a `ProcessResult(...)` instead of a dict.

**Verbose pattern (Finding 11):** Replace `list(filter(lambda x: x["value"] > 0, items))` with `[x for x in items if x["value"] > 0]`.

**Dicts to structured types — load_config return (Finding 12):** Define an `AppConfig` dataclass with a `users: dict[str, str]` field (matching the known schema accessed downstream). Change `load_config` return type to `AppConfig`. Parse the JSON dict into `AppConfig` before returning. Update `get_user` call site to pass `config.users` directly or adjust `get_user` to accept `AppConfig`.

**CLI without Typer (Finding 13):** Replace the `if __name__ == "__main__"` block with a Typer app. Define `app = typer.Typer()` and a `@app.command()` function `main` that takes `config_path: str` as a Typer argument and `status_code: StatusCode` as an option defaulting to `StatusCode.ACTIVE`. Add `import typer`.

# Files affected

- `workgen/tests/fixtures/python_audit/src/service.py` — All 13 findings are in this single file. Every change described above applies here.

# Task List

## Task 1: Remove dead imports and add required imports

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Remove `import os` (line 2) and `import sys` (line 3). Replace `from typing import Optional` with `from typing import Any`. Add `from collections.abc import Iterable, Mapping`. Add `from dataclasses import dataclass`. Add `from enum import StrEnum`. Add `import typer`. Keep `import json` as-is. The final import block should have six import statements: `json`, `collections.abc` imports, `dataclasses`, `enum`, `typing`, and `typer`.

**Verify:** No `os` or `sys` imports remain. All new imports present.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: remove dead imports, add required imports for audit fixes"`.

## Task 2: Define StatusCode enum and AppConfig/ProcessResult dataclasses

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

After the import block, define three new types in this order:

1. `StatusCode(StrEnum)` with members `ACTIVE = "active"`, `INACTIVE = "inactive"`, `PENDING = "pending"`.
2. `AppConfig` as a `@dataclass` with one field: `users: dict[str, str]`.
3. `ProcessResult` as a `@dataclass` with three fields: `total: int`, `names: list[str]`, `filtered: list[dict[str, Any]]`.

**Verify:** Three new type definitions exist after imports. No string literals for status codes remain outside the enum definition.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: define StatusCode enum, AppConfig and ProcessResult dataclasses"`.

## Task 3: Fix load_config to return AppConfig

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Change `load_config` return type from `dict` to `AppConfig`. After `data = json.load(f)`, return `AppConfig(users=data["users"])` instead of returning the raw dict. This parses the JSON into a typed model at the boundary.

**Verify:** `load_config` return annotation is `AppConfig`. Function body constructs and returns an `AppConfig` instance.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: load_config returns typed AppConfig instead of raw dict"`.

## Task 4: Fix get_user — parameter type, remove inappropriate defaults

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Change `get_user` signature from `(data: dict, name: str) -> Optional[str]` to `(data: Mapping[str, Any], name: str) -> str`. Replace the body: remove `data.get("users", {}).get(name, "")` and the conditional return. The body should simply be `return data["users"][name]`, letting `KeyError` propagate on malformed data or missing user.

Note: after Task 3, the `__main__` block will pass `config.users` (a `dict[str, str]`) to `get_user`, so `get_user` receives the users mapping directly. Alternatively, adjust the call in `__main__` to pass `config.users` and update `get_user` to just do `return data[name]` since it now receives the users sub-dict directly. Choose the simpler approach: `get_user` receives the users sub-mapping and does `return data[name]`.

**Verify:** No `.get(` calls remain in `get_user`. Return type is `str`, not `Optional[str]`. Parameter type is `Mapping[str, Any]`.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: fix get_user param type and remove inappropriate defaults"`.

## Task 5: Fix process_items — parameter type, generator expr, comprehension, return type

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Apply four changes to `process_items`:

1. Change parameter type from `items: list` to `items: Iterable[dict[str, Any]]`.
2. Change return type from `list` to `ProcessResult`.
3. Change `sum([x["value"] for x in items])` to `sum(x["value"] for x in items)` (remove intermediate list).
4. Replace `list(filter(lambda x: x["value"] > 0, items))` with `[x for x in items if x["value"] > 0]`.
5. Return `ProcessResult(total=total, names=names, filtered=filtered)` instead of a dict.

Important: since `items` is now `Iterable`, it can only be iterated once. The function iterates `items` three times (for total, names, filtered). Convert `items` to a `list` at the top of the function body: `items = list(items)`. This is acceptable because the materialization serves a purpose (multiple passes).

**Verify:** No `sum([` pattern. No `filter(lambda` pattern. Return type is `ProcessResult`. Parameter type is `Iterable[dict[str, Any]]`.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: fix process_items types, remove intermediate list, use comprehension"`.

## Task 6: Fix get_status to use StatusCode enum

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Change `get_status` signature to accept `code: StatusCode` and return `bool | None`. Replace string comparisons: `code == StatusCode.ACTIVE` returns `True`, `code == StatusCode.INACTIVE` returns `False`, `code == StatusCode.PENDING` returns `None`. The final fallback `return None` remains unchanged.

**Verify:** No string literals `"active"`, `"inactive"`, `"pending"` appear in `get_status`. Parameter is typed `StatusCode`.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: get_status uses StatusCode enum instead of string literals"`.

## Task 7: Move store_item and fetch_item into DataStore as methods

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Delete the free functions `store_item` (line 34) and `fetch_item` (line 37). Add two methods to `DataStore`:

1. `store(self, key: str, value: Any) -> None` — body: `self.data[key] = value`.
2. `fetch(self, key: str) -> Any` — body: `return self.data.get(key)` (no explicit `None` default — `.get()` returns `None` by default, fixing Finding 5 simultaneously).

**Verify:** No free functions `store_item` or `fetch_item` exist. `DataStore` has `store` and `fetch` methods. No `.get(key, None)` pattern remains.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: move store_item/fetch_item into DataStore as methods"`.

## Task 8: Fix run_pipeline — mutable default and bare except

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Apply two changes to `run_pipeline`:

1. Change signature from `run_pipeline(items=[])` to `run_pipeline(items: list[str] | None = None)`. Add `if items is None: items = []` as the first line of the function body.
2. Replace bare `except: pass` with `except ValueError: continue`. This catches only the specific exception from `int()` conversion and skips the item.

**Verify:** No `items=[]` in any function signature. No bare `except:` anywhere in the file. `except ValueError` is present.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: fix mutable default arg and bare except in run_pipeline"`.

## Task 9: Replace __main__ block with Typer CLI

**Files:** `workgen/tests/fixtures/python_audit/src/service.py`

Replace the `if __name__ == "__main__"` block with a Typer app. Define `app = typer.Typer()` at module level (after the class/function definitions). Define a `@app.command()` function `main` that takes `config_path: str` as a positional argument. Inside, call `load_config(config_path)`, then `get_status(StatusCode.ACTIVE)`, then print the status. End the file with `if __name__ == "__main__": app()`.

**Verify:** `typer.Typer()` is used. No raw `sys.argv` access. `@app.command()` decorator present.

After completing implementation: run `just fmt`, then `git add workgen/tests/fixtures/python_audit/src/service.py`, then `git commit -m "chore: replace ad-hoc CLI with Typer app"`.

## Task 10: Final verification

Run the full verification suite to confirm all changes are correct and no regressions were introduced:

1. Run `just fmt` to ensure formatting is clean.
2. Run `just diagnose` to check for compilation errors and lint warnings.
3. Run `just test` to run the full test suite.

**Verify:** All three commands pass with zero errors and zero warnings.

After completing verification: run `git add -A && git commit -m "chore: verify all python audit fixes pass"` only if any files changed during formatting.

---

# CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "work_items/PYTHON_AUDIT_REMEDIATION.md" })
```

Then call `mcp__workflow__next_task` to get the first task and begin.