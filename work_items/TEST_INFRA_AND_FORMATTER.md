# Goal

Make `just test` actually run the `.lx` test suite, add a Rust integration test harness that invokes the lx CLI on test files programmatically, fix the formatter's list/tuple separators from `"; "` to `" "` to match the language syntax, and add a formatter round-trip test that verifies `format(parse(source))` produces valid lx.

# Why

- `just test` runs `cargo test --workspace` which finds zero `#[test]` functions in the entire lx codebase. The 7 `.lx` test files in `tests/` and ~16 in `programs/*/tests/` exist but nothing executes them from the justfile. The "71/71 tests passing" claim in docs is unverifiable
- The formatter emits `"; "` as separator in lists, tuples, records, and maps (e.g., `[1; 2; 3]`) but the language syntax uses spaces (`[1 2 3]`). An LLM agent that formats its output gets different syntax than what it wrote. `format(parse(source))` does not reproduce the original source
- There are zero Rust-level tests for any lx component (parser, checker, interpreter, formatter). A regression in any component is only caught if someone manually runs a `.lx` file

# What changes

**Justfile:** The `test` recipe changes from `cargo test --workspace` to running both `cargo test` (for any future Rust tests) AND `cargo run -p lx-cli -- test` (for `.lx` suite tests).

**Rust integration tests:** A new `crates/lx-cli/tests/suite.rs` file adds a `#[test]` function that invokes the lx CLI binary on the test suite, so `cargo test` itself catches `.lx` test regressions.

**Formatter:** All `"; "` separators in list, tuple, record, and map emission change to `" "` (space). This matches the language's space-separated syntax.

**Round-trip test:** A new `#[test]` function verifies that for each `.lx` file, `format(parse(source))` produces output that re-parses without errors.

# Files affected

| File | Change |
|------|--------|
| `justfile` | Change `test` recipe to run both `cargo test` and `cargo run -p lx-cli -- test` |
| `crates/lx-cli/tests/suite.rs` | New file — Rust integration test that runs `.lx` test suite |
| `crates/lx/src/formatter/emit_expr.rs` | Change `"; "` to `" "` for list (line 117), tuple (line 106), record (line 134), map (line 155) separators |
| `crates/lx/src/formatter/emit_expr_helpers.rs` | Change `"; "` to `" "` for WithKind::Resources (line 188) and WithKind::Context (line 199) separators |
| `crates/lx/src/formatter/emit_stmt.rs` | Change `"; "` to `" "` for type params (line 221), class traits (line 152), use selective imports (line 207) separators |
| `crates/lx/src/formatter/emit_type.rs` | Change `"; "` to `" "` for type record fields (line 34) separator |

# Task List

### Task 1: Fix the justfile test recipe

In the `justfile` at the repo root, replace the `test` recipe (lines 25-28):

From:
```
test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test --workspace --exclude inference-server --all-targets --all-features -q 2>&1
```

To:
```
test:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo test --workspace --exclude inference-server --all-targets --all-features -q 2>&1
    cargo run -p lx-cli -- test
```

This runs Rust tests first, then the `.lx` suite tests via the CLI. The CLI test runner (`testing.rs`) discovers `.lx` files from workspace members defined in `lx.toml` (members: tests, brain, workgen, flows, pkg, workrunner), executes each via the interpreter, and reports pass/fail based on `assert` outcomes. A test passes if all asserts succeed and no runtime error occurs. A test fails if any assert fails or an unhandled error propagates.

### Task 2: Fix formatter list separator

In `crates/lx/src/formatter/emit_expr.rs`, change the list element separator at line 117:

From:
```rust
self.write("; ");
```

To:
```rust
self.write(" ");
```

This is inside the list emission loop, between elements. The language syntax is `[1 2 3]` (space-separated), not `[1; 2; 3]`.

### Task 3: Fix formatter tuple separator

In the same file `crates/lx/src/formatter/emit_expr.rs`, change the tuple element separator at line 106:

From:
```rust
self.write("; ");
```

To:
```rust
self.write(" ");
```

Tuple syntax is `(a b c)`, not `(a; b; c)`.

### Task 4: Fix formatter record separator

In `crates/lx/src/formatter/emit_expr.rs`, change the record field separator at line 134:

From:
```rust
self.write("; ");
```

To:
```rust
self.write(" ");
```

Record syntax is `{x: 1 y: 2}`, not `{x: 1; y: 2}`. Note: semicolons ARE used as statement separators in blocks, but records are not blocks — they use space separation.

### Task 5: Fix formatter map separator

In `crates/lx/src/formatter/emit_expr.rs`, change the map entry separator at line 155:

From:
```rust
self.write("; ");
```

To:
```rust
self.write(" ");
```

