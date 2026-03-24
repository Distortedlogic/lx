# Goal

Rewrite `validate_core` as an `AstVisitor` implementation instead of manual recursion, gaining complete traversal of all node types (including Pattern and TypeExpr subtrees that the current implementation skips).

# Prerequisites

None. This work item uses the current visitor API. The pre-visit expression hook is currently named `on_expr` (in `visitor/mod.rs` line 75). If AST_VISITOR_HOOKS_NORMALIZE has already been applied when this executes, the name will be `visit_expr` instead — check the actual trait definition before writing the impl.

# Why

- `validate_core` hand-rolls its own tree walk via manual recursion through `validate_stmt` and `validate_expr` functions. It only recurses into `NodeId::Expr` and `NodeId::Stmt` children, silently ignoring `NodeId::Pattern` and `NodeId::TypeExpr`. If a surface-only construct were nested inside a pattern or type expression subtree, it would escape detection
- The manual recursion duplicates what `AstVisitor` already provides. Using the visitor eliminates the duplication and guarantees complete coverage of all node types

# What changes

Replace the three functions (`validate_core`, `validate_stmt`, `validate_expr`) with a single `CoreValidator` struct that implements `AstVisitor`. Override `visit_expr` to check for disallowed surface constructs (Pipe, Section, Ternary, Coalesce, With(Binding)). The visitor infrastructure handles complete traversal automatically.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/folder/validate_core.rs` | Complete rewrite |

# Task List

### Task 1: Rewrite validate_core.rs

Replace the entire contents of `crates/lx/src/folder/validate_core.rs` with a struct implementing AstVisitor.

The new content should:

1. Define a `struct CoreValidator;`

2. Implement `AstVisitor for CoreValidator` with a single override — `on_expr` (the current name in the trait). If AST_VISITOR_HOOKS_NORMALIZE has already run when this executes, the name will be `visit_expr` — read `crates/lx/src/visitor/mod.rs` to confirm the actual method name. The implementation checks the expr and panics if it finds a disallowed surface construct:

   - `Expr::Pipe(_)` → panic with message about desugarer not converting to Apply
   - `Expr::Section(_)` → panic with message about desugarer not converting to lambda
   - `Expr::Ternary(_)` → panic with message about desugarer not converting to Match
   - `Expr::Coalesce(_)` → panic with message about desugarer not converting to Match
   - `Expr::With(w) if matches!(w.kind, WithKind::Binding { .. })` → panic with message about desugarer not converting to Block
   - All other variants → return `VisitAction::Descend`

   Include the span offset in panic messages (access via the `span` parameter).

3. Define the public function:

   ```rust
   pub(super) fn validate_core(program: &Program<Core>) {
       let mut validator = CoreValidator;
       validator.visit_program(program);
   }
   ```

Required imports:
- `use crate::ast::{Core, Expr, ExprId, Program, WithKind, AstArena};`
- `use crate::visitor::{AstVisitor, VisitAction};`
- `use miette::SourceSpan;`

The visitor infrastructure automatically traverses all children including Pattern and TypeExpr subtrees — no manual recursion needed.

### Task 2: Format and commit

Run `just fmt` then `git add -A && git commit -m "refactor: rewrite validate_core as AstVisitor impl"`.

### Task 3: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **Check the actual method name** in `visitor/mod.rs` before writing the impl. It is `on_expr` currently. It will be `visit_expr` if AST_VISITOR_HOOKS_NORMALIZE has already been applied.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/VALIDATE_CORE_REWRITE.md" })
```

Then call `next_task` to begin.
