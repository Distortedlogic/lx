# Goal

Make `lx test` assertion failures show expected and actual values instead of an AST debug dump. When `assert (result == [1, 2, 3])` fails, the agent should see `expected: [1, 2, 3]` and `actual: [1, 2, "three"]`, not `Binary(ExprBinary { op: Eq, left: ExprId(42), right: ExprId(43) })`.

# Why

LLM agents write lx tests and iterate on failing assertions. Currently `eval_assert` constructs the error with `format!("{expr_node:?}")` — the Rust Debug format of the AST node. This is useless to an agent. The agent needs to see the actual runtime values to diagnose the problem.

# Verified facts

- **eval_assert** is at `interpreter/eval.rs:222-240`. Current code:
  ```rust
  pub(super) async fn eval_assert(&mut self, expr: ExprId, msg: Option<ExprId>, span: SourceSpan) -> EvalResult<LxVal> {
      let val = self.eval(expr).await?;
      let val = self.force_defaults(val, span).await?;
      match val.as_bool() {
          Some(true) => Ok(LxVal::Unit),
          Some(false) => {
              let message = match msg {
                  Some(m) => { let mv = self.eval(m).await?; Some(mv.to_string()) },
                  None => None,
              };
              let expr_node = self.arena.expr(expr);
              Err(LxError::assert_fail(format!("{expr_node:?}"), message, span).into())
          },
          _ => Err(LxError::type_err(format!("assert requires Bool, got {} `{}`", val.type_name(), val.short_display()), span, None).into()),
      }
  }
  ```
- **LxError::Assert variant** (`error.rs:41-48`):
  ```rust
  #[error("assertion failed: {expr}")]
  #[diagnostic(code(lx::assert))]
  Assert { expr: String, message: Option<String>, #[label("assertion failed")] span: SourceSpan }
  ```
- **assert_fail constructor** (`error.rs:81-83`):
  ```rust
  pub fn assert_fail(expr: impl Into<String>, message: Option<String>, span: SourceSpan) -> Self {
      Self::Assert { expr: expr.into(), message, span }
  }
  ```
- **Note: the `message` field is captured but never rendered** — the `#[error]` format only includes `{expr}`, and there's no `#[help]` attribute on `message`. This is a pre-existing bug.
- **Interpreter struct** (`interpreter/mod.rs:40-48`):
  ```rust
  pub struct Interpreter {
      pub(crate) env: Arc<Env>,
      source: String,           // ← the source code, type is String not Arc<str>
      pub(crate) source_dir: Option<PathBuf>,
      pub(crate) module_cache: Arc<Mutex<HashMap<PathBuf, ModuleExports>>>,
      pub(crate) loading: Arc<Mutex<HashSet<PathBuf>>>,
      pub(crate) ctx: Arc<RuntimeCtx>,
      pub(crate) arena: Arc<AstArena>,
  }
  ```
- **LxVal::short_display()** (`value/mod.rs:284-287`):
  ```rust
  pub fn short_display(&self) -> String {
      let s = self.to_string();
      if s.len() > 80 { format!("{}...", &s[..77]) } else { s }
  }
  ```
- **ExprAssert struct** (`ast/expr_types.rs:177-181`): `{ expr: ExprId, msg: Option<ExprId> }`
- **ExprBinary struct** (`ast/expr_types.rs:105-110`): `{ op: BinOp, left: ExprId, right: ExprId }`
- **BinOp::Eq, BinOp::NotEq, BinOp::Lt, BinOp::Gt, BinOp::LtEq, BinOp::GtEq** — all defined in `types.rs:124-160`, all implement `Display` via strum.
- **The interpreter has `self.source: String`** and can extract source text via span: `&self.source[span.offset()..span.offset()+span.len()]` — but SourceSpan offsets are byte offsets and the source is UTF-8, so this is safe as long as spans are aligned to char boundaries (which the lexer guarantees).
- **The interpreter has `self.arena: Arc<AstArena>`** — can access any AST node by ID.

# What changes

**Modified `crates/lx/src/error.rs`:** Extend `Assert` with `expected: Option<String>` and `actual: Option<String>`. Add `#[help]` that renders message + expected/actual. Update constructor.

**Modified `crates/lx/src/interpreter/eval.rs`:** In `eval_assert`, detect binary comparison expressions, evaluate both sides separately, capture values on failure.

# Files affected

- EDIT: `crates/lx/src/error.rs` — extend Assert variant, update constructor
- EDIT: `crates/lx/src/interpreter/eval.rs` — detect comparison in assert, capture values

# Task List

### Task 1: Extend LxError::Assert with expected/actual fields

**Subject:** Add expected/actual value display and help text to assertion errors

**Description:** In `crates/lx/src/error.rs`:

Change the `Assert` variant from:
```rust
#[error("assertion failed: {expr}")]
#[diagnostic(code(lx::assert))]
Assert {
    expr: String,
    message: Option<String>,
    #[label("assertion failed")]
    span: SourceSpan,
},
```

To:
```rust
#[error("assertion failed: {expr}")]
#[diagnostic(code(lx::assert), help("{}", Self::assert_help_text(message, expected, actual)))]
Assert {
    expr: String,
    message: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    #[label("assertion failed")]
    span: SourceSpan,
},
```

