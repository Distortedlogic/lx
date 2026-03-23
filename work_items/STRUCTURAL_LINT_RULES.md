# Goal

Add structural lint rules to the existing linter framework and wire the linter into `lx check` so lint warnings appear alongside type errors in the same diagnostic output. The linter framework (registry, runner, matcher, rule trait) already exists with 2 rules — this work item adds correctness-focused rules that catch mistakes the type checker can't see, and integrates the linter into the check pipeline.

# Why

LLM agents writing lx programs make structural mistakes that pass the type checker but fail at runtime or produce wrong results: `break` outside a loop, unreachable code after a return, importing names they never use, duplicate fields in records. These are exactly the mistakes an agent makes when it's pattern-matching against languages in its training data (Python, Rust, TS) and guessing at lx semantics. The type checker is structurally unable to catch these — they require a separate validation pass.

The linter framework is ready: `LintRule` trait with `check_expr`/`check_stmt`/`check_pattern` hooks, `RuleRegistry` for collection, `LintRunner` that implements `AstVisitor` and dispatches to all rules. The runner produces `Vec<Diagnostic>` using the same diagnostic type as the checker. Just needs rules and integration.

# What changes

**Modified `crates/lx/src/checker/mod.rs`:** After type checking completes, run the linter on the same program and append lint diagnostics to the checker's diagnostic list.

**New rule files in `crates/lx/src/linter/rules/`:** One file per rule — `break_outside_loop.rs`, `unreachable_code.rs`, `unused_import.rs`, `duplicate_record_field.rs`, `single_branch_par.rs`, `mut_never_mutated.rs`.

**Modified `crates/lx/src/linter/rules/mod.rs`:** Register new rules in the module and in `RuleRegistry::default_rules()`.

# Files affected

- EDIT: `crates/lx/src/checker/mod.rs` — call linter after type checking, append diagnostics
- NEW: `crates/lx/src/linter/rules/break_outside_loop.rs`
- NEW: `crates/lx/src/linter/rules/unreachable_code.rs`
- NEW: `crates/lx/src/linter/rules/unused_import.rs`
- NEW: `crates/lx/src/linter/rules/duplicate_record_field.rs`
- NEW: `crates/lx/src/linter/rules/single_branch_par.rs`
- NEW: `crates/lx/src/linter/rules/mut_never_mutated.rs`
- EDIT: `crates/lx/src/linter/rules/mod.rs` — declare new rule modules, register in default_rules()
- EDIT: `crates/lx/src/linter/registry.rs` — if needed, register new rules in default_rules()

# Task List

### Task 1: Wire linter into the check pipeline

**Subject:** Run linter after type checking and merge diagnostics

**Description:** In `crates/lx/src/checker/mod.rs`, in the `check` and `check_with_imports` functions (or in the internal `check_program` method — wherever the `CheckResult` is assembled):

