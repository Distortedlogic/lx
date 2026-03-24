# Goal

Wrap the six inline Expr variants (`Block`, `Tuple`, `Loop`, `Par`, `Propagate`, `Break`) in named structs with `#[derive(AstWalk)]`, making every Expr variant uniformly hold a named struct. This enables uniform walk function generation in Phase 2 and aligns with the ruff pattern where every AST enum variant wraps a dedicated type.

# Why

- Six Expr variants use inline types (`Block(Vec<StmtId>)`, `Tuple(Vec<ExprId>)`, etc.) while the other twenty use named structs (`Binary(ExprBinary)`, `Func(ExprFunc)`, etc.)
- The inline variants force hand-written walk functions with per-variant logic (manual `for` loops, `if let` checks) while struct variants use the uniform `node.walk_children(v, arena)` pattern
- Visitor hook signatures are inconsistent — some receive struct references (`&ExprBinary`), others receive raw slices (`&[StmtId]`) or bare IDs (`ExprId`)
- The `AstWalk` derive macro cannot generate walk/dispatch functions for inline variants because they lack the `walk_children` method

# What changes

Six new structs in `expr_types.rs`, each with `#[derive(Debug, Clone, PartialEq, AstWalk)]`:

- `ExprBlock { pub stmts: Vec<StmtId> }` — replaces `Block(Vec<StmtId>)`
- `ExprTuple { pub elems: Vec<ExprId> }` — replaces `Tuple(Vec<ExprId>)`
- `ExprLoop { pub stmts: Vec<StmtId> }` — replaces `Loop(Vec<StmtId>)`
- `ExprPar { pub stmts: Vec<StmtId> }` — replaces `Par(Vec<StmtId>)`
- `ExprPropagate { pub inner: ExprId }` — replaces `Propagate(ExprId)`
- `ExprBreak { pub value: Option<ExprId> }` — replaces `Break(Option<ExprId>)`

The Expr enum changes from `Block(Vec<StmtId>)` to `Block(ExprBlock)`, etc. Every match site across the codebase updates to destructure through the new struct.

Visitor hooks change: `visit_block(&self, id, &[StmtId], span)` becomes `visit_block(&self, id, &ExprBlock, span)`, etc.

Walk functions simplify: all six can now call `node.walk_children(v, arena)` instead of hand-written loops.

# How it works

The change is purely structural — no behavioral change. The new structs are transparent wrappers. At every match site, `Expr::Block(stmts)` becomes `Expr::Block(ExprBlock { stmts })` or `Expr::Block(b)` where `b.stmts` is used. Construction sites change from `Expr::Block(vec![...])` to `Expr::Block(ExprBlock { stmts: vec![...] })`.

