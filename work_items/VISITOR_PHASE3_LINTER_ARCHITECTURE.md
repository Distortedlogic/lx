# Goal

Prerequisite: Phase 2 (VISITOR_PHASE2_INFRASTRUCTURE_REBUILD) must be completed first. AstVisitor is a single merged trait with no PatternVisitor/TypeVisitor separation.

Restructure the linter so that rules receive `&Program<Core>` instead of raw `(&[StmtId], &AstArena)`, all rules execute in a single AST walk instead of one walk per rule, and the `UnreachableCode` rule uses the visitor infrastructure instead of reimplementing the entire AST traversal.

# Why

- `LintRule::run` takes `(&[StmtId], &AstArena)` forcing every rule to manually loop stmts and call `dispatch_stmt` — boilerplate that defeats the visitor infrastructure
- The runner loops over rules and calls `rule.run()` for each, causing O(rules x AST) traversal. With 7 rules, that is 7 full AST walks
- `UnreachableCode` implements its own `walk_stmts`, `walk_stmt`, `walk_expr` covering ~15 Expr variants — a parallel traversal that will fall out of sync when AST nodes change
- `UnusedImport` implements `AstVisitor` but never uses it — it manually iterates stmts in `run()`

# What changes

1. Change `LintRule::run` signature to receive `&Program<Core>` and `&SemanticModel`
2. Add a `LintRule::check_expr` and `LintRule::check_stmt` method pair with default no-op implementations, called by a single-walk dispatcher
3. Create a `LintWalker` struct that implements `AstVisitor`, holds `&mut [Box<dyn LintRule>]`, and forwards each visited node to every rule's `check_expr`/`check_stmt`
4. Update `runner::lint` to create `LintWalker` and do one `walk_program` call
5. Rewrite `UnreachableCode` to use `check_expr` hooks — scanning sibling statements via `visit_block`/`visit_loop`/`visit_par` hooks on the `ExprBlock`/`ExprLoop`/`ExprPar` structs
6. Simplify `UnusedImport` to use its `run` method directly (it only needs top-level stmts, no walk)

# How it works

The `LintWalker` is the single visitor that walks the program once. At each expression node, it calls `rule.check_expr(id, expr, span, arena, model)` on every registered rule. At each statement node, it calls `rule.check_stmt(id, stmt, span, arena, model)`. Rules that only care about specific node types match in their `check_expr`/`check_stmt` and ignore the rest.

Rules that need pre/post-order behavior (like `BreakOutsideLoop` tracking loop depth) implement both `check_expr` (pre-order, called before children) and a new `leave_expr` hook (post-order, called after children). The `LintWalker` calls these at the appropriate times during its visitor walk.

