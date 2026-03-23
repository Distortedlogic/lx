# Goal

Add structural lint rules to the existing linter framework and wire the linter into `lx check` so lint warnings appear alongside type errors in the same diagnostic output. No separate `lx lint` command.

# Why

LLM agents writing lx programs make structural mistakes that pass the type checker but fail at runtime: `break` outside a loop, unreachable code after a break, importing names they never use, duplicate fields in records. The type checker is structurally unable to catch these — they require a separate validation pass.

The linter framework is ready: `LintRule` trait, `RuleRegistry`, `LintRunner` with `AstVisitor`. Just needs rules, two small trait extensions, and integration.

# Verified facts

- **LintRule trait** (`rule.rs:14-29`) has 5 methods: `name()`, `category()`, `check_expr()`, `check_stmt()`, `check_pattern()`. It does NOT have `enter_expr` or `leave_expr` hooks.
- **LintRunner** (`runner.rs:10-52`) implements `AstVisitor` with only `on_expr`, `on_stmt`, `visit_pattern`. It does NOT implement `leave_expr`. The `AstVisitor` trait DOES define `leave_expr` — LintRunner just doesn't override it.
- **`lint<P>` signature** (`runner.rs:48`): `pub fn lint<P>(program: &Program<P>, model: &SemanticModel, registry: &mut RuleRegistry) -> Vec<Diagnostic>`. Generic over P — works on Core.
- **RuleRegistry::default_rules()** (`registry.rs`): registers `EmptyMatch` and `RedundantPropagate`.
- **Exact AST variant shapes** (from `ast/mod.rs` and `ast/expr_types.rs`):
  - `Expr::Loop(Vec<StmtId>)` — loop body is a vec of stmts
  - `Expr::Break(Option<ExprId>)` — optional break value
  - `Expr::Block(Vec<StmtId>)` — block body
  - `Expr::Par(Vec<StmtId>)` — par branches are stmts
  - `Expr::Record(Vec<RecordField>)` where `RecordField::Named { name: Sym, value: ExprId }` or `RecordField::Spread(ExprId)`
  - `Stmt::Binding(Binding)` where `Binding { exported, mutable, target: BindTarget, type_ann, value }`
  - `Stmt::Use(UseStmt)` where `UseStmt { path: Vec<Sym>, kind: UseKind }`
  - `Stmt::FieldUpdate(StmtFieldUpdate)` where `StmtFieldUpdate { name: Sym, fields: Vec<Sym>, value: ExprId }`
  - `Stmt::Expr(ExprId)` — expression statement
- **Reference struct** (`semantic.rs:51-54`): `{ expr_id: ExprId, definition: DefinitionId }`. Does NOT track read vs write.
- **SemanticModel** has `references_to(def: DefinitionId) -> Vec<ExprId>` method.
- **DefinitionInfo** has `name: Sym, kind: DefKind, span, ty, scope, mutable`.
- **DefKind variants**: `Binding`, `FuncParam`, `PatternBind`, `Import`, `TypeDef`, `TraitDef`, `ClassDef`, `WithBinding`, `ResourceBinding`.
- **Checker.check()** at `mod.rs:217-221` builds CheckResult with `diagnostics: checker.diagnostics`. Linter diagnostics need to be appended here.

# What changes

**Modified `crates/lx/src/linter/rule.rs`:** Add `enter_expr` and `leave_expr` default methods to `LintRule` trait.

**Modified `crates/lx/src/linter/runner.rs`:** Add `leave_expr` impl to `LintRunner`'s `AstVisitor` that calls `rule.leave_expr()` for each rule. Rename existing `on_expr` dispatch to also call `rule.enter_expr()`.

**Modified `crates/lx/src/checker/mod.rs`:** After `checker.check_program(program)`, run `linter::lint(&program, &semantic, &mut registry)` and append diagnostics.

**New rule files in `crates/lx/src/linter/rules/`:** One file per rule.

**Modified `crates/lx/src/linter/rules/mod.rs`:** Register new rules.

# Files affected

