# Goal

Eliminate the redundant `LintRule` trait by making lint rules implement `AstVisitor` directly, and add stable error codes to all diagnostics.

# Prerequisites

**AST_VISITOR_HOOKS_NORMALIZE must be completed first.** This work item assumes `on_expr` has been renamed to `visit_expr` and `on_stmt` to `visit_stmt`.

# Why

- `LintRule` is a weaker duplicate of `AstVisitor` — it defines `check_expr`, `check_stmt`, `check_pattern`, `enter_expr`, `leave_expr` which are a strict subset of AstVisitor's hooks. LintRunner bridges the two, forwarding AstVisitor calls to LintRule calls. This translation layer means lint rules cannot hook into fine-grained nodes like `visit_binary` or `visit_func` — they must re-dispatch inside check_expr themselves
- Diagnostics have a `DiagnosticKind` enum but no stable error code string. Users cannot filter or suppress specific diagnostics, and tooling cannot identify errors programmatically

# Object-Safety Constraint

`AstVisitor` has generic methods `visit_program<P>` and `leave_program<P>`, making it NOT object-safe. `Box<dyn AstVisitor>` is impossible. The solution: each rule implements `AstVisitor` on its concrete struct (monomorphized), and a separate object-safe `LintRule` trait provides the dynamic dispatch interface. The `LintRule::run` method internally calls walk functions with `self` as the concrete `AstVisitor` impl. The registry stores `Box<dyn LintRule>`.

# What changes

1. Redefine `LintRule` as an object-safe trait with `run`, `take_diagnostics`, `name`, `code`, `category`
2. Each lint rule struct implements `AstVisitor` (for fine-grained hooks) AND `LintRule` (for dynamic dispatch)
3. `LintRule::run` calls `dispatch_stmt` in a loop over the program stmts, passing `self` as the concrete `AstVisitor` — this monomorphizes the walk at compile time while allowing dyn dispatch at the `run` call site
4. `SemanticModel` is passed to `run` so rules that need it (unused_import, redundant_propagate) can use it directly
5. Add `code: &'static str` field to `Diagnostic`
6. Assign codes to all `DiagnosticKind` variants and all lint rules

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/linter/rule.rs` | Redefine LintRule trait |
| `crates/lx/src/linter/runner.rs` | Simplify to iterate Box\<dyn LintRule\> |
| `crates/lx/src/linter/registry.rs` | Update type to Box\<dyn LintRule\> |
| `crates/lx/src/linter/mod.rs` | Update exports |
| `crates/lx/src/linter/rules/break_outside_loop.rs` | Implement AstVisitor + LintRule |
| `crates/lx/src/linter/rules/duplicate_record_field.rs` | Implement AstVisitor + LintRule |
| `crates/lx/src/linter/rules/empty_match.rs` | Implement AstVisitor + LintRule |
| `crates/lx/src/linter/rules/mut_never_mutated.rs` | Add code field to diagnostics |
| `crates/lx/src/linter/rules/redundant_propagate.rs` | Implement AstVisitor + LintRule |
| `crates/lx/src/linter/rules/single_branch_par.rs` | Implement AstVisitor + LintRule |
| `crates/lx/src/linter/rules/unreachable_code.rs` | Implement AstVisitor + LintRule |
| `crates/lx/src/linter/rules/unused_import.rs` | Implement AstVisitor + LintRule |
| `crates/lx/src/checker/mod.rs` | Add code field to Diagnostic |
| `crates/lx/src/checker/diagnostics.rs` | Add code() method to DiagnosticKind |

# Task List

### Task 1: Add code field to Diagnostic and DiagnosticKind

In `crates/lx/src/checker/mod.rs`, add `pub code: &'static str` field to the `Diagnostic` struct.

In `crates/lx/src/checker/diagnostics.rs`, add a `pub fn code(&self) -> &'static str` method to `DiagnosticKind`:

- `NegationRequiresNumeric` → `"E001"`
- `PropagateRequiresResultOrMaybe` → `"E002"`
- `TernaryCondNotBool` → `"E003"`
- `TimeoutMsNotNumeric` → `"E004"`
- `LogicalOpRequiresBool` → `"E005"`
- `MutableCaptureInConcurrent` → `"E006"`
- `NonExhaustiveMatch` → `"E007"`
- `DuplicateImport` → `"W001"`
- `UnknownImport` → `"E008"`
- `TypeMismatch` → `"E009"`
- `LintWarning` → `"L000"`
- `UnknownIdent` → `"E010"`
- `UnknownModule` → `"E011"`

