# Goal

Fix code quality violations found by the Rust audit: inline import paths, self-assignments, #[allow] attributes, &String/&Vec parameters, swallowed errors, intermediate collects, and string literals used as enum-like values.

# Why

The codebase has multiple quality issues: fully qualified paths at call sites instead of `use` statements, redundant `let x = x` self-assignments, `#[allow(dead_code)]` hiding warnings, `&String` and `&Vec<T>` parameters instead of `&str` and `&[T]`, a swallowed error via `let _ =`, an unnecessary intermediate `collect()` into Vec that is immediately iterated again, and string literals like "ok", "not found", "error" representing a fixed set of status values that should be an enum.

# What changes

- Add `use std::path::PathBuf` and `use std::fs` imports, replace inline import paths
- Remove self-assignment `let input = input` and `let result = result`
- Remove `#[allow(dead_code)]` — either use the constant or remove it
- Change `&String` parameter to `&str` in process function
- Change `&Vec<String>` parameter to `&[String]` in summarize function
- Replace `let _ = std::fs::remove_file(...)` with proper error handling, do not swallow errors
- Remove intermediate collect in summarize — chain iterators directly
- Replace string literals "ok", "not found", "error" with a Status enum

# Files affected

- src/main.rs — inline import paths, self-assignments, #[allow], &String parameter, &Vec parameter, swallowed error, intermediate collect, string literals as enum

# Task List

## Task 1: Fix inline imports

Add `use std::path::PathBuf` and `use std::fs` at file top. Replace `std::path::PathBuf::from` and `std::fs::read_to_string` with short names.

```
just fmt
git add src/main.rs
git commit -m "fix: replace inline import paths with use statements"
```

## Task 2: Remove self-assignments and #[allow]

Remove `let input = input` — make function param `mut input`. Remove `let result = result`. Remove `#[allow(dead_code)]` — either use MAGIC or delete it.

```
just fmt
git add src/main.rs
git commit -m "fix: remove self-assignments and #[allow] attribute"
```

## Task 3: Fix parameter types

Change `process(data: &String)` to `process(data: &str)`. Change `summarize(items: &Vec<String>)` to `summarize(items: &[String])`.

```
just fmt
git add src/main.rs
git commit -m "fix: use &str and &[T] instead of &String and &Vec"
```

## Task 4: Fix swallowed error and intermediate collect

Replace `let _ = std::fs::remove_file(...)` with `?` propagation or explicit handling. In summarize, remove the intermediate collect — chain the iterator directly.

```
just fmt
git add src/main.rs
git commit -m "fix: propagate error, remove intermediate collect"
```

## Task 5: Replace string literals with enum

Define a Status enum with Ok, NotFound, Error variants. Replace string literals in get_status with enum variants.

```
just fmt
git add src/main.rs
git commit -m "fix: replace status string literals with enum"
```

## Task 6: Verification

```
just test
just diagnose
just fmt
git add -A
git commit -m "chore: verify rust audit fixes"
```

# CRITICAL REMINDERS

- Run `just fmt` after every file change
- Run `just test` and `just diagnose` before final commit
- No inline import paths at call sites
- No self-assignments
- No #[allow] attributes
- No &String or &Vec parameters

# Task Loading Instructions

Load these instructions by reading this file, then execute each task in order. After each task, run `just fmt` and commit.