- EDIT: `crates/lx/src/linter/rule.rs` — add enter_expr, leave_expr hooks
- EDIT: `crates/lx/src/linter/runner.rs` — implement leave_expr on LintRunner, call enter_expr in on_expr
- EDIT: `crates/lx/src/checker/mod.rs` — integrate linter after type checking
- NEW: `crates/lx/src/linter/rules/break_outside_loop.rs`
- NEW: `crates/lx/src/linter/rules/unreachable_code.rs`
- NEW: `crates/lx/src/linter/rules/unused_import.rs`
- NEW: `crates/lx/src/linter/rules/duplicate_record_field.rs`
- NEW: `crates/lx/src/linter/rules/single_branch_par.rs`
- NEW: `crates/lx/src/linter/rules/mut_never_mutated.rs`
- EDIT: `crates/lx/src/linter/rules/mod.rs` — declare new modules, register in default_rules()

# Task List

### Task 1: Add enter/leave hooks to LintRule trait and LintRunner

**Subject:** Extend LintRule with enter_expr/leave_expr and wire into LintRunner

**Description:** Two files:

**File 1: `crates/lx/src/linter/rule.rs`**

Add two new default methods to the `LintRule` trait (after the existing `check_pattern` method at line 29):

```rust
fn enter_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) {}
fn leave_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) {}
```

These are void — they don't return diagnostics. They're for tracking state (like loop depth).

**File 2: `crates/lx/src/linter/runner.rs`**

In `LintRunner`'s `AstVisitor` impl:

1. In the existing `on_expr` method (lines 23-29), add a call to `rule.enter_expr()` before `rule.check_expr()`:
   ```rust
   fn on_expr(&mut self, id: ExprId, expr: &Expr, span: SourceSpan, arena: &AstArena) -> VisitAction {
       for rule in self.rules.iter_mut() {
           rule.enter_expr(id, expr, span, arena);
           let mut diags = rule.check_expr(id, expr, span, self.model, arena);
           self.diagnostics.append(&mut diags);
       }
       VisitAction::Descend
   }
   ```

2. Add a `leave_expr` implementation to the `AstVisitor` impl:
   ```rust
   fn leave_expr(&mut self, id: ExprId, expr: &Expr, span: SourceSpan, arena: &AstArena) -> std::ops::ControlFlow<()> {
       for rule in self.rules.iter_mut() {
           rule.leave_expr(id, expr, span, arena);
       }
       std::ops::ControlFlow::Continue(())
   }
   ```

   Check the exact signature of `AstVisitor::leave_expr` in `visitor/mod.rs` — the return type is `ControlFlow<()>`. Make sure it matches.

**ActiveForm:** Adding enter/leave hooks to LintRule trait and LintRunner

### Task 2: Wire linter into the check pipeline

**Subject:** Run linter after type checking and merge diagnostics

**Description:** In `crates/lx/src/checker/mod.rs`:

The `check` function (lines 217-221) is:
```rust
pub fn check(program: &Program<Core>, source: Arc<str>) -> CheckResult {
    let mut checker = Checker::new(&program.arena);
    checker.check_program(program);
    let semantic = checker.sem.build(checker.expr_types, checker.type_defs, checker.trait_fields, checker.type_arena);
    CheckResult { diagnostics: checker.diagnostics, source, semantic }
}
```

Change to:
```rust
pub fn check(program: &Program<Core>, source: Arc<str>) -> CheckResult {
    let mut checker = Checker::new(&program.arena);
    checker.check_program(program);
    let semantic = checker.sem.build(checker.expr_types, checker.type_defs, checker.trait_fields, checker.type_arena);
    let mut diagnostics = checker.diagnostics;
    let mut registry = crate::linter::RuleRegistry::default_rules();
    let lint_diags = crate::linter::lint(program, &semantic, &mut registry);
    diagnostics.extend(lint_diags);
    CheckResult { diagnostics, source, semantic }
}
```

Do the same for `check_with_imports` (lines 224-244) — after building `semantic`, run linter and extend diagnostics.

Verify the import: `crate::linter::RuleRegistry` and `crate::linter::lint` should be accessible since both are `pub` exports from the linter module.

**ActiveForm:** Wiring linter into check pipeline

### Task 3: Rule — break_outside_loop

**Subject:** Detect break expressions outside of loop context

**Description:** Create `crates/lx/src/linter/rules/break_outside_loop.rs`:

```rust
use crate::ast::{AstArena, Expr, ExprId};
use crate::checker::Diagnostic;
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct BreakOutsideLoop {
    loop_depth: usize,
}

impl BreakOutsideLoop {
    pub fn new() -> Self {
        Self { loop_depth: 0 }
    }
}

impl LintRule for BreakOutsideLoop {
    fn name(&self) -> &'static str { "break_outside_loop" }
    fn category(&self) -> RuleCategory { RuleCategory::Correctness }

    fn enter_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, _arena: &AstArena) {
        if matches!(expr, Expr::Loop(_)) {
            self.loop_depth += 1;
        }
    }

    fn leave_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, _arena: &AstArena) {
        if matches!(expr, Expr::Loop(_)) {
            self.loop_depth -= 1;
        }
    }

    fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
        if matches!(expr, Expr::Break(_)) && self.loop_depth == 0 {
            return vec![Diagnostic {
                level: crate::checker::DiagLevel::Error,
                kind: DiagnosticKind::LintWarning {
                    rule_name: "break_outside_loop".into(),
                    message: "break used outside of a loop".into(),
                },
                span,
                secondary: vec![],
                fix: None,
            }];
        }
        vec![]
    }
}
```

Note: `enter_expr` fires BEFORE `check_expr` in the same `on_expr` call (as wired in Task 1). So when we're at a `Loop` node, `enter_expr` increments depth, then `check_expr` runs (which won't match Break), then walker descends into children. When a `Break` is encountered inside, `enter_expr` does nothing (not Loop), `check_expr` sees Break with depth > 0, passes. When we leave the Loop, `leave_expr` decrements.

**IMPORTANT timing concern**: `enter_expr` for `Expr::Loop` fires at the Loop node itself. Then the walker descends into the Loop's children (the stmts). Break nodes inside will see `loop_depth == 1`. When the walker returns from the Loop's children, `leave_expr` fires and decrements. This is correct.

Register in `rules/mod.rs`: add `pub mod break_outside_loop;` and in `default_rules()` add `registry.register(Box::new(break_outside_loop::BreakOutsideLoop::new()));`.

**ActiveForm:** Implementing break_outside_loop lint rule

### Task 4: Rule — unreachable_code

**Subject:** Detect statements after break in a block

**Description:** Create `crates/lx/src/linter/rules/unreachable_code.rs`:

```rust
use crate::ast::{AstArena, Expr, ExprId, Stmt};
use crate::checker::Diagnostic;
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct UnreachableCode;

impl LintRule for UnreachableCode {
    fn name(&self) -> &'static str { "unreachable_code" }
    fn category(&self) -> RuleCategory { RuleCategory::Correctness }

    fn check_expr(&mut self, _id: ExprId, expr: &Expr, _span: SourceSpan, _model: &SemanticModel, arena: &AstArena) -> Vec<Diagnostic> {
        let stmts = match expr {
            Expr::Block(s) | Expr::Loop(s) => s,
            _ => return vec![],
        };

        let mut found_break = false;
        let mut diags = vec![];

        for &sid in stmts {
            let stmt_span = arena.stmt_span(sid);
            if found_break {
                diags.push(Diagnostic {
                    level: crate::checker::DiagLevel::Warning,
                    kind: DiagnosticKind::LintWarning {
                        rule_name: "unreachable_code".into(),
                        message: "unreachable code after break".into(),
                    },
                    span: stmt_span,
                    secondary: vec![],
                    fix: None,
                });
                break; // Only flag the first unreachable statement
            }

            let stmt = arena.stmt(sid);
            match stmt {
                Stmt::Expr(eid) => {
                    if matches!(arena.expr(*eid), Expr::Break(_)) {
                        found_break = true;
                    }
                },
                _ => {},
            }
        }

        diags
    }
}
```

This only detects code after `Expr::Break` in the same block. Does not analyze branches — keeps it simple.

Register in `rules/mod.rs`.

**ActiveForm:** Implementing unreachable_code lint rule

### Task 5: Rule — unused_import

**Subject:** Detect imports that are never referenced

**Description:** Create `crates/lx/src/linter/rules/unused_import.rs`:

