# Goal

Fix all 16 audit violations in `workgen/tests/fixtures/rust_audit/src/main.rs`. The file is a test fixture containing intentional Rust anti-patterns: `#[allow(dead_code)]` on an unused constant, `&String` and `&Vec<T>` parameters, inline import paths, swallowed errors, self-assignments, intermediate collects, verbose type annotations, string literals used as enums, and dead code. Every violation must be resolved to bring the fixture into full compliance with the codebase audit checklist.

# Why

- The file contains an `#[allow(dead_code)]` attribute that masks an unused constant, violating the no-allow-macros rule
- `&String` and `&Vec<String>` parameters force callers to own a `String`/`Vec` unnecessarily; `&str` and `&[String]` are idiomatic
- Three inline import paths (`std::path::PathBuf`, `std::fs::read_to_string`, `std::fs::remove_file`) bypass the no-inline-imports rule
- Two swallowed errors (`.unwrap()` on IO and `let _ =` on file removal) hide failures from callers
- Two self-assignments (`let input = input;`, `let result = result;`) are pointless rebindings
- Two intermediate `collect()` calls allocate a `Vec` only to iterate it again immediately
- Explicit type annotations on two locals where inference suffices add visual noise
- `get_status` returns string literals from a fixed set of variants that should be an enum for exhaustiveness and type safety

# What changes

**Imports:** Add `use std::path::PathBuf;` and `use std::fs;` at the top of the file. Remove all inline `std::path::PathBuf`, `std::fs::read_to_string`, and `std::fs::remove_file` usages, replacing with short names `PathBuf`, `fs::read_to_string`, `fs::remove_file`.

**Dead code removal:** Delete the `#[allow(dead_code)]` attribute and the `MAGIC` constant entirely.

**`process` function:** Change parameter from `data: &String` to `data: &str`. Change return type to `Result<HashMap<String, String>, std::io::Error>`. Replace `.unwrap()` with `?`.

**`transform` function:** Remove the `let input = input;` self-assignment. Use the `input` parameter directly.

**`summarize` function:** Change parameter from `items: &Vec<String>` to `items: &[String]`. Eliminate the intermediate `collected` and `result` variables. Chain the entire operation: `items.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(", ")`. Remove explicit type annotations.

**`get_status` function:** Define a `Status` enum with variants `Ok`, `NotFound`, `Error`. Change return type from `&'static str` to `Status`. Return enum variants instead of string literals. Derive `Debug` on `Status` so it can be printed.

**`main` function:** Remove the `let result = result;` self-assignment. Handle the `process` call's `Result` with `.expect()` or propagate (since `main` can return `Result`). Change `main` signature to `fn main() -> Result<(), Box<dyn std::error::Error>>` to support error propagation. Replace `let _ = std::fs::remove_file(...)` with explicit error handling — use `if let Err(e) = fs::remove_file(...)` and match on `ErrorKind::NotFound` to ignore only that case, propagating other errors. Use short import names at all call sites. Update the `println!` format to use `{:?}` for the `Status` enum value.

# Files affected

- `workgen/tests/fixtures/rust_audit/src/main.rs` — all changes described above apply to this single file

# Task List

## Task 1: Remove dead code and `#[allow]` attribute, add missing imports

**Subject:** Remove dead code and add missing use statements

**Files:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Delete lines 3–4 (the `#[allow(dead_code)]` attribute and `const MAGIC: i32 = 42;`)
- Add `use std::path::PathBuf;` after the existing `use std::collections::HashMap;` line
- Add `use std::fs;` after the PathBuf import
- Add `use std::io;` after the fs import (needed for error handling in later tasks)

**Verification:** File compiles with `just diagnose` (warnings about unused imports are acceptable at this stage since later tasks will consume them).

**After completing implementation:** Run `just fmt`, then `git add workgen/tests/fixtures/rust_audit/src/main.rs`, then `git commit -m "Remove dead code and #[allow] attr, add missing imports"`.

---

## Task 2: Fix `process` function — `&String` param, inline paths, swallowed error

**Subject:** Fix process function signature and error handling

**Files:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Change `process` parameter from `data: &String` to `data: &str`
- Change return type from `HashMap<String, String>` to `io::Result<HashMap<String, String>>`
- Replace `std::path::PathBuf::from(data)` with `PathBuf::from(data)`
- Replace `std::fs::read_to_string(&path).unwrap()` with `fs::read_to_string(&path)?`
- Add `Ok(map)` as the final expression instead of bare `map`

**Verification:** The function now propagates IO errors instead of panicking.

**After completing implementation:** Run `just fmt`, then `git add workgen/tests/fixtures/rust_audit/src/main.rs`, then `git commit -m "Fix process fn: &str param, short imports, error propagation"`.

