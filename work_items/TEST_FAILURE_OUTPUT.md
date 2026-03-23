# Goal

Make `lx test` assertion failures show expected and actual values instead of an AST debug dump. When `assert (result == [1, 2, 3])` fails, the agent should see `expected: [1, 2, 3]` and `actual: [1, 2, "three"]`, not `Binary(ExprBinary { op: Eq, left: ExprId(42), right: ExprId(43) })`.

# Why

LLM agents write lx tests and iterate on failing assertions. Currently, `eval_assert` in the interpreter constructs the error with `format!("{expr_node:?}")` — the Rust Debug format of the AST node. This is useless to an agent trying to figure out what went wrong. The agent needs to see the actual runtime values to diagnose the problem.

The `LxVal` type already has `short_display()` (truncates to 80 chars) and a full `Display` impl. The assertion evaluation already evaluates the whole expression to a boolean — the change is to evaluate the two sides of a comparison separately and capture their values before comparing.

# What changes

**Modified `crates/lx/src/error.rs`:** Extend `LxError::Assert` with optional `expected` and `actual` string fields. Update the miette error format to render them.

**Modified `crates/lx/src/interpreter/eval.rs`:** In `eval_assert`, detect when the assert expression is a binary comparison (`Eq`, `NotEq`, `Lt`, `Gt`, `Lte`, `Gte`). If so, evaluate both sides separately, capture their display strings, then perform the comparison. On failure, pass the captured values to `assert_fail`.

# Files affected

- EDIT: `crates/lx/src/error.rs` — extend Assert variant with expected/actual fields
- EDIT: `crates/lx/src/interpreter/eval.rs` — detect comparison in assert, capture values

# Task List

### Task 1: Extend LxError::Assert with expected and actual fields

**Subject:** Add expected/actual value display to assertion error type

**Description:** In `crates/lx/src/error.rs`, modify the `Assert` variant of `LxError`:

Current shape (approximately):
```rust
Assert {
    expr: String,
    message: Option<String>,
    span: SourceSpan,
}
```

Change to:
```rust
Assert {
    expr: String,
    message: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    #[label("assertion failed")]
    span: SourceSpan,
}
```

Update the `#[error]` format string. The miette `#[error]` attribute controls the top-line message. Change it from `"assertion failed: {expr}"` to a dynamic format that includes expected/actual when present. Since miette's `#[error]` might not support conditional formatting, implement it as:
- `#[error("assertion failed: {expr}")]` — keep the top line simple
- Add a `#[help]` attribute that renders the expected/actual detail. Compute the help string dynamically:
  - If both `expected` and `actual` are Some: `"expected: {expected}\n  actual: {actual}"`
  - If `message` is Some: include that too
  - If neither: no help

Alternatively, if miette supports `#[diagnostic(help(...))]` with a method, implement `fn help_text(&self) -> Option<String>` and use that.

Update the `assert_fail` constructor:
```rust
pub fn assert_fail(
    expr: impl Into<String>,
    message: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    span: SourceSpan,
) -> Self
```

Find all call sites of `assert_fail` and update them. There should be at least:
- `interpreter/eval.rs` `eval_assert` — this is the main one (updated in Task 2)
- Any other places that construct `LxError::Assert` directly

For existing call sites that don't have expected/actual info, pass `None, None` for the new fields.

**ActiveForm:** Extending Assert error with expected/actual fields

### Task 2: Capture comparison values in eval_assert

**Subject:** Evaluate both sides of comparison assertions and capture values on failure

**Description:** In `crates/lx/src/interpreter/eval.rs`, modify `eval_assert` (around line 222):

Current flow:
1. Evaluate the whole assert expression to a value
2. Check if it's `true` or `false`
3. On false: construct error with AST debug dump

New flow:
1. Look at the assert expression AST node (via `self.arena.expr(expr)`)
2. If it's `Expr::Binary { op, left, right }` where `op` is `Eq`, `NotEq`, `Lt`, `Gt`, `Lte`, or `Gte`:
   a. Evaluate `left` → `left_val`
   b. Evaluate `right` → `right_val`
   c. Perform the comparison (evaluate the original binary expression, or compare the values directly)
   d. If the comparison is false (assertion fails):
      - Capture `expected = right_val.short_display()` (the right side is conventionally the expected value in `assert (actual == expected)`)
      - Capture `actual = left_val.short_display()`
      - For `NotEq`: swap the labels — `"should not equal: {right_val}"` or keep expected/actual but note the operator
      - For ordering ops (`Lt`, `Gt`, etc.): include the operator in the message: `"expected {left_val} {op} {right_val}"`
      - Construct `LxError::assert_fail(expr_source, message, Some(expected), Some(actual), span)`
3. If the expression is NOT a binary comparison (it's just a boolean expression):
   a. Evaluate normally
   b. On false: construct error with `None, None` for expected/actual (fallback to current behavior minus the AST debug dump)

For the `expr` string field in the error: instead of `format!("{expr_node:?}")` (Rust Debug dump), use the formatter to render the expression as lx source. Call `lx::formatter::format_expr(expr_id, arena)` if such a function exists, or reconstruct a readable representation. If no single-expression formatter exists, use a simpler approach: format as `"{left_display} {op} {right_display}"` for binary comparisons, or `"<expression>"` for non-comparison asserts.

Check if the `Formatter` has a method to format a single expression (not a whole program). If not, use a simple stringification: get the source span and slice the original source text to get the expression text. The interpreter has `self.source: Arc<str>` — use `&self.source[span.offset()..span.offset()+span.len()]` to extract the original source text of the assert expression.

**ActiveForm:** Capturing comparison values in assertion evaluation

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
mcp__workflow__load_work_item({ path: "work_items/TEST_FAILURE_OUTPUT.md" })
```

Then call `next_task` to begin.
