# Goal

Remediate fifteen audit violations in `workgen/tests/fixtures/rust_audit/src/main.rs` spanning a prohibited `#[allow(...)]` attribute, suboptimal parameter types, inline import paths, swallowed errors, self-assignments, an intermediate collect, string literals used where an enum should exist, magic number literals, a by-value Vec parameter that should be a slice, and an extracted function with a single call site. Each fix applies the single best idiomatic Rust practice for the violation.

# Why

- `#[allow(dead_code)]` hides an unused constant instead of removing it, violating codebase rules and masking dead code
- `&String` and `&Vec<String>` parameters force callers into unnecessary allocations and restrict what types can be passed
- `Vec<String>` taken by value in `transform` forces callers to heap-allocate when a slice reference would suffice
- Inline `std::path::PathBuf::from`, `std::fs::read_to_string`, and `std::fs::remove_file` at call sites violate the no-inline-import-paths rule and hurt readability
- `.unwrap()` on `read_to_string` panics on I/O failure instead of propagating the error to the caller
- `let _ = std::fs::remove_file(...)` silently discards a `Result`, violating the no-swallowed-errors rule
- Two `let x = x;` self-assignments are pointless rebindings that add noise
- An intermediate `.collect::<Vec<String>>()` followed by another `.collect::<Vec<&str>>()` allocates two temporary Vecs only to call `.join()` — one allocation suffices
- `get_status` returns raw string literals from a closed set of three values, losing exhaustiveness checking, typo safety, and zero-cost representation that an enum provides
- Magic number literals `200` and `404` are used as bare HTTP status codes without named constants or enum coverage
- `get_status` is a non-pub function called from exactly one site — its body should be inlined at the call site

# What changes

**Attribute and dead code removal** — Delete the `#[allow(dead_code)]` attribute and the unused `MAGIC` constant entirely.

**Self-assignment removal** — Delete `let input = input;` inside `transform` and `let result = result;` inside `main`. Both are no-op rebindings.

**Import additions and inline path fixes** — Add `use std::path::PathBuf;` and `use std::fs;` to the import block. Replace all fully-qualified `std::path::PathBuf::from(...)`, `std::fs::read_to_string(...)`, and `std::fs::remove_file(...)` call-site paths with their short names.

**Parameter type fixes** — Change the `process` function parameter from `&String` to `&str`. Change the `summarize` function parameter from `&Vec<String>` to `&[String]`. Change the `transform` function parameter from `Vec<String>` to `&[str]` and adjust its body to iterate over the borrowed slice instead of consuming a Vec. Update all call sites to pass the corrected types.

**Status enum definition** — Define a `Status` enum with variants `Ok`, `NotFound`, and `Error`, deriving `Debug`. Replace the string literal returns in `get_status` with enum variants and change its return type to `Status`. This also absorbs the magic number literal issue — the match arms on `200` and `404` become meaningful when paired with typed enum variants.

**Error propagation in process** — Change `process` to return `io::Result<HashMap<String, String>>`. Replace `.unwrap()` on `read_to_string` with the `?` operator. Wrap the return value in `Ok(...)`. Change `main` to return `io::Result<()>` and propagate the error from `process` with `?`. Add `Ok(())` at the end of main.

**Error handling for remove_file** — Since `main` now returns `io::Result<()>`, replace `let _ = fs::remove_file("temp.txt")` with `fs::remove_file("temp.txt")?` to propagate the error instead of swallowing it.

**Intermediate collect removal** — In `summarize`, remove both the intermediate `collected: Vec<String>` and `result: Vec<&str>` variables. Chain the iterator into a single expression: map to lowercase, collect into a single `Vec<String>`, and call `join` on it directly.

**Inline get_status at single call site** — Move the match expression from `get_status` directly into the single call site in `main`, bind the result to a local variable, use it in the `println!` with the `Debug` formatter, and delete the `get_status` function entirely.

# How it works

The file is a self-contained fixture with no external callers, so all changes are local. The key interaction is between error propagation and the `main` function: once `process` returns `Result`, `main` must return `Result` too (or explicitly handle errors). Making `main` return `io::Result<()>` allows both the `process` call and the `remove_file` call to propagate errors with `?`, which is the cleanest solution.

The `transform` parameter change from `Vec<String>` to `&[str]` shifts the function from consuming owned data to borrowing it. The body changes from `into_iter()` to `iter()`, and each element is now `&&str` which `to_uppercase` handles via deref. The call site changes from `transform(vec!["hello".into(), "world".into()])` to `transform(&["hello", "world"])`, eliminating the heap allocation.

Inlining `get_status` is safe because the function is non-pub and called exactly once. The Status enum remains defined at module scope for type safety, but the match expression moves into `main` directly.

# Files affected

- `workgen/tests/fixtures/rust_audit/src/main.rs` — All fifteen fixes apply to this single file: delete the allow attribute and dead constant, remove two self-assignments, add two use statements and fix three inline paths, change three function parameter types, define a Status enum, refactor two functions for proper error handling, change main to return Result, eliminate two intermediate collects, and inline the single-call-site function.

# Task List

## Task 1: Remove prohibited #[allow(dead_code)] and unused MAGIC constant

**Subject:** Remove #[allow(dead_code)] attribute and unused MAGIC constant
**ActiveForm:** Removing prohibited allow attribute and dead constant

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, delete line 3 (`#[allow(dead_code)]`) and line 4 (`const MAGIC: i32 = 42;`) entirely. Both lines are removed — the constant is unused anywhere in the file and the allow attribute is prohibited by codebase rules. After deletion, no `#[allow(` should appear in the file and no MAGIC identifier should exist.

