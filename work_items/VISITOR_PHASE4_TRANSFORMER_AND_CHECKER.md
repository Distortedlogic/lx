# Goal

Add flat-map semantics to the statement transformer (enabling statement removal and expansion) using a `transform_stmts` method, and eliminate the checker's manual AST walk by having `Checker` implement `AstVisitor` while keeping scope/binding/type operations interleaved in the visit/leave hooks.

# Why

- `TransformOp` has `Continue`, `Replace`, and `Stop` but no way to remove a statement — dead code elimination, import cleanup, and similar transforms are impossible. rustc solves this with separate `flat_map_*` methods that return `SmallVec` without changing the per-node walk signature
- The checker in `checker/mod.rs` manually walks the AST via `check_program` → `check_stmt` → `check_expr`, duplicating the visitor traversal. The manual walk mirrors what `walk_program`/`dispatch_stmt`/`dispatch_expr` already do
- The original plan to split scope/binding tracking into a separate `SemanticIndexer` pass is architecturally unsound: `set_definition_type` is called with types determined during inference (`visit_stmt.rs`, `check_expr.rs`, `synth_compound.rs`, `synth_control.rs`, `infer_pattern.rs`), `resolve_in_scope`/`lookup_type` are used DURING inference (`type_ops.rs`) to look up types, and `push_scope`/`pop_scope` are interleaved with type-dependent operations across 5+ files. These operations cannot be separated into a pre-pass

# What changes

1. Add a `transform_stmts` method to `AstTransformer` with a default implementation that maps each `StmtId` through `walk_transform_stmt` — returns `Vec<StmtId>`. Transforms that need removal/expansion override this method
2. Update `walk_transform_program` to call `transform_stmts` instead of mapping individual stmts
3. Leave `walk_transform_stmt`'s return type as `StmtId` — the proc-macro-generated `recurse_children` code calls `walk_transform_stmt` and expects a single `StmtId` return, so changing it would break ALL generated code
4. Have `Checker` implement `AstVisitor` to eliminate its duplicated manual walk. The checker's `check_program` loop becomes `walk_program(self, program)`. Statement handling moves into `visit_stmt`/`leave_stmt`. Expression handling moves into `visit_expr`/`leave_expr`
5. Keep `SemanticModelBuilder` on `Checker` as it is today — scope/binding/type operations remain interleaved in the visit/leave hooks because they depend on types computed during inference

# How it works

For the flat-map transformer: `walk_transform_stmt` continues to return a single `StmtId`. A new `transform_stmts(&mut self, stmts: Vec<StmtId>, arena: &mut AstArena) -> Vec<StmtId>` method on `AstTransformer` provides the default mapping behavior: `stmts.into_iter().map(|s| walk_transform_stmt(self, s, arena)).collect()`. `walk_transform_program` calls `t.transform_stmts(stmts, &mut program.arena)` instead of doing the map inline. Transforms that need removal or expansion override `transform_stmts` to flat-map: call `walk_transform_stmt` for each stmt, then conditionally emit zero or multiple replacement stmts. This follows rustc's pattern and avoids touching the proc macro.

For the checker-as-visitor: `Checker` implements `AstVisitor` (and the required `PatternVisitor` + `TypeVisitor` supertraits). The `visit_stmt` hook dispatches to the existing `check_stmt` logic. The `visit_expr` hook dispatches to the existing `synth_expr`/`check_expr` logic. The `check_program` method calls `walk_program(self, program)` instead of manually iterating stmts. Scope push/pop, definition tracking, reference resolution, and type inference all remain on the `Checker` struct, interleaved in the hooks exactly as they are today. The only change is eliminating the manual walk loop — the visitor infrastructure handles traversal.

# Checker-as-visitor challenge: bidirectional type flow