The AstWalk derive macro already handles structs with Vec and Option ID fields correctly, so the generated `walk_children`, `children`, and `recurse_children` methods work automatically for all six new types.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/ast/expr_types.rs` | Add ExprBlock, ExprTuple, ExprLoop, ExprPar, ExprPropagate, ExprBreak structs |
| `crates/lx/src/ast/mod.rs` | Update Expr enum variants to use new structs; add to pub use exports |
| `crates/lx/src/visitor/visitor_trait.rs` | Update hook signatures for block, tuple, loop, par, propagate, break |
| `crates/lx/src/visitor/walk/mod.rs` | Update walk_expr match arms and walk_binding |
| `crates/lx/src/visitor/walk/walk_expr.rs` | Update walk_block, walk_tuple signatures and impls |
| `crates/lx/src/visitor/walk/walk_expr2.rs` | Update walk_loop, walk_par, walk_propagate, walk_break; remove hand-rolled dispatch for propagate and break |
| `crates/lx/src/parser/expr.rs` | Update Expr::Loop, Expr::Par, Expr::Break construction sites |
| `crates/lx/src/parser/expr_pratt.rs` | Update Expr::Propagate construction site |
| `crates/lx/src/parser/expr_compound.rs` | Update Expr::Tuple construction site |
| `crates/lx/src/parser/expr_helpers.rs` | Update Expr::Block construction site |
| `crates/lx/src/formatter/emit_expr.rs` | Update match arms for all six variants |
| `crates/lx/src/interpreter/mod.rs` | Update match arms for Block, Tuple, Loop, Par, Propagate, Break |
| `crates/lx/src/checker/check_expr.rs` | Update match arms for all six variants |
| `crates/lx/src/checker/type_ops.rs` | Update match arms for all six variants |
| `crates/lx/src/checker/capture.rs` | Update visitor hook implementations: visit_block, visit_loop, visit_par signatures change from `&[StmtId]` to new struct references |
| `crates/lx/src/folder/desugar.rs` | Update Expr::Block construction site |
| `crates/lx/src/stdlib/diag/diag_helpers.rs` | Update Expr::Propagate match in unwrap_propagate |
| `crates/lx/src/stdlib/diag/diag_walk.rs` | Update visitor hook implementations (visit_par, leave_par, visit_loop, leave_loop) and walk function calls (walk_par, walk_loop) to use new struct signatures |
| `crates/lx/src/linter/rules/unreachable_code.rs` | Update matches on Block, Loop, Par, Break, Propagate, Tuple |
| `crates/lx/src/linter/rules/mut_never_mutated.rs` | Update matches on Block, Loop, Par |
| `crates/lx/src/linter/rules/single_branch_par.rs` | Update match on Expr::Par |
| `crates/lx/src/linter/rules/redundant_propagate.rs` | Update match on Expr::Propagate |
| `crates/lx/src/linter/rules/break_outside_loop.rs` | Update matches on Expr::Loop, Expr::Break |
| `crates/lx/src/linter/matcher.rs` | Update matches on Expr::Propagate, Expr::Block |

# Task List

### Task 1: Define new wrapper structs

In `crates/lx/src/ast/expr_types.rs`, add six new structs. Each must have `#[derive(Debug, Clone, PartialEq, AstWalk)]` and public fields:

- `ExprBlock` with field `pub stmts: Vec<StmtId>`
- `ExprTuple` with field `pub elems: Vec<ExprId>`
- `ExprLoop` with field `pub stmts: Vec<StmtId>`
- `ExprPar` with field `pub stmts: Vec<StmtId>`
- `ExprPropagate` with field `pub inner: ExprId`
- `ExprBreak` with field `pub value: Option<ExprId>`

Add the necessary imports (`StmtId`, `ExprId`) if not already present. Ensure `lx_macros::AstWalk` is imported.

### Task 2: Update Expr enum to use new structs

In `crates/lx/src/ast/mod.rs`, change the six Expr variants:

- `Block(Vec<StmtId>)` → `Block(ExprBlock)`
- `Tuple(Vec<ExprId>)` → `Tuple(ExprTuple)`
- `Loop(Vec<StmtId>)` → `Loop(ExprLoop)`
- `Par(Vec<StmtId>)` → `Par(ExprPar)`
- `Propagate(ExprId)` → `Propagate(ExprPropagate)`
- `Break(Option<ExprId>)` → `Break(ExprBreak)`

Ensure the new struct types are available via the `pub use expr_types::*` re-export already in mod.rs.

### Task 3: Update visitor trait hook signatures

In `crates/lx/src/visitor/visitor_trait.rs`, update these method signatures and their default implementations:

- `visit_block` and `leave_block`: change `_stmts: &[StmtId]` parameter to `_block: &ExprBlock`
- `visit_tuple` and `leave_tuple`: change `_elems: &[ExprId]` parameter to `_tuple: &ExprTuple`
- `visit_loop` and `leave_loop`: change `_stmts: &[StmtId]` parameter to `_loop: &ExprLoop`
- `visit_par` and `leave_par`: change `_stmts: &[StmtId]` parameter to `_par: &ExprPar`
- `visit_propagate` and `leave_propagate`: change `_inner: ExprId` parameter to `_propagate: &ExprPropagate`
- `visit_break` and `leave_break`: change `_value: Option<ExprId>` parameter to `_brk: &ExprBreak`

Add imports for all six new types.

### Task 4: Update walk/mod.rs dispatch and walk_expr

