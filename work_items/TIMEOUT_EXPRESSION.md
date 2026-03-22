# Goal

Add `timeout` as a first-class expression wrapper so any expression can be time-bounded. Currently there is no language-level timeout mechanism — the only option is to manually race expressions inside a `sel` block, which is verbose for the common case of "run this one thing with a time limit."

# Why

- Every agentic workflow has operations that can hang — LLM calls, HTTP requests, agent asks. Timeout is the most common `sel` usage pattern but requires 3 lines of boilerplate for one expression.
- The research in `research/concurrency/design-patterns.md` covers structured concurrency (Trio, Kotlin) where timeout wraps expressions directly.
- Adding `timeout` as a keyword-level expression makes timeout a first-class language feature instead of requiring verbose `sel` blocks.

**Note:** `timeout` currently exists as a builtin function in `crates/lx/src/builtins/convert.rs` that performs a blocking `std::thread::sleep`. Promoting `timeout` to a keyword will shadow that builtin; the old sleep-based `timeout` should be removed or renamed (e.g., `sleep`) as part of this work.

# What Changes

## New expression: `timeout MILLIS EXPR`

When used outside a `sel` block, `timeout millis expr` evaluates `expr` with a time limit. If `expr` completes within `millis`, returns `Ok result`. If it exceeds the limit, returns `Err {kind: :timeout, ms: millis}`.

```
result = timeout 5000 (http.get url)
result ? {
  Ok data -> process data
  Err {kind: :timeout ..} -> fallback
}
```

This desugars internally to the existing `sel` machinery — wrapping the expression and a timeout arm.

## Lexer changes

Add `Timeout` to `TokenKind` in `crates/lx/src/lexer/token.rs`. Add `"timeout" => TokenKind::Timeout` to the `ident_or_keyword` function in `crates/lx/src/lexer/helpers.rs`.

## Parser changes

In `crates/lx/src/parser/expr.rs`, inside `expr_parser()`, add a `timeout_expr` combinator (similar to `emit_expr` or `assert_expr`): consume `TokenKind::Timeout`, parse the milliseconds expression, parse the body expression, and produce `Expr::Timeout { ms: SExpr, body: SExpr }`. Add it to the `atom` choice list.

## AST changes

Add `Timeout { ms: Box<SExpr>, body: Box<SExpr> }` variant to the `Expr` enum in `crates/lx/src/ast/mod.rs`.

## Interpreter changes

In the interpreter, `Timeout` evaluates as: spawn `body` as a future, race it against `tokio::time::sleep(Duration::from_millis(ms))`. If body wins, return `Ok(result)`. If timer wins, return `Err({kind: :timeout, ms: ms_val})`.

## Checker changes

Add a match arm for `Timeout` in the checker that synthesizes `Result { ok: body_type, err: Record }`.

# Files Affected

**Modified files:**
- `crates/lx/src/lexer/token.rs` — add `Timeout` to `TokenKind`
- `crates/lx/src/lexer/helpers.rs` — map `"timeout"` to `TokenKind::Timeout` in `ident_or_keyword`
- `crates/lx/src/ast/mod.rs` — add `Timeout` variant to `Expr`
- `crates/lx/src/parser/expr.rs` — parse `timeout MILLIS EXPR` as expression atom
- `crates/lx/src/interpreter/mod.rs` — add `Expr::Timeout` match arm in `eval()`
- `crates/lx/src/interpreter/eval.rs` — implement `eval_timeout` method
- `crates/lx/src/checker/synth.rs` — synthesize type for `Timeout`
- `crates/lx/src/visitor/walk/mod.rs` — add `Expr::Timeout` dispatch in `walk_expr`
- `crates/lx/src/visitor/walk/walk_expr.rs` — add `walk_timeout` function
- `crates/lx/src/visitor/mod.rs` — add `visit_timeout` method to `AstVisitor` trait
- `crates/lx/src/builtins/convert.rs` — remove or rename the old `timeout` builtin (now a keyword)

**New files:**
- `tests/80_timeout_expr.lx` — tests for timeout as expression

# Task List

### Task 1: Add Timeout token, AST variant, and parser support

**Subject:** Parse `timeout MILLIS EXPR` as a standalone expression

**Description:**

1. Add `Timeout` to `TokenKind` in `crates/lx/src/lexer/token.rs` (alongside `Emit`, `Yield`, etc.).
2. Add `"timeout" => TokenKind::Timeout` to `ident_or_keyword` in `crates/lx/src/lexer/helpers.rs`.
3. Add `Timeout { ms: Box<SExpr>, body: Box<SExpr> }` to the `Expr` enum in `crates/lx/src/ast/mod.rs`.
4. In `crates/lx/src/parser/expr.rs`, inside `expr_parser()`, add a `timeout_expr` combinator following the pattern of `assert_expr` (which also takes two sub-expressions): consume `TokenKind::Timeout`, then parse `ms` expression, then parse `body` expression, producing `Expr::Timeout { ms: Box::new(ms), body: Box::new(body) }`. Add `timeout_expr` to the `atom` choice list.
5. Update the visitor: add `walk_timeout` in `crates/lx/src/visitor/walk/walk_expr.rs`, add `Expr::Timeout` dispatch in `walk_expr` in `crates/lx/src/visitor/walk/mod.rs`, and add `visit_timeout` to the `AstVisitor` trait in `crates/lx/src/visitor/mod.rs`.
6. Remove or rename the old `timeout` builtin in `crates/lx/src/builtins/convert.rs` (rename to `sleep` since it does blocking sleep, not async timeout).

Run `just diagnose`.

**ActiveForm:** Adding Timeout token, AST variant, and parser

---

### Task 2: Implement Timeout evaluation in interpreter

**Subject:** Evaluate timeout expression via tokio select

**Description:** Add `Expr::Timeout { ms, body }` to the eval match in `crates/lx/src/interpreter/mod.rs` (in the `eval()` method, around line 175 where `Sel` is handled), dispatching to a new `eval_timeout` method. Implement `eval_timeout` in `crates/lx/src/interpreter/eval.rs`:

1. Evaluate `ms` to get the timeout duration in milliseconds (must be Int or Float).
2. Spawn `body` evaluation as a future.
3. Use `tokio::select!` to race the body future against `tokio::time::sleep(Duration::from_millis(ms_val))`.
4. If body completes first: return `Ok(LxVal::ok(result))`.
5. If timer fires first: return `Ok(LxVal::err(LxVal::Record(indexmap!{"kind" => LxVal::str(":timeout"), "ms" => LxVal::Int(ms_val)})))`.

Run `just diagnose`.

**ActiveForm:** Implementing timeout evaluation

---

### Task 3: Add checker support and write tests

**Subject:** Type-check Timeout expression and create test suite

**Description:** In `crates/lx/src/checker/synth.rs`, add a match arm for `Expr::Timeout`: synthesize `ms` (expect Int or Float), synthesize `body`, return `Type::Result { ok: body_type, err: Type::Record(...) }`.

Create `tests/80_timeout_expr.lx` with tests:
1. `timeout 1000 (42)` — should return `Ok 42` (instant completion).
2. `timeout 1 (loop {})` — should return `Err` with kind `:timeout` (loop never completes).
3. Verify the timeout error record has `kind` and `ms` fields.
4. Compose with `?` operator: `timeout 1000 (42) ?` should unwrap to 42.

Run `just diagnose` and `just test`.

**ActiveForm:** Adding checker support and writing timeout tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/TIMEOUT_EXPRESSION.md" })
```

Then call `next_task` to begin.