```rust
use crate::ast::{AstArena, Stmt, StmtId, UseKind};
use crate::checker::Diagnostic;
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::{DefKind, SemanticModel};
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct UnusedImport;

impl LintRule for UnusedImport {
    fn name(&self) -> &'static str { "unused_import" }
    fn category(&self) -> RuleCategory { RuleCategory::Correctness }

    fn check_stmt(&mut self, _id: StmtId, stmt: &Stmt, span: SourceSpan, model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
        let use_stmt = match stmt {
            Stmt::Use(u) => u,
            _ => return vec![],
        };

        let mut diags = vec![];

        // Find all import definitions and check if they have references
        // Iterate model.definitions to find imports matching this Use statement's names
        let names_to_check: Vec<_> = match &use_stmt.kind {
            UseKind::Whole => {
                // Whole import: use std.foo — check if the module name is referenced
                use_stmt.path.last().map(|n| vec![*n]).unwrap_or_default()
            },
            UseKind::Alias(alias) => vec![*alias],
            UseKind::Selective(names) => names.clone(),
        };

        for name in &names_to_check {
            // Find the definition for this import name
            let def = model.definitions.iter().enumerate().find(|(_, d)| {
                matches!(d.kind, DefKind::Import) && d.name == *name && d.span == span
            });

            if let Some((def_id, _)) = def {
                let refs = model.references_to(def_id);
                if refs.is_empty() {
                    diags.push(Diagnostic {
                        level: crate::checker::DiagLevel::Warning,
                        kind: DiagnosticKind::LintWarning {
                            rule_name: "unused_import".into(),
                            message: format!("unused import '{}'", name),
                        },
                        span,
                        secondary: vec![],
                        fix: None,
                    });
                }
            }
        }

        diags
    }
}
```

Note: We match imports by both name AND span to handle cases where the same name is imported in different scopes. The `references_to(def_id)` method returns all ExprIds that reference this definition — if empty, the import is unused.

Register in `rules/mod.rs`.

**ActiveForm:** Implementing unused_import lint rule

### Task 6: Rule — duplicate_record_field

**Subject:** Detect duplicate field names in record literals

**Description:** Create `crates/lx/src/linter/rules/duplicate_record_field.rs`:

```rust
use std::collections::HashSet;

use crate::ast::{AstArena, Expr, ExprId, RecordField};
use crate::checker::Diagnostic;
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct DuplicateRecordField;

impl LintRule for DuplicateRecordField {
    fn name(&self) -> &'static str { "duplicate_record_field" }
    fn category(&self) -> RuleCategory { RuleCategory::Correctness }

    fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
        let fields = match expr {
            Expr::Record(fields) => fields,
            _ => return vec![],
        };

        let mut seen = HashSet::new();
        let mut diags = vec![];

        for field in fields {
            if let RecordField::Named { name, .. } = field {
                if !seen.insert(*name) {
                    diags.push(Diagnostic {
                        level: crate::checker::DiagLevel::Error,
                        kind: DiagnosticKind::LintWarning {
                            rule_name: "duplicate_record_field".into(),
                            message: format!("duplicate field '{}' in record literal", name),
                        },
                        span,
                        secondary: vec![],
                        fix: None,
                    });
                }
            }
        }

        diags
    }
}
```