In `crates/lx/src/visitor/walk/mod.rs`:

In `walk_expr`, update the six match arms to extract the inner struct and pass it to the dispatch function. For example, `Expr::Block(stmts)` becomes `Expr::Block(block)` and passes `block` to the dispatcher. Similarly for Tuple, Loop, Par, Propagate, Break.

In `walk_binding`, the `BindTarget::Pattern(pid)` arm dereferences `*pid` — this should not need changes.

Update the dispatch function references as needed based on signature changes in walk_expr2.rs (next task).

### Task 5: Update walk_expr.rs

In `crates/lx/src/visitor/walk/walk_expr.rs`:

Replace `walk_block_dispatch` and `walk_tuple_dispatch` macro invocations. Change from `walk_dispatch_id_slice!` to `walk_dispatch_id!` since they now take a struct reference instead of a slice. Update the type parameters accordingly: `walk_block_dispatch` dispatches `ExprBlock` with `ExprId`, `walk_tuple_dispatch` dispatches `ExprTuple` with `ExprId`.

Update `walk_block`: change signature from `(v, id, stmts: &[StmtId], span, arena)` to `(v, id, block: &ExprBlock, span, arena)`. Replace the manual `for &s in stmts` loop with `block.walk_children(v, arena)?;`. Update the `leave_block` call to pass `block` instead of `stmts`.

Update `walk_tuple`: change signature from `(v, id, elems: &[ExprId], span, arena)` to `(v, id, tuple: &ExprTuple, span, arena)`. Replace the manual `for &e in elems` loop with `tuple.walk_children(v, arena)?;`. Update the `leave_tuple` call to pass `tuple` instead of `elems`.

### Task 6: Update walk_expr2.rs

In `crates/lx/src/visitor/walk/walk_expr2.rs`:

Replace `walk_loop_dispatch`, `walk_par_dispatch`, `walk_sel_dispatch` macro invocations. Change `walk_loop_dispatch` and `walk_par_dispatch` from `walk_dispatch_id_slice!` to `walk_dispatch_id!` since they now take struct references. `walk_sel_dispatch` stays as-is (SelArm was already a struct).

Replace the hand-written `walk_propagate_dispatch` and `walk_break_dispatch` functions with `walk_dispatch_id!` macro invocations: `walk_dispatch_id!(walk_propagate_dispatch, walk_propagate, visit_propagate, leave_propagate, ExprPropagate, ExprId)` and `walk_dispatch_id!(walk_break_dispatch, walk_break, visit_break, leave_break, ExprBreak, ExprId)`.

Update `walk_loop`: change signature to take `&ExprLoop`, replace manual loop with `loop_node.walk_children(v, arena)?;`, update leave call.

Update `walk_par`: change signature to take `&ExprPar`, replace manual loop with `par.walk_children(v, arena)?;`, update leave call.

Update `walk_propagate`: change signature to take `&ExprPropagate`, replace `dispatch_expr(v, inner, arena)?;` with `propagate.walk_children(v, arena)?;`, update leave call.

Update `walk_break`: change signature to take `&ExprBreak`, replace `if let Some(val)` with `brk.walk_children(v, arena)?;`, update leave call.

### Task 7: Update parser construction sites

In `crates/lx/src/parser/expr.rs`, update:

- `Expr::Loop(stmts)` → `Expr::Loop(ExprLoop { stmts })`
- `Expr::Par(stmts)` → `Expr::Par(ExprPar { stmts })`
- `Expr::Break(val)` → `Expr::Break(ExprBreak { value: val })`

In `crates/lx/src/parser/expr_pratt.rs`, update:

- `Expr::Propagate(o)` → `Expr::Propagate(ExprPropagate { inner: o })`

In `crates/lx/src/parser/expr_compound.rs`, update:

- `Expr::Tuple(elems)` → `Expr::Tuple(ExprTuple { elems })`

In `crates/lx/src/parser/expr_helpers.rs`, update:

- `Expr::Block(stmts)` → `Expr::Block(ExprBlock { stmts })`

Add the necessary imports for the new struct types in each file.