Wait — miette's `#[diagnostic(help(...))]` with `Self::method()` may not work in derive macros. Check the miette docs for how to compute help text dynamically. The existing `Parse` and `Type` variants use `#[help] help: Option<String>` — a simple optional string field with the `#[help]` attribute. Follow that pattern:

```rust
#[error("assertion failed: {expr}")]
#[diagnostic(code(lx::assert))]
Assert {
    expr: String,
    message: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    #[help]
    help: Option<String>,
    #[label("assertion failed")]
    span: SourceSpan,
},
```

Update the `assert_fail` constructor:
```rust
pub fn assert_fail(
    expr: impl Into<String>,
    message: Option<String>,
    expected: Option<String>,
    actual: Option<String>,
    span: SourceSpan,
) -> Self {
    let mut help_parts: Vec<String> = Vec::new();
    if let Some(ref msg) = message {
        help_parts.push(msg.clone());
    }
    if let (Some(ref exp), Some(ref act)) = (&expected, &actual) {
        help_parts.push(format!("expected: {exp}"));
        help_parts.push(format!("  actual: {act}"));
    }
    let help = if help_parts.is_empty() { None } else { Some(help_parts.join("\n")) };
    Self::Assert { expr: expr.into(), message, expected, actual, help, span }
}
```

There should be no other call sites for `assert_fail` besides `eval.rs`. Verify by searching for `assert_fail` — if there are others, update them to pass `None, None` for the new parameters.

**ActiveForm:** Extending Assert error with expected/actual fields

### Task 2: Capture comparison values in eval_assert

**Subject:** Evaluate both sides of comparison assertions and capture values on failure

**Description:** In `crates/lx/src/interpreter/eval.rs`, replace the `eval_assert` method (lines 222-240) with:

```rust
pub(super) async fn eval_assert(&mut self, expr: ExprId, msg: Option<ExprId>, span: SourceSpan) -> EvalResult<LxVal> {
    let expr_node = self.arena.expr(expr).clone();

    // Check if assert expression is a binary comparison
    if let Expr::Binary(binary) = &expr_node {
        match binary.op {
            BinOp::Eq | BinOp::NotEq | BinOp::Lt | BinOp::Gt | BinOp::LtEq | BinOp::GtEq => {
                let left_val = self.eval(binary.left).await?;
                let left_val = self.force_defaults(left_val, span).await?;
                let right_val = self.eval(binary.right).await?;
                let right_val = self.force_defaults(right_val, span).await?;

                // Now evaluate the full comparison
                let result = self.eval(expr).await?;
                let result = self.force_defaults(result, span).await?;

                match result.as_bool() {
                    Some(true) => return Ok(LxVal::Unit),
                    Some(false) => {
                        let message = match msg {
                            Some(m) => { let mv = self.eval(m).await?; Some(mv.to_string()) },
                            None => None,
                        };
                        // Extract source text for the expression
                        let expr_text = if span.offset() + span.len() <= self.source.len() {
                            self.source[span.offset()..span.offset() + span.len()].to_string()
                        } else {
                            format!("{} {} {}", left_val.short_display(), binary.op, right_val.short_display())
                        };
                        return Err(LxError::assert_fail(
                            expr_text,
                            message,
                            Some(right_val.short_display()),
                            Some(left_val.short_display()),
                            span,
                        ).into());
                    },
                    _ => {
                        return Err(LxError::type_err(
                            format!("assert requires Bool, got {} `{}`", result.type_name(), result.short_display()),
                            span, None,
                        ).into());
                    },
                }
            },
            _ => {}, // Fall through to general case for non-comparison binary ops
        }
    }

    // General case: non-comparison expression
    let val = self.eval(expr).await?;
    let val = self.force_defaults(val, span).await?;
    match val.as_bool() {
        Some(true) => Ok(LxVal::Unit),
        Some(false) => {
            let message = match msg {
                Some(m) => { let mv = self.eval(m).await?; Some(mv.to_string()) },
                None => None,
            };
            let expr_text = if span.offset() + span.len() <= self.source.len() {
                self.source[span.offset()..span.offset() + span.len()].to_string()
            } else {
                format!("{:?}", expr_node)
            };
            Err(LxError::assert_fail(expr_text, message, None, None, span).into())
        },
        _ => Err(LxError::type_err(format!("assert requires Bool, got {} `{}`", val.type_name(), val.short_display()), span, None).into()),
    }
}
```

**Important note on double-evaluation**: The comparison case evaluates `left` and `right` separately to capture their values, then evaluates the full expression again for the comparison result. This means the expression is evaluated twice. For pure expressions (which assertions should be), this is fine. For expressions with side effects, the assert could trigger effects twice — but assert expressions should be pure comparisons, so this is acceptable.

**Alternative to avoid double-eval**: Instead of re-evaluating the full expression, perform the comparison manually:
```rust
let comparison_result = match binary.op {
    BinOp::Eq => left_val == right_val,
    BinOp::NotEq => left_val != right_val,
    // For Lt, Gt, etc. — check if LxVal implements PartialOrd
    _ => { /* fall through to general case */ },
};
```
Check if `LxVal` implements `PartialOrd` or has comparison methods. If it does, use the direct comparison to avoid double-evaluation. If not, the double-eval approach is fine.

Add necessary imports at the top of `eval.rs`:
```rust
use crate::ast::{Expr, BinOp, ExprBinary};
```
Check what's already imported — `Expr` and `BinOp` may already be in scope.

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
