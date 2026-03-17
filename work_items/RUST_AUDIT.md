# Goal

Fix all Rust audit violations in `workgen/tests/fixtures/rust_audit/src/main.rs`. The file contains 11 violations spanning inline import paths, self-assignments, `#[allow(dead_code)]`, `&String`/`&Vec<T>` parameters, an intermediate `collect()`, verbose type annotations, string literals used instead of an enum, a swallowed error, and extracted functions with single call sites. All violations will be resolved by inlining function bodies, replacing anti-patterns with idiomatic Rust, and introducing an enum for status variants.

# Why

- `#[allow(dead_code)]` hides unused code instead of removing it — masks incomplete work
- `&String` and `&Vec<String>` parameters force callers to own heap-allocated types when borrowed slices suffice
- Inline import paths (`std::path::PathBuf::from`, `std::fs::read_to_string`, `std::fs::remove_file`) at call sites violate the short-name-at-call-site rule and reduce readability
- Two self-assignments (`let input = input;` and `let result = result;`) are pointless rebindings that add noise
- An intermediate `collect()` into `Vec<String>` followed by a second `collect()` into `Vec<&str>` wastes allocations when a single iterator chain suffices
- Explicit type annotations on `collected` and `result` are redundant where inference handles both
- String literals `"ok"`, `"not found"`, `"error"` representing a fixed status set lack exhaustiveness checking, typo prevention, and refactorability
- `let _ = std::fs::remove_file(...)` silently discards a `Result`, violating the no-swallowed-errors rule
- Four non-pub single-call-site functions (`process`, `transform`, `summarize`, `get_status`) add indirection without value — the file is 41 lines so inlining stays well under the 300-line limit

# What changes

**Enum introduction:**
- Define a `Status` enum with variants `Ok`, `NotFound`, `Error` and a `Display` impl to replace the string-returning `get_status` function

**Function inlining:**
- Inline the body of `process` at its single call site in `main`
- Inline the body of `transform` at its single call site in `main`, removing the self-assignment (`let input = input;`) during inlining
- Inline the body of `summarize` at its single call site in `main`, collapsing the double-collect into a single iterator chain and removing verbose type annotations
- Inline the status lookup (formerly `get_status`) at its single call site in `main`, returning a `Status` enum variant instead of a string
- Delete all four free functions after inlining

**Import cleanup:**
- Add `use std::path::PathBuf;` and `use std::fs;` at the top of the file
- Replace `std::path::PathBuf::from(...)` with `PathBuf::from(...)`
- Replace `std::fs::read_to_string(...)` with `fs::read_to_string(...)`
- Replace `std::fs::remove_file(...)` with `fs::remove_file(...)`

**Attribute removal:**
- Remove `#[allow(dead_code)]` from the `MAGIC` constant
- If `MAGIC` is unused after inlining (it is — nothing references it), delete the constant entirely

**Parameter type fixes (applied during inlining):**
- The `&String` parameter from `process` becomes `&str` at the inlined usage
- The `&Vec<String>` parameter from `summarize` becomes a direct slice operation at the inlined usage

**Self-assignment removal:**
- Remove `let input = input;` (was in `transform`, eliminated by inlining)
- Remove `let result = result;` in `main`

**Error handling:**
- Replace `let _ = fs::remove_file("temp.txt")` with explicit error handling — propagate via `?` by making `main` return `Result<(), Box<dyn std::error::Error>>`

**Iterator chain simplification (applied during inlining):**
- Replace the double-collect in `summarize` with `items.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(", ")`

# How it works

The four free functions are each called exactly once from `main`. Inlining them into `main` eliminates unnecessary indirection while the file remains well under 300 lines. During inlining, each function's violations (parameter types, self-assignments, verbose annotations, double-collect) are fixed at the point of integration rather than preserved.

The `Status` enum replaces the `&'static str` return of `get_status`. A `Display` impl on `Status` provides the same string output for `println!` formatting. The match expression is inlined directly into `main`.

Making `main` return `Result` allows the `fs::remove_file` error to propagate via `?` instead of being silently discarded.

# Files affected

- `workgen/tests/fixtures/rust_audit/src/main.rs` — all changes: remove `#[allow(dead_code)]` and `MAGIC` constant, add `use` statements for `std::path::PathBuf` and `std::fs`, define `Status` enum with `Display` impl, inline all four functions into `main`, fix parameter types during inlining, remove self-assignments, collapse double-collect, make `main` return `Result`, propagate `remove_file` error

# Task List

## Task 1: Define Status enum and Display impl

**Subject:** Define Status enum and Display impl

**Active form:** Defining Status enum and Display impl

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Add `use std::fmt;` to the imports at the top of the file
- Below the imports, define an enum `Status` with variants `Ok`, `NotFound`, `Error`
- Implement `fmt::Display` for `Status` mapping `Ok` to `"ok"`, `NotFound` to `"not found"`, `Error` to `"error"`

## Task 2: Add use statements and remove allow attribute and MAGIC constant

**Subject:** Clean up imports and remove dead code

**Active form:** Cleaning up imports and removing dead code

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Add `use std::path::PathBuf;` and `use std::fs;` to the import block at the top of the file
- Remove the `#[allow(dead_code)]` attribute on line 3 and the `const MAGIC: i32 = 42;` declaration on line 4 — the constant is unused

## Task 3: Inline all functions into main and fix all remaining violations

**Subject:** Inline functions into main and fix violations

**Active form:** Inlining functions into main and fixing violations

**File:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Delete the four free functions: `process`, `transform`, `summarize`, `get_status`
- Change `main` signature to `fn main() -> Result<(), Box<dyn std::error::Error>>` and add `Ok(())` at the end
- Inline the body of `process` into `main`: use `PathBuf::from("config.txt")` (not `&"config.txt".to_string()`) for the path, `fs::read_to_string(&path).unwrap()` for reading, and build the HashMap inline. This eliminates the `&String` parameter issue
- Remove the self-assignment `let result = result;` — it is the line immediately after the inlined process logic
- Replace `let _ = std::fs::remove_file("temp.txt")` with `fs::remove_file("temp.txt")?` to propagate the error
- Inline `transform`: write `let items: Vec<String> = vec!["hello".into(), "world".into()].into_iter().map(|s| s.to_uppercase()).collect();` — no self-assignment, no separate function
- Inline `summarize` as a single chain: `let summary = items.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(", ");` — this eliminates the `&Vec<String>` parameter, the double-collect, and the verbose type annotations
- Inline `get_status` as a match expression returning a `Status` variant: `let status = match 200 { 200 => Status::Ok, 404 => Status::NotFound, _ => Status::Error };`
- Update the `println!` to use the new `status` variable

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.