### Task 8: Update formatter match arms

In `crates/lx/src/formatter/emit_expr.rs`, update these match arms:

- `Expr::Block(stmts)` → `Expr::Block(ExprBlock { stmts })`
- `Expr::Tuple(elems)` → `Expr::Tuple(ExprTuple { elems })`
- `Expr::Propagate(inner)` → `Expr::Propagate(ExprPropagate { inner })`
- `Expr::Loop(stmts)` → `Expr::Loop(ExprLoop { stmts })`
- `Expr::Break(val)` → `Expr::Break(ExprBreak { value: val })` (or destructure as `value` and update the body)
- `Expr::Par(stmts)` → `Expr::Par(ExprPar { stmts })`

Add the necessary imports for the new struct types.

### Task 9: Update interpreter match arms

In `crates/lx/src/interpreter/mod.rs`, update these match arms in the `eval` method:

- `Expr::Block(ref stmts)` → `Expr::Block(ref b)` with `b.stmts` access (or `Expr::Block(ExprBlock { ref stmts })`)
- `Expr::Tuple(ref elems)` → `Expr::Tuple(ref t)` with `t.elems` access (or `Expr::Tuple(ExprTuple { ref elems })`)
- `Expr::Propagate(inner)` → `Expr::Propagate(ExprPropagate { inner })`
- `Expr::Loop(ref stmts)` → `Expr::Loop(ref l)` with `l.stmts` access (or `Expr::Loop(ExprLoop { ref stmts })`)
- `Expr::Break(val)` → `Expr::Break(ExprBreak { value: val })`
- `Expr::Par(ref stmts)` → `Expr::Par(ref p)` with `p.stmts` access (or `Expr::Par(ExprPar { ref stmts })`)

Add the necessary imports for the new struct types.

### Task 10: Update checker match arms

In `crates/lx/src/checker/check_expr.rs`, update all match arms for the six variants to destructure through the new structs. The `Expr::Block(stmts)` arm destructures as `Expr::Block(ExprBlock { stmts })`. The `Expr::Tuple(_)`, `Expr::Propagate(_)`, `Expr::Loop(_)`, `Expr::Break(_)`, `Expr::Par(_)` wildcard arms can stay as wildcards since they only forward to synth.

In `crates/lx/src/checker/type_ops.rs`, update all six match arms: `Expr::Block(stmts)` → `Expr::Block(ExprBlock { stmts })`, `Expr::Tuple(elems)` → `Expr::Tuple(ExprTuple { elems })`, `Expr::Propagate(inner)` → `Expr::Propagate(ExprPropagate { inner })`, `Expr::Loop(stmts)` → `Expr::Loop(ExprLoop { stmts })`, `Expr::Break(value)` → `Expr::Break(ExprBreak { value })`, `Expr::Par(stmts)` → `Expr::Par(ExprPar { stmts })`.

Add the necessary imports for the new struct types in each file.

### Task 11: Update checker/capture.rs visitor hook signatures

In `crates/lx/src/checker/capture.rs`, update three visitor hook implementations on `FreeVarCollector`:

- `visit_block(&mut self, _id: ExprId, stmts: &[StmtId], _span: SourceSpan)` → `visit_block(&mut self, _id: ExprId, block: &ExprBlock, _span: SourceSpan)` — change `for &s in stmts` to `for &s in &block.stmts`
- `visit_loop(&mut self, _id: ExprId, stmts: &[StmtId], _span: SourceSpan)` → `visit_loop(&mut self, _id: ExprId, loop_node: &ExprLoop, _span: SourceSpan)` — change `for &s in stmts` to `for &s in &loop_node.stmts`
- `visit_par(&mut self, _id: ExprId, stmts: &[StmtId], _span: SourceSpan)` → `visit_par(&mut self, _id: ExprId, par: &ExprPar, _span: SourceSpan)` — change `for &s in stmts` to `for &s in &par.stmts`

Add imports for `ExprBlock`, `ExprLoop`, `ExprPar`.

### Task 12: Update stdlib/diag/diag_walk.rs visitor hooks and walk calls