Note: `RecordField::Named { name: Sym, value: ExprId }` — the `name` field is a `Sym`. `Sym` implements `Hash` and `Eq` (it's an interned string), so `HashSet::insert` works.

Register in `rules/mod.rs`.

**ActiveForm:** Implementing duplicate_record_field lint rule

### Task 7: Rule — single_branch_par

**Subject:** Detect par blocks with only one branch

**Description:** Create `crates/lx/src/linter/rules/single_branch_par.rs`:

```rust
use crate::ast::{AstArena, Expr, ExprId};
use crate::checker::Diagnostic;
use crate::checker::diagnostics::DiagnosticKind;
use crate::checker::semantic::SemanticModel;
use crate::linter::rule::{LintRule, RuleCategory};
use miette::SourceSpan;

pub struct SingleBranchPar;

impl LintRule for SingleBranchPar {
    fn name(&self) -> &'static str { "single_branch_par" }
    fn category(&self) -> RuleCategory { RuleCategory::Correctness }

    fn check_expr(&mut self, _id: ExprId, expr: &Expr, span: SourceSpan, _model: &SemanticModel, _arena: &AstArena) -> Vec<Diagnostic> {
        if let Expr::Par(stmts) = expr {
            if stmts.len() <= 1 {
                return vec![Diagnostic {
                    level: crate::checker::DiagLevel::Warning,
                    kind: DiagnosticKind::LintWarning {
                        rule_name: "single_branch_par".into(),
                        message: "par block with a single branch has no concurrency — use the expression directly".into(),
                    },
                    span,
                    secondary: vec![],
                    fix: None,
                }];
            }
        }
        vec![]
    }
}
```

Register in `rules/mod.rs`.

**ActiveForm:** Implementing single_branch_par lint rule

### Task 8: Rule — mut_never_mutated

**Subject:** Detect mutable bindings that are never mutated

**Description:** Create `crates/lx/src/linter/rules/mut_never_mutated.rs`:

This rule cannot use `check_stmt` alone because it needs to scan the entire program for mutations. Instead, it collects mutable bindings during the walk and checks at the end. Since `LintRule` fires per-node, we need a different strategy: collect all mutable binding definitions and all mutation sites, then compare.

Approach: The rule accumulates mutable bindings in `check_stmt` and accumulates mutation sites (FieldUpdates and Reassignments). Then in a final check, it reports unused mutables. But `LintRule` has no "finalize" hook.

Revised approach: Implement this check directly in the linter runner or as a post-lint pass. Add a function in `runner.rs`:

```rust
pub fn check_unused_mut(program: &Program<impl std::any::Any>, model: &SemanticModel, arena: &AstArena) -> Vec<Diagnostic> {
    // 1. Collect all definitions where mutable == true and kind == DefKind::Binding
    let mut_defs: Vec<(DefinitionId, &DefinitionInfo)> = model.definitions.iter().enumerate()
        .filter(|(_, d)| d.mutable && matches!(d.kind, DefKind::Binding))
        .collect();

    // 2. Collect all mutation targets: scan all stmts for FieldUpdate and Reassign
    let mut mutated_names: HashSet<Sym> = HashSet::new();
    for &sid in &program.stmts {
        collect_mutations(sid, arena, &mut mutated_names);
    }

    // 3. For each mutable def, if name not in mutated_names, emit warning
    let mut diags = vec![];
    for (_, def) in mut_defs {
        if !mutated_names.contains(&def.name) {
            diags.push(Diagnostic {
                level: DiagLevel::Warning,
                kind: DiagnosticKind::LintWarning {
                    rule_name: "mut_never_mutated".into(),
                    message: format!("binding '{}' declared as mut but never mutated", def.name),
                },
                span: def.span,
                secondary: vec![],
                fix: None,
            });
        }
    }
    diags
}

fn collect_mutations(sid: StmtId, arena: &AstArena, mutated: &mut HashSet<Sym>) {
    let stmt = arena.stmt(sid);
    match stmt {
        Stmt::FieldUpdate(fu) => { mutated.insert(fu.name); },
        Stmt::Binding(b) => {
            if let BindTarget::Reassign(name) = &b.target {
                mutated.insert(*name);
            }
            // Recurse into the value expression for nested blocks
            collect_mutations_expr(b.value, arena, mutated);
        },
        Stmt::Expr(eid) => {
            collect_mutations_expr(*eid, arena, mutated);
        },
        _ => {},
    }
}

fn collect_mutations_expr(eid: ExprId, arena: &AstArena, mutated: &mut HashSet<Sym>) {
    let expr = arena.expr(eid);
    match expr {
        Expr::Block(stmts) | Expr::Loop(stmts) | Expr::Par(stmts) => {
            for &sid in stmts { collect_mutations(sid, arena, mutated); }
        },
        // Add other compound expression variants that contain stmts
        Expr::With(w) => {
            for &sid in &w.body { collect_mutations(sid, arena, mutated); }
        },
        Expr::Match(m) => {
            for arm in &m.arms { collect_mutations_expr(arm.body, arena, mutated); }
        },
        Expr::Func(f) => {
            collect_mutations_expr(f.body, arena, mutated);
        },
        _ => {},
    }
}
```

Call `check_unused_mut` from the `lint` function in `runner.rs`, after the visitor walk, and append results.

Note: This scans by `Sym` name, not by `DefinitionId`. This means if a mutable `x` in scope A is never mutated, but a different `x` in scope B is mutated, the first `x` won't be flagged. This is a false negative for shadowed names — acceptable for a first pass. A more precise version would track by DefinitionId, but that requires matching FieldUpdate targets to specific definitions, which the semantic model doesn't support (FieldUpdate doesn't record which definition it targets).

**ActiveForm:** Implementing mut_never_mutated lint check

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
mcp__workflow__load_work_item({ path: "work_items/STRUCTURAL_LINT_RULES.md" })
```

Then call `next_task` to begin.