Update `Checker::emit` to use `code: kind.code()` when constructing Diagnostic. Update `make_type_error_diagnostic` similarly.

There are exactly 3 other `Diagnostic {` construction sites in the checker: `visit_stmt.rs` line 189 (DuplicateImport), and 2 in `mod.rs` (emit, make_type_error_diagnostic). Search with `rg --type rust 'Diagnostic \{' crates/lx/src/` and add the `code` field to every one.

### Task 2: Redefine LintRule trait

In `crates/lx/src/linter/rule.rs`, replace the entire file contents with:

```rust
use crate::ast::{AstArena, StmtId};
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCategory {
    Style,
    Correctness,
    Performance,
    Concurrency,
}

pub trait LintRule {
    fn name(&self) -> &'static str;
    fn code(&self) -> &'static str;
    fn category(&self) -> RuleCategory;
    fn run(&mut self, stmts: &[StmtId], arena: &AstArena, model: &SemanticModel);
    fn take_diagnostics(&mut self) -> Vec<Diagnostic>;
}
```

This trait is object-safe — no generic methods, no `Self` constraints. Each rule's `run` implementation calls `dispatch_stmt(self, sid, arena)` from the visitor walk module. Since `dispatch_stmt` takes `V: AstVisitor + ?Sized` and each rule implements `AstVisitor` on its concrete struct, the call is monomorphized. The `Box<dyn LintRule>` dispatch happens at the `run` call level, not at individual visitor hooks.

### Task 3: Rewrite LintRunner / lint function

In `crates/lx/src/linter/runner.rs`, simplify to:

```rust
use crate::ast::Program;
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;
use crate::linter::rules::mut_never_mutated::check_unused_mut;
use super::registry::RuleRegistry;

pub fn lint<P>(program: &Program<P>, model: &SemanticModel, registry: &mut RuleRegistry) -> Vec<Diagnostic> {
    let mut all_diags = Vec::new();
    for rule in registry.rules_mut() {
        rule.run(&program.stmts, &program.arena, model);
        all_diags.extend(rule.take_diagnostics());
    }
    all_diags.extend(check_unused_mut(program, model, &program.arena));
    all_diags
}
```

Delete the `LintRunner` struct and its `AstVisitor` impl entirely. The runner is now just this function.

### Task 4: Update RuleRegistry

In `crates/lx/src/linter/registry.rs`, change the type from `Vec<Box<dyn LintRule>>` to use the new `LintRule` trait (same name, different definition). No API change needed — `register` still takes `Box<dyn LintRule>`, `rules_mut` still returns `&mut [Box<dyn LintRule>]`.

### Task 5: Convert break_outside_loop

Current state: struct `BreakOutsideLoop` with `loop_depth: usize` field. Uses `enter_expr` to increment on Loop, `leave_expr` to decrement, `check_expr` to detect Break at depth 0.

New implementation:
- Add `diagnostics: Vec<Diagnostic>` field. Initialize in `new()`.
- Implement `AstVisitor`: override `visit_expr` — if Loop, increment depth; if Break at depth 0, push diagnostic to `self.diagnostics`. Return `VisitAction::Descend`. Override `leave_expr` — if Loop, decrement depth.
- Implement `LintRule`: `name` → `"break_outside_loop"`, `code` → `"L001"`, `category` → `Correctness`. `run` iterates stmts calling `crate::visitor::dispatch_stmt(self, sid, arena)`. `take_diagnostics` drains `self.diagnostics`.
- Remove the old `LintRule` impl (the trait no longer exists with the old methods).

### Task 6: Convert duplicate_record_field

Current state: unit struct `DuplicateRecordField`. Uses `check_expr` to detect Expr::Record with duplicate field names via HashSet.

New implementation:
- Add `diagnostics: Vec<Diagnostic>` field.
- Implement `AstVisitor`: override `visit_expr` — if `Expr::Record`, check for duplicate Named fields, push diagnostic. Return Descend.
- Implement `LintRule`: `name` → `"duplicate_record_field"`, `code` → `"L006"`, `category` → `Correctness`. `run` iterates stmts calling `dispatch_stmt`. `take_diagnostics` drains.

