# Goal

Add `Expr::Invalid` and `Stmt::Invalid` variants to the AST so the parser can emit placeholder nodes during error recovery instead of dropping statements entirely. Each consumer (checker, formatter, interpreter, visitor, transformer) gets a defined strategy for handling invalid nodes: the checker emits no cascading diagnostics, the formatter reproduces the original source text, the interpreter returns an error, and the visitor/transformer pass through without descending.

# Why

- The parser currently recovers from syntax errors by skipping tokens and producing `None` for the failed statement (in `stmt.rs` lines 19-27). The `None` is filtered out via `.flatten()`, leaving a gap in the AST. Any bindings, type definitions, or imports in the skipped region are invisible to all downstream passes
- A single syntax error causes cascading false diagnostics: if `let x = <broken>` is dropped, every subsequent reference to `x` produces an `UnknownIdent` error. The checker has `Type::Error` and `Type::Unknown` for internal recovery but no way to know that a definition was attempted and failed
- Every production language toolchain (rustc, TypeScript, ruff, roslyn) uses error/invalid nodes to preserve partial AST structure through syntax errors. This is the standard approach for IDE-quality diagnostics

# What changes

**AST:** Add `Expr::Invalid` and `Stmt::Invalid` variants. Both are unit variants — the span is tracked by the `Spanned<T>` wrapper in the arena. Both are leaf nodes with no children — the visitor skips them, the transformer passes them through unchanged.

**Parser:** Instead of mapping parse failures to `None` and filtering, map them to `Stmt::Invalid` allocated in the arena. The span covers the skipped token range.

**Checker:** When encountering `Stmt::Invalid`, skip it silently — the parser already emitted a syntax error diagnostic. When encountering `Expr::Invalid`, return `Type::Error` without emitting additional diagnostics.

**Formatter:** When encountering `Stmt::Invalid` or `Expr::Invalid`, emit the original source text from the span (requires access to source text).

**Interpreter:** When encountering either invalid variant, return an error immediately.

**Visitor/Transformer:** `Expr::Invalid` and `Stmt::Invalid` are leaf nodes — no children to walk. The AstWalk macro handles this automatically via `#[walk(skip)]` or by having no walkable fields.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/ast/mod.rs` | Add `Invalid` variant to `Expr` and `Stmt` enums |
| `crates/lx/src/parser/stmt.rs` | Emit `Stmt::Invalid` on recovery instead of `None` |
| `crates/lx/src/checker/visit_stmt.rs` | Handle `Stmt::Invalid` — skip silently |
| `crates/lx/src/checker/type_ops.rs` | Handle `Expr::Invalid` — return `Type::Error` |
| `crates/lx/src/checker/check_expr.rs` | Handle `Expr::Invalid` in exhaustive match |
| `crates/lx/src/formatter/emit_stmt.rs` | Emit original source text for `Stmt::Invalid` |
| `crates/lx/src/formatter/emit_expr.rs` | Emit original source text for `Expr::Invalid` |
| `crates/lx/src/interpreter/exec_stmt.rs` | Return error for `Stmt::Invalid` |
| `crates/lx/src/interpreter/mod.rs` | Return error for `Expr::Invalid` |
| `crates/lx/src/visitor/walk/mod.rs` | Handle `Stmt::Invalid` and `Expr::Invalid` as no-ops |
| `crates/lx/src/folder/desugar.rs` | Pass through invalid nodes unchanged |
| `crates/lx/src/ast/display.rs` | Handle `Pattern` display (no change needed — invalid is expr/stmt only) |

# Task List

### Task 1: Add Invalid variants to Expr and Stmt

In `crates/lx/src/ast/mod.rs`:

Add to the `Stmt` enum, after the last variant:
```rust
Invalid,
```

Add to the `Expr` enum, after the last variant:
```rust
Invalid,
```

These are unit variants with no fields. The span is already tracked by the `Spanned<T>` wrapper in the arena — every node has a span via `arena.expr_span(id)` / `arena.stmt_span(id)`. No additional span field is needed on the variant itself.

The `AstWalk` derive on both enums will automatically generate empty walk/children/recurse implementations for unit variants (see `walk_enum.rs` line 34-36: `Fields::Unit => recurse returns self, children returns empty, walk does nothing`).

### Task 2: Update parser to emit Stmt::Invalid on recovery

In `crates/lx/src/parser/stmt.rs`:

The arena is `Rc<RefCell<AstArena>>` (aliased as `ArenaRef` in `parser/mod.rs`). Parsers access it via `arena.borrow_mut().alloc_stmt(...)`. Spans are captured via `map_with(|value, ctx| ss(ctx.span()))` where `ss()` converts chumsky Span to miette SourceSpan.

The current recovery code (stmt.rs lines 19-27):
```rust
let skip_to_semi = any().and_is(none_of([TokenKind::Semi, TokenKind::Eof])).repeated().at_least(1).then(just(TokenKind::Semi).or_not()).ignored();
let recoverable_stmt = stmt_parser(expr, arena).map(Some).recover_with(via_parser(skip_to_semi.map(|_| None)));
```

The `skip_to_semi.map(|_| None)` drops the span. Change to `map_with` to capture it. The arena needs to be cloned into the closure:

```rust
let a_recovery = arena.clone();
let recoverable_stmt = stmt_parser(expr, arena)
    .recover_with(via_parser(skip_to_semi.map_with(move |_, ctx| {
        a_recovery.borrow_mut().alloc_stmt(Stmt::Invalid, ss(ctx.span()))
    })));
