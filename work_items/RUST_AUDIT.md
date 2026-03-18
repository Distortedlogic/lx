# Goal

Fix all code quality violations identified by the Rust codebase audit in `workgen/tests/fixtures/rust_audit/src/main.rs`. The file contains 10 distinct violations spanning `#[allow(...)]` usage, suboptimal parameter types, inline import paths, self-assignments, intermediate collects, verbose type annotations, string literals instead of enums, and swallowed errors.

# Why

- `#[allow(dead_code)]` hides whether `MAGIC` is actually used, masking incomplete work
- `&String` and `&Vec<String>` parameters force callers to own a `String`/`Vec` unnecessarily and prevent passing `&str`/slices directly
- Inline import paths (`std::path::PathBuf`, `std::fs::read_to_string`, `std::fs::remove_file`) reduce readability and violate the project's code style rule
- Self-assignments (`let input = input;`, `let result = result;`) are pointless rebindings that add noise
- An intermediate `collect()` into `Vec<String>` followed by a second `collect()` into `Vec<&str>` wastes allocation when the result is immediately joined
- Explicit type annotations on lines 20–21 are redundant where inference works
- `get_status` returns `&'static str` from a fixed set of three values — an enum provides exhaustiveness checking, typo prevention, and zero-cost representation
- `let _ = std::fs::remove_file(...)` silently discards the `Result`, violating the no-swallowed-errors rule

# What changes

**Attribute removal:** Remove the `#[allow(dead_code)]` attribute from the `MAGIC` constant. The constant is unused — remove it entirely.

**Import hoisting:** Add `use std::path::PathBuf;` and `use std::fs;` at the top of the file. Replace `std::path::PathBuf::from(data)` with `PathBuf::from(data)`, `std::fs::read_to_string(&path)` with `fs::read_to_string(&path)`, and `std::fs::remove_file("temp.txt")` with `fs::remove_file("temp.txt")`.

**Parameter type fixes:** Change `process`'s `data` parameter from `&String` to `&str`. Change `summarize`'s `items` parameter from `&Vec<String>` to `&[String]`.

**Self-assignment removal:** Remove `let input = input;` from `transform`. Remove `let result = result;` from `main`.

**Intermediate collect elimination and type annotation removal:** Replace the two-step collect in `summarize` with a single chain: call `items.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(", ")` and return the result directly, removing the `collected` and `result` locals along with their explicit type annotations.

**Status enum introduction:** Define an enum `Status` with variants `Ok`, `NotFound`, and `Error`. Change `get_status` to return `Status` instead of `&'static str`. Derive `Debug` and implement `Display` on `Status` so the `println!` in `main` continues to work.

**Error handling:** Replace `let _ = fs::remove_file("temp.txt")` with explicit error handling — propagate or print the error. Since `main` currently returns unit, use `if let Err(e) = fs::remove_file("temp.txt") { eprintln!("{e}"); }`.

# Files affected

- `workgen/tests/fixtures/rust_audit/src/main.rs` — all 10 fixes applied to this single file

# Task List

## Task 1: Remove `#[allow(dead_code)]` and unused `MAGIC` constant

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

Remove lines 3–4 entirely (the `#[allow(dead_code)]` attribute and the `const MAGIC: i32 = 42;` declaration). The constant is unused anywhere in the file.

## Task 2: Hoist inline import paths to `use` statements

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

Add `use std::path::PathBuf;` and `use std::fs;` to the imports at the top of the file. Replace `std::path::PathBuf::from(data)` with `PathBuf::from(data)` in `process`. Replace `std::fs::read_to_string(&path)` with `fs::read_to_string(&path)` in `process`. Replace `std::fs::remove_file("temp.txt")` with `fs::remove_file("temp.txt")` in `main`.

## Task 3: Fix `&String` and `&Vec<String>` parameter types

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

Change the `data` parameter of `process` from `&String` to `&str`. Change the `items` parameter of `summarize` from `&Vec<String>` to `&[String]`.

## Task 4: Remove self-assignments

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

Remove `let input = input;` from the `transform` function body. Remove `let result = result;` from the `main` function body.

## Task 5: Eliminate intermediate collect and verbose type annotations in `summarize`

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

Replace the body of `summarize` with a single expression: `items.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(", ")`. This removes the `collected` and `result` locals along with their explicit type annotations.

## Task 6: Introduce `Status` enum to replace string literals in `get_status`

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

Define `enum Status { Ok, NotFound, Error }` with `#[derive(Debug)]`. Implement `std::fmt::Display` for `Status`, mapping `Ok` to `"ok"`, `NotFound` to `"not found"`, and `Error` to `"error"`. Change `get_status` return type from `&'static str` to `Status` and return the enum variants instead of string literals.

## Task 7: Handle swallowed error on `fs::remove_file`

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

Replace `let _ = fs::remove_file("temp.txt")` with `if let Err(e) = fs::remove_file("temp.txt") { eprintln!("{e}"); }`.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.