The checker has bidirectional type flow — `check_expr` receives an `expected` type and checks against it, while `synth_expr` synthesizes a type bottom-up. The visitor's `visit_expr` hook does not receive an expected type. The solution: `visit_expr` calls `synth_expr` (bottom-up synthesis). Bidirectional checking in `check_func_against`, `check_list`, `check_match`, and `check_block` remains as hand-written match dispatch called from within `synth_expr`/`check_expr` — these methods already recursively call `synth_expr`/`check_expr` which will trigger the visitor hooks for child nodes. The visitor handles top-level traversal; the checker's internal methods handle type-directed recursion within expressions.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/visitor/transformer.rs` | Add `transform_stmts` method with default implementation |
| `crates/lx/src/visitor/walk_transform/mod.rs` | Update `walk_transform_program` to call `transform_stmts` |
| `crates/lx/src/checker/mod.rs` | Implement `AstVisitor` for `Checker`; replace manual `check_program` loop with `walk_program` call |
| `crates/lx/src/checker/visit_stmt.rs` | Adapt `check_stmt` to work within visitor hook context |
| `crates/lx/src/checker/check_expr.rs` | Adapt `check_expr`/`check_func_against`/`check_match`/`check_block` to work within visitor context |
| `crates/lx/src/checker/synth_compound.rs` | No change needed — `synth_func_type`, `synth_apply_type`, `synth_match_type` already call `self.sem.push_scope`/`pop_scope`/`add_definition`/`set_definition_type` inline, and this stays as-is |
| `crates/lx/src/checker/synth_control.rs` | No change needed — `synth_with_type` already calls `self.sem.push_scope`/`pop_scope`/`add_definition`/`set_definition_type` inline, and this stays as-is |
| `crates/lx/src/checker/type_ops.rs` | No change needed — `synth_expr` calls `self.sem.resolve_in_scope`/`add_reference` inline, and this stays as-is |
| `crates/lx/src/checker/infer_pattern.rs` | No change needed — `infer_pattern_bindings` calls `self.sem.add_definition`/`set_definition_type` inline, and this stays as-is |
| `crates/lx/src/checker/generics.rs` | No change needed — `push_generic_scope`/`pop_generic_scope` operate on `Checker`'s own `generic_scope` stack, not on `sem` |
| `crates/lx/src/folder/desugar.rs` | No change needed — `Desugarer` only overrides `leave_expr`, not `transform_stmts` |

# Task List

### Task 1: Add transform_stmts to AstTransformer

In `crates/lx/src/visitor/transformer.rs`:

Add a `transform_stmts` method to the `AstTransformer` trait with this signature and default implementation:

```
fn transform_stmts(&mut self, stmts: Vec<StmtId>, arena: &mut AstArena) -> Vec<StmtId> {
    stmts.into_iter().map(|s| super::walk_transform::walk_transform_stmt(self, s, arena)).collect()
}
```

This requires adding `AstArena` and `StmtId` to the imports if not already present (they are already imported).

### Task 2: Update walk_transform_program to use transform_stmts

In `crates/lx/src/visitor/walk_transform/mod.rs`:

Change `walk_transform_program` from:

```
let stmts: Vec<StmtId> = program.stmts.clone();
let folded: Vec<StmtId> = stmts.into_iter().map(|s| walk_transform_stmt(t, s, &mut program.arena)).collect();
```

to:

```
let stmts: Vec<StmtId> = program.stmts.clone();
let folded: Vec<StmtId> = t.transform_stmts(stmts, &mut program.arena);
```

### Task 3: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add flat-map transform_stmts method to AstTransformer for statement removal and expansion"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 5: Implement AstVisitor for Checker

In `crates/lx/src/checker/mod.rs`:

Add the necessary imports: `use crate::visitor::{AstVisitor, PatternVisitor, TypeVisitor, VisitAction};` and `use crate::ast::{Stmt, StmtId};` (check which are already imported and only add missing ones).

Implement `PatternVisitor` for `Checker<'_>` as an empty impl (all default methods).

Implement `TypeVisitor` for `Checker<'_>` as an empty impl (all default methods).

Implement `AstVisitor` for `Checker<'_>` with these hooks:

In `visit_stmt`: return `VisitAction::Skip` always. This tells the visitor to call `leave_stmt` but NOT to descend into children automatically — the checker's own `check_stmt` handles child traversal with type-directed logic.

In `leave_stmt`: call `self.check_stmt(id, self.arena)` where `id` is the `StmtId` parameter. This requires `self.arena` to be available. Since `check_stmt` takes `&AstArena` and `self.arena` is `&'a AstArena`, extract `self.arena` into a local: `let arena = self.arena; self.check_stmt(id, arena);`.

Change `check_program` from:

```
fn check_program(&mut self, program: &Program<Core>) {
    for &sid in &program.stmts {
        self.check_stmt(sid, &program.arena);
    }
}
```

to:

```
fn check_program(&mut self, program: &Program<Core>) {
    crate::visitor::walk_program(self, program);
}
```

Since the visitor's `walk_program` iterates `program.stmts` and calls `dispatch_stmt` for each, and `dispatch_stmt` calls `visit_stmt` (which returns `Skip`), then calls `leave_stmt` (which calls `check_stmt`), this produces the same behavior as the manual loop.

### Task 6: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 7: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: make Checker implement AstVisitor to eliminate manual AST walk"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 8: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 9: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 10: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 11: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 12: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/VISITOR_PHASE4_TRANSFORMER_AND_CHECKER.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

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
