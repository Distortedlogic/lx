# Goal

Allow lowercase keyword tokens as record field names, dot-access field names, and module path segments. Currently `{par: 1}`, `r.par`, and `use std/match` fail because keyword tokens (`Par`, `Sel`, etc.) are not accepted where `Ident` is expected. Uppercase keywords like `Agent`, `Trait` already work in dot-access via `type_name()` at `expr_pratt.rs:69`.

# Why

- `{par: 1}` fails: `looks_like_record()` at `expr_helpers.rs:66` calls `ident()` which only matches `TokenKind::Ident`. `Par` is a keyword token, not Ident. The parser falls through to block parsing.
- `r.par` fails: `dot_rhs` at `expr_pratt.rs:68` calls `ident()` for named fields. Same issue.
- `use std/match` fails: `path_seg` at `stmt.rs:90-93` only matches `Ident` and special-cased `Yield`.

# Files affected

- `crates/lx/src/parser/expr.rs` lines 10-15 — Add `ident_or_keyword()` helper
- `crates/lx/src/parser/expr_helpers.rs` line 66 — Use it in `looks_like_record()`
- `crates/lx/src/parser/expr_helpers.rs` line 80 — Use it in `named_field`
- `crates/lx/src/parser/expr_pratt.rs` line 68 — Use it in `dot_rhs`
- `crates/lx/src/parser/stmt.rs` lines 90-93 — Use it in `path_seg`

# Task List

### Task 1: Add ident_or_keyword helper in expr.rs

In `crates/lx/src/parser/expr.rs`, add a new function directly after `ident()` at line 15. The function has the same signature as `ident()` but uses a `select!` matching `TokenKind::Ident(n) => n` plus every lowercase keyword variant from `crates/lx/src/lexer/helpers.rs:16-33`. The exact mappings (TokenKind variant → intern string): `Use` → "use", `Loop` → "loop", `Break` → "break", `Par` → "par", `Sel` → "sel", `Assert` → "assert", `Emit` → "emit", `Yield` → "yield", `With` → "with", `Timeout` → "timeout", `As` → "as". Do NOT include `True`/`False` (those are boolean literals, not field names). Do NOT include uppercase keyword variants (`AgentKw`, `Trait`, etc.) — those are already handled by `type_name()` in `dot_rhs`. Use `crate::sym::intern` to convert each string to Sym.

### Task 2: Use ident_or_keyword in record parsing

In `crates/lx/src/parser/expr_helpers.rs`, replace `ident()` with `super::expr::ident_or_keyword()` in two places: line 66 inside `looks_like_record()`, and line 80 inside the `named_field` parser. This ensures `{par: 1}` is recognized as a record by the lookahead AND parsed correctly as a named field.

### Task 3: Use ident_or_keyword in dot_rhs

In `crates/lx/src/parser/expr_pratt.rs`, replace `ident()` with `super::expr::ident_or_keyword()` at line 68 in the `named` parser inside `dot_rhs`. The `type_field` at line 69 stays as-is (it already handles uppercase keywords). The `choice` at line 81 tries `named` before `type_field`, so lowercase keywords are matched first.

### Task 4: Use ident_or_keyword in use path segments

In `crates/lx/src/parser/stmt.rs`, replace the `select!` at lines 90-93 with `super::expr::ident_or_keyword()`. This removes the special-cased `Yield` match since `ident_or_keyword()` already maps `Yield` → `intern("yield")`.

### Task 5: Add tests

Create `tests/parser_keyword_positions.lx`:

Record with keyword field names: `r = {par: 1; sel: 2; loop: 3; emit: 4}`. Assert `r.par == 1`, `r.sel == 2`, `r.loop == 3`, `r.emit == 4`.

Dot-access on keyword fields: `data = {yield: "hello"}`. Assert `data.yield == "hello"`.

Normal records still work (regression): `normal = {name: "test"; count: 5}`. Assert `normal.name == "test"` and `normal.count == 5`.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