---

## Task 3: Fix `transform` function — remove self-assignment

**Subject:** Remove self-assignment in transform function

**Files:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Delete the line `let input = input;` inside the `transform` function body
- The next line `input.into_iter().map(|s| s.to_uppercase()).collect()` uses the parameter directly — no other changes needed

**Verification:** Function behaves identically without the redundant rebinding.

**After completing implementation:** Run `just fmt`, then `git add workgen/tests/fixtures/rust_audit/src/main.rs`, then `git commit -m "Remove self-assignment in transform function"`.

---

## Task 4: Fix `summarize` function — `&Vec` param, intermediate collects, verbose annotations

**Subject:** Fix summarize function parameter and eliminate intermediate collects

**Files:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Change parameter from `items: &Vec<String>` to `items: &[String]`
- Replace the entire function body (lines 20–22) with a single expression: `items.iter().map(|s| s.to_lowercase()).collect::<Vec<_>>().join(", ")`
- This eliminates both intermediate `collect()` calls, removes the explicit type annotations on `collected` and `result`, and removes the unnecessary `as_str()` mapping

**Verification:** The function returns the same comma-separated lowercase string as before.

**After completing implementation:** Run `just fmt`, then `git add workgen/tests/fixtures/rust_audit/src/main.rs`, then `git commit -m "Fix summarize: slice param, eliminate intermediate collects"`.

---

## Task 5: Replace `get_status` string returns with a `Status` enum

**Subject:** Define Status enum and replace string literals in get_status

**Files:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Define a new enum above the `get_status` function with `#[derive(Debug)]`: `enum Status { Ok, NotFound, Error }`
- Change `get_status` return type from `&'static str` to `Status`
- Replace the match arms: `200 => Status::Ok`, `404 => Status::NotFound`, `_ => Status::Error`

**Verification:** The enum provides exhaustiveness checking and eliminates string-based dispatch.

**After completing implementation:** Run `just fmt`, then `git add workgen/tests/fixtures/rust_audit/src/main.rs`, then `git commit -m "Replace string returns with Status enum in get_status"`.

---

## Task 6: Fix `main` function — self-assignment, swallowed error, inline path, error propagation

**Subject:** Fix main function error handling and cleanup

**Files:** `workgen/tests/fixtures/rust_audit/src/main.rs`

**Changes:**
- Change `main` signature to `fn main() -> Result<(), Box<dyn std::error::Error>>`
- Change `let result = process(&"config.txt".to_string());` to `let result = process("config.txt")?;` — this passes a `&str` directly (no `.to_string()` needed since param is now `&str`) and propagates the error
- Delete the `let result = result;` self-assignment line
- Replace `let _ = std::fs::remove_file("temp.txt");` with: `if let Err(e) = fs::remove_file("temp.txt") { if e.kind() != io::ErrorKind::NotFound { return Err(e.into()); } }` — this uses the short import name, explicitly handles the error by ignoring only "not found" and propagating all other IO errors
- Update the `println!` to use `{:?}` for the `Status` value (it already uses `{:?}` for result and `Status` derives `Debug`): `println!("{:?} {} {:?}", result, summary, get_status(200));`
- Add `Ok(())` as the final expression of main

**Verification:** `main` now propagates all errors and handles file-not-found explicitly.

**After completing implementation:** Run `just fmt`, then `git add workgen/tests/fixtures/rust_audit/src/main.rs`, then `git commit -m "Fix main: error propagation, remove self-assignment and swallowed error"`.

---

## Task 7: Final verification

**Subject:** Run full verification suite

**Changes:** None — this is verification only.

**Steps:**
1. Run `just fmt` and confirm no formatting changes
2. Run `just diagnose` and confirm zero warnings, zero errors
3. Run `just test` and confirm all tests pass

**After completing verification:** Run `git add workgen/tests/fixtures/rust_audit/src/main.rs`, then `git commit -m "Verify: all rust audit violations resolved in test fixture"` (only if any formatting changes were made by `just fmt`).

---

# CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Run `just fmt`, `git add`, `git commit` after each task.** Format, stage, and commit after every task completes.
2. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
3. **Tasks are implementation-only.** The final task handles verification.
4. **No code comments or doc strings.** Do not add comments to the code.
5. **No `#[allow(...)]` macros.** Do not suppress any warnings.

---

# Task Loading Instructions

To begin executing this work item, run:

```
mcp__workflow__load_work_item({ path: "work_items/RUST_AUDIT_FIXTURE_FIXES.md" })
```

Then call `mcp__workflow__next_task` to get the first task and begin implementation.