After type checking is complete and the `SemanticModel` has been built:
1. Create a `lx::linter::RuleRegistry::default_rules()`
2. Call `lx::linter::lint(&program, &semantic_model, &mut registry)` — this returns `Vec<Diagnostic>`
3. Append the lint diagnostics to `self.diagnostics` (or to the CheckResult's diagnostics vec)
4. The lint diagnostics will automatically appear in `lx check` output because `check.rs` in the CLI already iterates all diagnostics

Verify that `linter::lint` takes `&Program<Core>` (since by this point the program has been desugared). Check the generic parameter `P` on `lint<P>` — the function signature is `pub fn lint<P>(program: &Program<P>, model: &SemanticModel, registry: &mut RuleRegistry) -> Vec<Diagnostic>`. Since it's generic over P, it works on Core.

Also verify that the linter's `Diagnostic` type is the same as the checker's — both should be `crate::checker::diagnostics::Diagnostic`. If the linter uses a different path, reconcile the imports.

**ActiveForm:** Wiring linter into check pipeline

### Task 2: Rule — break_outside_loop

**Subject:** Detect break/continue expressions outside of loop context

**Description:** Create `crates/lx/src/linter/rules/break_outside_loop.rs`:

```rust
pub struct BreakOutsideLoop {
    loop_depth: usize,
}
```

Implement `LintRule`:
- `name()` → `"break_outside_loop"`
- `category()` → `RuleCategory::Correctness`
- `check_expr()`:
  - When entering `Expr::Loop { .. }`: increment `self.loop_depth`. But wait — `check_expr` is called per-expression, not enter/leave. The LintRule trait only has `check_expr` which fires once per expression, not enter/leave hooks.

  Revised approach: Since the LintRule trait doesn't have enter/leave hooks, and the runner calls `check_expr` once per expression during the visitor walk, the rule needs to track state differently. Use the `AstArena` to look upward — but there's no parent map by default.

  Simpler approach: Use the `ExprMatcher` or just check if the current expression is `Expr::Break` and then walk the arena's parent map (if available via `SemanticModel`) to see if there's an enclosing loop. If no parent map is available, add a `loop_depth: usize` field and note that the LintRunner's `AstVisitor` calls `on_expr`/`leave_expr` — but LintRule only gets `check_expr`.

  Best approach: Instead of implementing this as a `LintRule`, implement it directly in the `LintRunner` by extending the runner to track loop depth. In `runner.rs`, add a `loop_depth: usize` field to `LintRunner`. In the `on_expr` implementation, when `Expr::Loop` is encountered, increment `loop_depth`. In `leave_expr`, when `Expr::Loop` is encountered, decrement `loop_depth`. In `on_expr`, when `Expr::Break` is encountered and `loop_depth == 0`, emit a diagnostic directly.

  Actually, re-reading the runner: `LintRunner` implements `AstVisitor` and has `on_expr` and `leave_expr` (via `on_stmt`). But the current `on_expr` only dispatches to rules. We can add structural checks directly in the runner.

  Alternative: Make `BreakOutsideLoop` stateful. The runner calls `check_expr` for every expression in tree order. Track loop depth:
  - On `Expr::Loop`: `self.loop_depth += 1; return vec![]`
  - On `Expr::Break`: if `self.loop_depth == 0`, emit diagnostic
  - Problem: there's no "leave" call to decrement. The check_expr hook fires once per expression, in pre-order.

  Final approach: Extend `LintRule` trait with optional `enter_expr` and `leave_expr` hooks, defaulting to no-op. Update `LintRunner` to call `enter_expr` on enter and `leave_expr` on leave. Then `BreakOutsideLoop` uses enter/leave to track loop depth.

  Add to `LintRule` trait in `rule.rs`:
  ```rust
  fn enter_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) {}
  fn leave_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena) {}
  ```

  In `LintRunner`'s `AstVisitor::on_expr`, call `rule.enter_expr()` for each rule. Add a `leave_expr` implementation to `LintRunner`'s `AstVisitor` impl that calls `rule.leave_expr()` for each rule.

  Then in `BreakOutsideLoop`:
  - `enter_expr`: if `Expr::Loop`, increment depth
  - `leave_expr`: if `Expr::Loop`, decrement depth
  - `check_expr`: if `Expr::Break` and `loop_depth == 0`, emit `LintWarning { rule_name: "break_outside_loop", message: "break used outside of a loop" }`

Register in `rules/mod.rs` and in `RuleRegistry::default_rules()`.

**ActiveForm:** Implementing break_outside_loop lint rule

### Task 3: Rule — unreachable_code

**Subject:** Detect statements after break in a block

**Description:** Create `crates/lx/src/linter/rules/unreachable_code.rs`:

```rust
pub struct UnreachableCode;
```

Implement `LintRule`:
- `name()` → `"unreachable_code"`
- `category()` → `RuleCategory::Correctness`
- `check_expr()`:
  - Match on `Expr::Block(stmts)` (check the Block variant in the AST to get the correct field name)
  - Iterate through the block's statements
  - Track whether a previous statement's expression was `Expr::Break`
  - If a statement follows a break statement, emit diagnostic on the following statement's span: `"unreachable code after break"`
  - Check both `Expr::Break` and any expression that unconditionally returns/breaks in all branches

Keep it simple for the first pass: only detect code after `Expr::Break` in the same block. Don't try to analyze branches.

Register in `rules/mod.rs` and `default_rules()`.

**ActiveForm:** Implementing unreachable_code lint rule

### Task 4: Rule — unused_import

**Subject:** Detect imports that are never referenced

**Description:** Create `crates/lx/src/linter/rules/unused_import.rs`:

```rust
pub struct UnusedImport;
```

Implement `LintRule`:
- `name()` → `"unused_import"`
- `category()` → `RuleCategory::Correctness`
- `check_stmt()`:
  - Match on `Stmt::Use(use_stmt)` — check the exact Use variant shape in the AST
  - For each imported name in the use statement, check if it appears in `model.references`
  - Specifically: the SemanticModel has `definitions: Vec<DefinitionInfo>` where imported names have `kind: DefKind::Import`. Check if any `Reference` in `model.references` points to that definition.
  - If an imported name has no references, emit: `LintWarning { rule_name: "unused_import", message: "unused import 'foo'" }`
  - Include a `Fix` with `Applicability::MachineApplicable` that removes the import (the TextEdit span covers the entire import statement if it's the only name, or just the unused name if it's a selective import with multiple names)

Note: Check the `SemanticModel` API to see if references track which definition they resolve to. If `Reference` has a `definition: DefinitionId` field, match against the import's definition. If not, fall back to name matching — check if any `Reference` has the same `name` as the import.

Register in `rules/mod.rs` and `default_rules()`.

**ActiveForm:** Implementing unused_import lint rule

### Task 5: Rule — duplicate_record_field

**Subject:** Detect duplicate field names in record literals

**Description:** Create `crates/lx/src/linter/rules/duplicate_record_field.rs`:

```rust
pub struct DuplicateRecordField;
```

Implement `LintRule`:
- `name()` → `"duplicate_record_field"`
- `category()` → `RuleCategory::Correctness`
- `check_expr()`:
  - Match on `Expr::Record(fields)` — check the exact Record variant in the AST (it may be `Expr::Record { fields }` or similar)
  - Collect field names into a `HashSet`
  - If inserting a name returns false (already present), emit: `LintWarning { rule_name: "duplicate_record_field", message: "duplicate field 'name' in record literal" }` with the span of the duplicate field

Register in `rules/mod.rs` and `default_rules()`.

**ActiveForm:** Implementing duplicate_record_field lint rule

### Task 6: Rule — single_branch_par

**Subject:** Detect par blocks with only one branch

**Description:** Create `crates/lx/src/linter/rules/single_branch_par.rs`:

```rust
pub struct SingleBranchPar;
```

Implement `LintRule`:
- `name()` → `"single_branch_par"`
- `category()` → `RuleCategory::Correctness`
- `check_expr()`:
  - Match on `Expr::Par(branches)` — check the exact Par variant in the AST
  - If branches length is 1, emit: `LintWarning { rule_name: "single_branch_par", message: "par block with a single branch has no concurrency — use the expression directly" }`

Register in `rules/mod.rs` and `default_rules()`.

**ActiveForm:** Implementing single_branch_par lint rule

### Task 7: Rule — mut_never_mutated

**Subject:** Detect mutable bindings that are never mutated

**Description:** Create `crates/lx/src/linter/rules/mut_never_mutated.rs`:

```rust
pub struct MutNeverMutated;
```

Implement `LintRule`:
- `name()` → `"mut_never_mutated"`
- `category()` → `RuleCategory::Style`
- `check_stmt()`:
  - Match on `Stmt::Binding(binding)` — check the exact Binding variant
  - If `binding.mutable` is true:
    - Look up the binding name in `model.definitions` to find its `DefinitionId`
    - Search `model.references` for any reference to this definition that is a write/mutation (check if the `Reference` type has a `kind` or `is_write` field)
    - If no write references found, also scan the program's statements for `Stmt::FieldUpdate` targeting this binding name (field updates are a form of mutation in lx)
    - If truly never mutated: emit `LintWarning { rule_name: "mut_never_mutated", message: "binding 'x' declared as mut but never mutated" }`
    - Include a `Fix` with `Applicability::MachineApplicable` — remove the `mut` keyword from the binding

Note: The SemanticModel may not track write vs read references. If `Reference` doesn't distinguish reads from writes, fall back to scanning for `Stmt::FieldUpdate` where the target matches the binding name. Check the `Reference` struct definition to determine what's available.

Register in `rules/mod.rs` and `default_rules()`.

**ActiveForm:** Implementing mut_never_mutated lint rule

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