`UnreachableCode` no longer needs its own walk. It receives `ExprBlock`/`ExprLoop`/`ExprPar` via `check_expr`, reads the `stmts` field to scan for breaks followed by unreachable statements, and reports diagnostics. The visitor handles recursion into child nodes automatically.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/linter/rule.rs` | Rework `LintRule` trait: add `check_expr`, `check_stmt`, `leave_expr` with defaults; change `run` to receive `&Program<Core>` |
| `crates/lx/src/linter/runner.rs` | Create `LintWalker` implementing `AstVisitor`; rewrite `lint()` to do single walk |
| `crates/lx/src/linter/mod.rs` | Update re-exports |
| `crates/lx/src/linter/rules/break_outside_loop.rs` | Migrate from `AstVisitor` impl to `check_expr`/`leave_expr` on `LintRule` |
| `crates/lx/src/linter/rules/unreachable_code.rs` | Delete hand-rolled walk; implement `check_expr` scanning block/loop/par stmts |
| `crates/lx/src/linter/rules/empty_match.rs` | Migrate to `check_expr` |
| `crates/lx/src/linter/rules/redundant_propagate.rs` | Collapse two-phase collect-then-analyze into single-phase inline `check_expr`; eliminate `candidates` field and post-walk loop |
| `crates/lx/src/linter/rules/duplicate_record_field.rs` | Migrate to `check_expr` |
| `crates/lx/src/linter/rules/single_branch_par.rs` | Migrate to `check_expr` |
| `crates/lx/src/linter/rules/unused_import.rs` | Simplify — just use `run` with `program.stmts` iteration |
| `crates/lx/src/linter/rules/mut_never_mutated.rs` | No changes needed — standalone function called directly from runner, not a LintRule |

# Task List

### Task 1: Rework LintRule trait

In `crates/lx/src/linter/rule.rs`:

Add imports for `ExprId`, `Expr`, `StmtId`, `Stmt`, `AstArena`, `Program`, `Core`. Add `use miette::SourceSpan;`.

Keep existing methods `name`, `code`, `category`, `take_diagnostics`.

Change `run` signature from `fn run(&mut self, stmts: &[StmtId], arena: &AstArena, model: &SemanticModel)` to `fn run(&mut self, program: &Program<Core>, model: &SemanticModel)`. Add a default implementation that does nothing (rules that need full-program access override this).

Add new methods with default no-op implementations:
- `fn check_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {}`
- `fn leave_expr(&mut self, _id: ExprId, _expr: &Expr, _span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {}`
- `fn check_stmt(&mut self, _id: StmtId, _stmt: &Stmt, _span: SourceSpan, _arena: &AstArena, _model: &SemanticModel) {}`

### Task 2: Create LintWalker and rewrite runner

In `crates/lx/src/linter/runner.rs`:

Replace the current `lint` function body. Create a `LintWalker` struct that holds `rules: &mut Vec<Box<dyn LintRule>>`, `arena: &AstArena`, and `model: &SemanticModel`.

Implement `AstVisitor` for `LintWalker` — the defaults handle all pattern/type hooks. In `visit_expr`, call `rule.check_expr(id, expr, span, self.arena, self.model)` for each rule. In `leave_expr`, call `rule.leave_expr(id, expr, span, self.arena, self.model)` for each rule. In `visit_stmt`, call `rule.check_stmt(id, stmt, span, self.arena, self.model)` for each rule. All visit methods return `VisitAction::Descend`.

In the `lint` function: first call `rule.run(program, model)` for each rule (for rules that need top-level access). Then create a `LintWalker` and call `walk_program(&mut walker, program)`. Finally collect diagnostics from all rules. Keep the `check_unused_mut` call as-is.

### Task 3: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: rework LintRule trait and add single-walk LintWalker"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 5: Migrate BreakOutsideLoop to new trait

In `crates/lx/src/linter/rules/break_outside_loop.rs`:

Remove `impl AstVisitor for BreakOutsideLoop` and its `visit_expr`/`leave_expr` methods entirely. Remove the `AstVisitor`/`VisitAction`/`dispatch_stmt` imports.

Move the logic from the old `visit_expr` into `check_expr` on the `LintRule` impl. Move the logic from the old `leave_expr` into `leave_expr` on the `LintRule` impl. The logic is identical — increment `loop_depth` on `Expr::Loop`, check `Expr::Break` when depth is 0, decrement on leave.

In the `LintRule` impl, change `run` to have an empty body (or remove the override entirely if the default is empty). The `check_expr`/`leave_expr` hooks handle everything via the `LintWalker`.

Remove the `dispatch_stmt` import and the manual stmt loop from `run`.

### Task 6: Migrate remaining expression-based rules

Migrate the following rules from their current approach to `check_expr` on `LintRule`:

In `empty_match.rs`: read the current implementation, move the match-detection logic into `check_expr`. The rule checks for `Expr::Match` with zero arms. Remove the `impl AstVisitor` block and visitor-related imports. Remove the manual stmt dispatch loop from `run`.

In `redundant_propagate.rs`: the current implementation uses a two-phase collect-then-analyze pattern: it collects `Expr::Propagate` candidates into a `Vec` during the `AstVisitor` walk, then analyzes their types in a post-walk loop in `run()`. Collapse this into a single-phase inline check in `check_expr`, since `&SemanticModel` is available as a parameter. Match on `Expr::Propagate`, look up the inner expression's type via `model.type_of_expr` immediately, and push the diagnostic inline. Eliminate the `candidates: Vec<(ExprId, SourceSpan)>` field from the struct and the post-walk analysis loop in `run()`. Remove the `impl AstVisitor` block and visitor-related imports. Remove the manual stmt dispatch loop from `run`.

In `duplicate_record_field.rs`: move record-field-duplication detection into `check_expr`. Remove the `impl AstVisitor` block and visitor-related imports. Remove the manual stmt dispatch loop from `run`.

In `single_branch_par.rs`: move single-branch par detection into `check_expr`. Remove the `impl AstVisitor` block and visitor-related imports. Remove the manual stmt dispatch loop from `run`.

### Task 7: Rewrite UnreachableCode to use check_expr

In `crates/lx/src/linter/rules/unreachable_code.rs`:

Delete all of: `walk_stmts`, `walk_stmt`, `walk_expr` methods. These are the hand-rolled walk that duplicates the visitor.

Implement `check_expr` on the `LintRule` impl. In `check_expr`, match on expressions that contain statement lists: `Expr::Block(b)` scans `b.stmts`, `Expr::Loop(l)` scans `l.stmts`, `Expr::Par(p)` scans `p.stmts`. For each statement list, iterate pairs of adjacent statements. If statement N is `Stmt::Expr(eid)` where the expr is `Expr::Break(_)`, flag statement N+1 as unreachable.

Note: scanning `Expr::Par` stmts for break-then-unreachable is a behavior ADDITION. The current implementation does NOT scan Par stmts for this pattern — it only recurses into them via `walk_stmt`. This is intentional: Par branches can independently contain unreachable code after breaks.

The visitor handles recursion into child nodes automatically — the rule only needs to inspect sibling relationships within statement lists.

Remove the manual stmt iteration from `run` (replace with empty body or remove the override).

### Task 8: Simplify UnusedImport

In `crates/lx/src/linter/rules/unused_import.rs`:

Remove the `impl AstVisitor for UnusedImport` block (it was already empty). Remove the `AstVisitor`/`PatternVisitor`/`TypeVisitor` imports.

Move the existing logic from `run` to use the new signature: access `program.stmts` and `program.arena` from the `&Program<Core>` parameter. The logic is the same — iterate top-level stmts looking for `Stmt::Use`.

This rule does not need `check_expr` or `check_stmt` — it only inspects top-level imports via `run`.

### Task 9: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 10: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: migrate all lint rules to single-walk LintRule trait"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 11: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 12: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 13: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 14: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 15: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/VISITOR_PHASE3_LINTER_ARCHITECTURE.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **NEVER run raw cargo commands.** No `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`. ALWAYS use `just fmt`, `just test`, `just diagnose`, `just fix`. The `just` recipes include additional steps that raw `cargo` skips.
2. **Between implementation tasks, ONLY run `just fmt` + `git commit`.** Do NOT run `just test`, `just diagnose`, or any compilation/verification command between implementation tasks. These are expensive and only belong in the FINAL VERIFICATION section at the end.
3. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
4. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
5. **`just fmt` is not `cargo fmt`.** `just fmt` runs `dx fmt` + `cargo fmt` + `eclint`. Running `cargo fmt` alone is wrong.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.

## Task Loading Instructions

To execute this work item, read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
- `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
- `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded
- `activeForm`: A present-continuous form of the subject

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execute tasks strictly in order. Run commands EXACTLY as written. Do not substitute `cargo` for `just`. Do not run any command not specified in the current task. Do not "pre-check" compilation between implementation tasks. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands.