### Task 7: Convert empty_match

Current state: unit struct `EmptyMatch`. Uses `check_expr` with `ExprMatcher::empty_match()`.

New implementation:
- Add `diagnostics: Vec<Diagnostic>` field.
- Implement `AstVisitor`: override `visit_expr` — use the same ExprMatcher logic, push diagnostic if matched. Return Descend.
- Implement `LintRule`: `name` → `"empty-match"`, `code` → `"L002"`, `category` → `Correctness`. Same `run`/`take_diagnostics` pattern.

### Task 8: Convert redundant_propagate

Current state: unit struct `RedundantPropagate`. Uses `check_expr` — checks `Expr::Propagate`, uses `model.type_of_expr()` and `model.type_arena` to verify type is Result or Maybe.

New implementation:
- Add `diagnostics: Vec<Diagnostic>` and `model: Option<*const SemanticModel>` fields. The raw pointer avoids lifetime issues; the model outlives the `run` call.
- Implement `AstVisitor`: override `visit_expr` — if `Expr::Propagate`, access model via `unsafe { &*self.model.unwrap() }`, perform the same type check. Push diagnostic if needed.
- Implement `LintRule`: `run` stores the model pointer (`self.model = Some(model as *const _)`) before iterating stmts, then clears it after. `code` → `"L003"`.

### Task 9: Convert single_branch_par

Current state: unit struct `SingleBranchPar`. Uses `check_expr` to detect `Expr::Par` with 0 or 1 stmts.

New implementation:
- Add `diagnostics: Vec<Diagnostic>` field.
- Implement `AstVisitor`: override `visit_expr` — check for `Expr::Par` with `stmts.len() <= 1`. Return Descend.
- Implement `LintRule`: `code` → `"L007"`. Same `run`/`take_diagnostics` pattern.

### Task 10: Convert unreachable_code

Current state: unit struct `UnreachableCode`. Uses `check_expr` to scan Block/Loop stmts for code after Break.

New implementation:
- Add `diagnostics: Vec<Diagnostic>` field.
- Implement `AstVisitor`: override `visit_expr` — same logic scanning stmts for Break followed by more stmts. Return Descend.
- Implement `LintRule`: `code` → `"L004"`. Same pattern.

### Task 11: Convert unused_import

Current state: unit struct `UnusedImport`. Uses `check_stmt` — matches `Stmt::Use`, uses `model.definitions` and `model.references_to()`.

New implementation:
- Add `diagnostics: Vec<Diagnostic>` and `model: Option<*const SemanticModel>` fields.
- Implement `AstVisitor`: override `visit_stmt` — if `Stmt::Use`, access model via `unsafe { &*self.model.unwrap() }`, perform the same definition/reference check. Push diagnostic if unused.
- Implement `LintRule`: `run` stores model pointer before iterating stmts, clears after. `code` → `"L005"`.

### Task 12: Update mut_never_mutated diagnostics

`mut_never_mutated.rs` is a standalone function (`check_unused_mut`), not a LintRule impl. Leave it as a standalone function. Update its `Diagnostic` construction to include `code: "L008"`.

### Task 13: Update linter mod.rs exports

In `crates/lx/src/linter/mod.rs`, update exports:

```rust
pub use rule::{LintRule, RuleCategory};
```

### Task 14: Format and commit

Run `just fmt` then `git add -A && git commit -m "refactor: unify lint rules as AstVisitor impls, add diagnostic error codes"`.

### Task 15: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **AstVisitor is NOT object-safe** due to generic `visit_program<P>`. Do NOT attempt `Box<dyn AstVisitor>`. Each rule implements AstVisitor on its concrete struct; dyn dispatch happens via the separate `LintRule` trait.
5. **Every rule's `run` method** must call `crate::visitor::dispatch_stmt(self, sid, arena)` — import `dispatch_stmt` from `crate::visitor`.
6. **Rules needing SemanticModel** (unused_import, redundant_propagate) store a raw pointer set in `run` and cleared after. This is safe because the model outlives the run call.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/LINT_INFRASTRUCTURE_UNIFY.md" })
```

Then call `next_task` to begin.