```

Remove the `.map(Some)` from the success path. The program_parser's final `.map()` changes from:
```rust
.map(|stmts: Vec<Option<StmtId>>| stmts.into_iter().flatten().collect())
```
To:
```rust
.collect::<Vec<StmtId>>()
```

### Task 3: Handle Invalid in checker

In `crates/lx/src/checker/visit_stmt.rs`:

Add a match arm for `Stmt::Invalid` in `check_stmt` (or whatever the top-level statement dispatch function is called). The arm should do nothing — no diagnostics, no type recording:
```rust
Stmt::Invalid => {},
```

In `crates/lx/src/checker/type_ops.rs` (in `synth_expr_inner` or the equivalent exhaustive expression match):

Add:
```rust
Expr::Invalid => self.type_arena.error(),
```

This returns the error type, which suppresses unification errors downstream — any expression that consumes an Invalid result will unify with `Type::Error` and not produce cascading diagnostics (the checker already has this behavior for `Type::Error`).

In `crates/lx/src/checker/check_expr.rs`:

If `check_expr` has an exhaustive match on `Expr` variants (per the CHECKER_HYGIENE work item), add `Expr::Invalid` to the appropriate arm. It should return `self.type_arena.error()`.

### Task 4: Handle Invalid in formatter

The Formatter struct has only `arena: &'a AstArena`, `output: String`, `indent: usize` — no source text field. Add `source: &'a str` to the Formatter struct. Thread it from the `format()` entry point (which receives the Program — get source text from the caller or add source to Program). Update the Formatter constructor and `format()` function signature to accept `source: &str`.

In `crates/lx/src/formatter/emit_stmt.rs`:

Add a match arm for `Stmt::Invalid`:
```rust
Stmt::Invalid => {
    let span = self.arena.stmt_span(id);
    self.write(&self.source[span.offset()..span.offset() + span.len()]);
},
```

In `crates/lx/src/formatter/emit_expr.rs`:

Add a match arm for `Expr::Invalid`:
```rust
Expr::Invalid => {
    let span = self.arena.expr_span(id);
    self.write(&self.source[span.offset()..span.offset() + span.len()]);
},
```

### Task 5: Handle Invalid in interpreter

In `crates/lx/src/interpreter/exec_stmt.rs`:

Interpreter errors use `LxError::runtime(message, span)`. Add a match arm for `Stmt::Invalid`:
```rust
Stmt::Invalid => {
    return Err(LxError::runtime("cannot execute invalid statement (syntax error)", span));
},
```

In `crates/lx/src/interpreter/mod.rs` (in the `eval` method's match on `Expr`):

Add:
```rust
Expr::Invalid => {
    Err(LxError::runtime("cannot evaluate invalid expression (syntax error)", span))
},
```

### Task 6: Handle Invalid in visitor walk functions

In `crates/lx/src/visitor/walk/mod.rs`:

In `walk_stmt` (the match on `stmt`), add:
```rust
Stmt::Invalid => {},
```

In `walk_expr` (the match on `expr`), add:
```rust
Expr::Invalid => {},
```

These are no-ops — invalid nodes have no children to walk. The `visit_stmt`/`visit_expr` and `leave_stmt`/`leave_expr` hooks on the visitor trait will still fire for invalid nodes (they fire for ALL nodes before/after the walk), so visitor consumers can detect invalid nodes if they choose to.

### Task 7: Handle Invalid in desugarer and transformer

In `crates/lx/src/folder/desugar.rs`:

The desugarer implements `AstTransformer` and overrides `leave_expr`. Since `Expr::Invalid` won't match any of the desugaring patterns (Pipe, Section, Ternary, Coalesce, Literal with interpolation, With binding), it will fall through to the `other => other` arm and be preserved unchanged. No explicit change needed.

`desugar.rs` `leave_expr` has a catch-all `other => other` at line 45. `Expr::Invalid` falls through unchanged. No change needed.

In `crates/lx/src/folder/validate_core.rs`:

`validate_core.rs` `visit_expr` has a catch-all `_ => VisitAction::Descend` at line 21. `Expr::Invalid` falls through. No change needed.

### Task 8: Handle Invalid in linter and capture analysis

In `crates/lx/src/checker/capture.rs`:

If the free variable collector has an exhaustive match on `Expr`, add:
```rust
Expr::Invalid => {},
```

Invalid nodes have no sub-expressions, so no free variables.

In `crates/lx/src/linter/rules/`:

Check each lint rule file. Rules that match on specific `Expr` variants (like `Expr::Match`, `Expr::Break`, `Expr::Record`) will naturally ignore `Expr::Invalid` since it won't match. Rules that iterate statement lists (like unreachable_code) should skip `Stmt::Invalid` entries — they represent regions where the parser already reported an error.

### Task 9: Compile, format, and verify

Run `just fmt` to format all changed files.

Run `just diagnose`. The compiler will identify any remaining exhaustive match sites that need `Invalid` arms. Follow each error to its source and add the appropriate handling (no-op for analysis passes, error for execution).

Run `just test` to verify all existing tests pass.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Do not add, skip, reorder, or combine tasks.**
3. **Invalid nodes must NOT produce cascading diagnostics.** The parser emits the syntax error. Every downstream pass must be silent about Invalid nodes.
4. **Invalid variants are unit variants** — the span comes from `Spanned<T>` in the arena. Do not add a span field to the variant.
5. **The chumsky parser threading model** matters for Task 2. Read how the arena is passed through the parser before changing the recovery logic. Match the existing convention exactly.
6. **`Type::Error` already suppresses cascading unification errors** in the checker. Returning `Type::Error` for `Expr::Invalid` is sufficient.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AST_ERROR_RECOVERY_NODES.md" })
```

Then call `next_task` to begin.