Map syntax is `%{"a": 1 "b": 2}`, not `%{"a": 1; "b": 2}`.

### Task 6: Fix remaining separator sites

In `crates/lx/src/formatter/emit_expr_helpers.rs`:
- Line 188 (WithKind::Resources): change `"; "` to `" "`
- Line 199 (WithKind::Context): change `"; "` to `" "`

In `crates/lx/src/formatter/emit_stmt.rs`:
- Line 152 (class traits list): change `"; "` to `" "`
- Line 207 (use selective imports): change `"; "` to `" "`
- Line 221 (type params): change `"; "` to `" "`

In `crates/lx/src/formatter/emit_type.rs`:
- Line 34 (type record fields): change `"; "` to `" "`

All of these are data structure element separators, not statement separators. They all use space separation in the language syntax.

### Task 7: Add Rust integration test harness

Create `crates/lx-cli/tests/suite.rs`:

```rust
use std::process::Command;

#[test]
fn lx_test_suite() {
    let output = Command::new(env!("CARGO_BIN_EXE_lx"))
        .args(["test"])
        .current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/../..")
        .output()
        .expect("failed to execute lx test");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!(
            "lx test suite failed (exit code {:?}):\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            stdout,
            stderr
        );
    }
}
```

The binary is named `lx` (not `lx-cli`) per `crates/lx-cli/Cargo.toml` line 8: `[[bin]] name = "lx"`. The `env!("CARGO_BIN_EXE_lx")` macro resolves to the compiled binary path. The test passes if the CLI exits with code 0 (all `.lx` tests passed), fails otherwise.

### Task 8: Add formatter round-trip test

Create `crates/lx/tests/formatter_roundtrip.rs`:

The exact public API (verified from source):
- `lx::lexer::lex(source: &str) -> Result<(Vec<Token>, CommentStore), LxError>` (at `lexer/mod.rs:24`)
- `lx::parser::parse(tokens: Vec<Token>, file: FileId, comments: CommentStore, source: &str) -> ParseResult` (at `parser/mod.rs:58`), where `ParseResult.program` is `Option<Program<Surface>>`
- `lx::formatter::format<P>(program: &Program<P>) -> String` (at `formatter/mod.rs:54`)

```rust
use std::fs;
use lx::lexer::lex;
use lx::parser::parse;
use lx::formatter::format;
use lx::source::FileId;

fn roundtrip_check(path: &str) {
    let source = fs::read_to_string(path).unwrap();
    let (tokens, comments) = lex(&source).expect("lex failed");
    let result = parse(tokens, FileId::new(0), comments, &source);
    let program = result.program.expect("parse failed");
    let formatted = format(&program);

    let (tokens2, comments2) = lex(&formatted).expect("re-lex failed");
    let result2 = parse(tokens2, FileId::new(0), comments2, &formatted);
    assert!(
        result2.program.is_some(),
        "round-trip failed for {}: formatted output does not re-parse.\nFormatted:\n{}",
        path,
        formatted
    );
}

#[test]
fn formatter_roundtrips_test_files() {
    for entry in fs::read_dir("../../tests").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().map(|e| e == "lx").unwrap_or(false) {
            roundtrip_check(path.to_str().unwrap());
        }
    }
}
```

This test verifies that formatted output can be re-parsed without errors. It does NOT assert the formatted output equals the original source (the formatter may normalize whitespace, indentation, etc.), only that it produces valid lx.

### Task 9: Compile, format, and verify

Run `just fmt` to format all changed files.

Run `just test` using the NEW recipe. This now runs:
1. `cargo test --workspace` — which runs the new Rust integration test (Task 7) and round-trip test (Task 8)
2. `cargo run -p lx-cli -- test` — which runs the `.lx` test suite

Fix any failures. Common issues:
- The binary name in `env!("CARGO_BIN_EXE_...")` must match Cargo.toml exactly
- The round-trip test may fail on files that use syntax the formatter doesn't handle identically — investigate each failure
- The `.lx` test suite may have pre-existing failures — document them but don't block on them

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Do not add, skip, reorder, or combine tasks.**
3. **The formatter separator change is `"; "` → `" "` (space, not empty string).** Do not remove the separator entirely.
4. **The binary is named `lx`** (not `lx-cli`). The `[[bin]]` section in `crates/lx-cli/Cargo.toml` line 8 declares `name = "lx"`. Use `env!("CARGO_BIN_EXE_lx")`.
5. **Statement separators remain semicolons.** Only change data structure element separators (lists, tuples, records, maps, type params, etc.). Block statements continue to use `; ` or newlines.
6. **The round-trip test only checks re-parsability**, not exact source reproduction. Formatting normalizes whitespace and indentation by design.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/TEST_INFRA_AND_FORMATTER.md" })
```

Then call `next_task` to begin.