In `crates/lx/src/stdlib/diag/diag_walk.rs`, update visitor hook implementations on `Walker`:

- `leave_par(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan)` → `leave_par(&mut self, _id: ExprId, _par: &ExprPar, _span: SourceSpan)`
- `leave_loop(&mut self, _id: ExprId, _stmts: &[StmtId], _span: SourceSpan)` → `leave_loop(&mut self, _id: ExprId, _loop: &ExprLoop, _span: SourceSpan)`
- `visit_par(&mut self, _id: ExprId, stmts: &[StmtId], span: SourceSpan)` → `visit_par(&mut self, _id: ExprId, par: &ExprPar, span: SourceSpan)` — change `walk_par(self, _id, stmts, span, arena)` to `walk_par(self, _id, par, span, arena)`
- `visit_loop(&mut self, _id: ExprId, stmts: &[StmtId], span: SourceSpan)` → `visit_loop(&mut self, _id: ExprId, loop_node: &ExprLoop, span: SourceSpan)` — change `walk_loop(self, _id, stmts, span, arena)` to `walk_loop(self, _id, loop_node, span, arena)`

Add imports for `ExprPar`, `ExprLoop` to the use statement.

### Task 13: Update stdlib/diag/diag_helpers.rs

In `crates/lx/src/stdlib/diag/diag_helpers.rs`, update the `unwrap_propagate` function:

- `Expr::Propagate(inner)` → `Expr::Propagate(ExprPropagate { inner })`

Add import for `ExprPropagate`.

### Task 14: Update folder/desugar.rs

In `crates/lx/src/folder/desugar.rs`, update the `Expr::Block` construction site:

- `Expr::Block(block_stmts)` → `Expr::Block(ExprBlock { stmts: block_stmts })`

Add import for `ExprBlock`.

### Task 15: Update linter rules

In `crates/lx/src/linter/rules/unreachable_code.rs`:

- `Expr::Block(stmts) | Expr::Loop(stmts)` → `Expr::Block(ExprBlock { stmts }) | Expr::Loop(ExprLoop { stmts })`
- `Expr::Par(stmts)` → `Expr::Par(ExprPar { stmts })`
- `Expr::Propagate(inner) | Expr::Break(Some(inner))` — update to destructure through structs: `Expr::Propagate(ExprPropagate { inner }) | Expr::Break(ExprBreak { value: Some(inner) })`
- `Expr::Tuple(elems)` → `Expr::Tuple(ExprTuple { elems })`
- `matches!(arena.expr(*eid), Expr::Break(_))` — this wildcard match does not need changes since it ignores the inner value

In `crates/lx/src/linter/rules/mut_never_mutated.rs`:

- `Expr::Block(stmts) | Expr::Loop(stmts) | Expr::Par(stmts)` → `Expr::Block(ExprBlock { stmts }) | Expr::Loop(ExprLoop { stmts }) | Expr::Par(ExprPar { stmts })`

In `crates/lx/src/linter/rules/single_branch_par.rs`:

- `Expr::Par(stmts)` → `Expr::Par(ExprPar { stmts })`

In `crates/lx/src/linter/rules/redundant_propagate.rs`:

- `Expr::Propagate(inner_id)` → `Expr::Propagate(ExprPropagate { inner: inner_id })`

In `crates/lx/src/linter/rules/break_outside_loop.rs`:

- `matches!(expr, Expr::Loop(_))` and `matches!(expr, Expr::Break(_))` — these wildcard matches do not need changes since they ignore the inner value

Add the necessary imports for the new struct types in each file that destructures.

### Task 16: Update linter/matcher.rs

In `crates/lx/src/linter/matcher.rs`:

- `Expr::Propagate(id)` → `Expr::Propagate(ExprPropagate { inner: id })` (or rename binding accordingly)
- `Expr::Block(_)` — this wildcard match does not need changes

Add import for `ExprPropagate`.

### Task 17: Format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 18: Commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: wrap inline Expr variants in named structs for uniform AST"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 19: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 20: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 21: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 22: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 23: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/VISITOR_PHASE1_UNIFORM_VARIANTS.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

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