---

## Task 2: Remove self-assignments

**Subject:** Delete pointless self-assignment rebindings
**ActiveForm:** Removing self-assignments

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, delete the line `let input = input;` inside the `transform` function and delete the line `let result = result;` inside `main`. Both are no-op rebindings. The original bindings are already usable without reassignment. After this task, no occurrence of the pattern `let x = x;` where both sides are the same identifier should remain.

---

## Task 3: Add use statements and fix inline import paths

**Subject:** Fix inline import paths with top-level use statements
**ActiveForm:** Adding use statements and replacing inline paths

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, add `use std::path::PathBuf;` and `use std::fs;` to the import block at the top of the file, alongside the existing `use std::collections::HashMap;`. In the `process` function, replace `std::path::PathBuf::from(data)` with `PathBuf::from(data)`. In the `process` function, replace `std::fs::read_to_string(&path)` with `fs::read_to_string(&path)`. In `main`, replace `std::fs::remove_file("temp.txt")` with `fs::remove_file("temp.txt")`. After this task, no occurrence of `std::path::PathBuf` or `std::fs::` should remain at any call site.

---

## Task 4: Fix parameter types on process, transform, and summarize

**Subject:** Change &String, Vec\<String>, and &Vec\<String> parameters to idiomatic types
**ActiveForm:** Changing parameter types to idiomatic Rust references

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, in the `process` function signature, change the parameter type from `data: &String` to `data: &str` — no body changes needed since PathBuf::from accepts &str. At the call site in `main`, change `process(&"config.txt".to_string())` to `process("config.txt")`. In the `transform` function signature, change the parameter from `input: Vec<String>` to `input: &[str]` and in the body change `input.into_iter()` to `input.iter()` — the `map(|s| s.to_uppercase())` and `collect()` remain unchanged since to_uppercase works on &&str via deref. At the call site in `main`, change `transform(vec!["hello".into(), "world".into()])` to `transform(&["hello", "world"])`. In the `summarize` function signature, change the parameter type from `items: &Vec<String>` to `items: &[String]` — no body or call-site changes needed since &Vec\<String> auto-derefs to &[String].

---

## Task 5: Define Status enum and refactor get_status return type

**Subject:** Replace string literals with Status enum in get_status
**ActiveForm:** Defining Status enum and refactoring get_status

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, define a new enum named Status above the `get_status` function with three variants: Ok, NotFound, and Error. Add `#[derive(Debug)]` on the enum. Change the return type of `get_status` from `&'static str` to Status. Replace the match arm returning "ok" with Status::Ok, the arm returning "not found" with Status::NotFound, and the arm returning "error" with Status::Error. In the println call in `main` that uses get_status(200), change the format specifier for that argument from `{}` to `{:?}` since Status derives Debug but does not implement Display. After this task, no string literals "ok", "not found", or "error" should remain in get_status.

---

## Task 6: Propagate errors in process and make main return Result

**Subject:** Change process to return Result and main to return io::Result
**ActiveForm:** Refactoring process and main for proper error propagation

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, add `use std::io;` to the import block at the top. Change the return type of `process` from `HashMap<String, String>` to `io::Result<HashMap<String, String>>`. Replace the `.unwrap()` on the `fs::read_to_string(&path)` call with the `?` operator. Wrap the final return expression in `Ok(...)`. Change the `main` function signature from `fn main()` to `fn main() -> io::Result<()>`. At the call site of `process` in `main`, append `?` to propagate the error. Add `Ok(())` as the last expression in `main`. After this task, no `.unwrap()` call should exist in process, and both process and main should return Result types.

---

## Task 7: Handle remove_file error instead of swallowing

**Subject:** Replace swallowed let _ = on remove_file with error propagation
**ActiveForm:** Fixing swallowed error on remove_file call

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, in `main`, replace `let _ = fs::remove_file("temp.txt")` with `fs::remove_file("temp.txt")?`. Since `main` now returns `io::Result<()>` from Task 6, the `?` operator propagates the error cleanly. After this task, no `let _ =` pattern should remain on any Result-returning expression in the file.

---

## Task 8: Remove intermediate collects in summarize

**Subject:** Eliminate unnecessary intermediate Vec allocations in summarize
**ActiveForm:** Removing intermediate collects in summarize

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, inside the `summarize` function, the current code creates `collected: Vec<String>` by mapping to lowercase and collecting, then creates `result: Vec<&str>` by mapping as_str and collecting again, then calls `result.join(", ")`. Replace the entire body with a single expression: call `items.iter()`, map each element to its lowercase form, collect into a single Vec\<String>, and call `.join(", ")` on that. This eliminates the `collected` variable and the `result` variable entirely, reducing two heap allocations to one. After this task, no variable named `collected` or `result` should exist inside `summarize`.

---

## Task 9: Inline get_status at its single call site and delete the function

**Subject:** Inline single-call-site get_status into main and delete the function
**ActiveForm:** Inlining get_status at call site and deleting function

**Description:** In `workgen/tests/fixtures/rust_audit/src/main.rs`, the `get_status` function is non-pub and called from exactly one location in `main`. In `main`, before the println call, add a let status binding with the match expression currently in `get_status` — matching 200 to Status::Ok, 404 to Status::NotFound, and the wildcard to Status::Error. Replace `get_status(200)` in the println with `status`. Delete the entire `get_status` function definition. After this task, no function named get_status should exist in the file. The match expression appears inline in main and the Status enum definition remains at module scope.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.