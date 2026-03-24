# Goal

Make leave hooks return `()` instead of `ControlFlow<()>`, convert `visit_program` from a traversal driver to a pure hook, and remove the redundant `arena: &AstArena` parameter from all visitor hook signatures.

# Prerequisites

None.

# Why

- `leave_*` methods return `ControlFlow<()>`, allowing them to abort the walk after children have already been visited. This makes push/pop patterns unsafe — if a leave hook aborts, the matching pop never runs. `diag_walk.rs` and `break_outside_loop.rs` both rely on leave hooks for cleanup
- `visit_program` has a default that calls `walk_program` internally, making it both entry point and hook. Overriding it without calling `walk_program` silently skips the entire tree
- Every `visit_*`/`leave_*` receives `arena: &AstArena` redundantly — implementors that need arena already have it on `self` or from `LintRule::run`

# All 10 AstVisitor implementors (exhaustive)

| # | Struct | File | Hooks that ACTIVELY use arena | Behind Box\<dyn\>? |
|---|--------|------|-------------------------------|---------------------|
| 1 | `CoreValidator` | `folder/validate_core.rs` | none — `_arena` unused | No |
| 2 | `BreakOutsideLoop` | `linter/rules/break_outside_loop.rs` | none — `_arena` unused | Yes (LintRule) |
| 3 | `DuplicateRecordField` | `linter/rules/duplicate_record_field.rs` | none — `_arena` unused | Yes (LintRule) |
| 4 | `SingleBranchPar` | `linter/rules/single_branch_par.rs` | none — `_arena` unused | Yes (LintRule) |
| 5 | `RedundantPropagate` | `linter/rules/redundant_propagate.rs` | none — `_arena` unused | Yes (LintRule) |
| 6 | `UnusedImport` | `linter/rules/unused_import.rs` | none — overrides zero hooks | Yes (LintRule) |
| 7 | `EmptyMatch` | `linter/rules/empty_match.rs` | `visit_expr` — calls `ExprMatcher::empty_match().matches(expr, arena)` | Yes (LintRule) |
| 8 | `UnreachableCode` | `linter/rules/unreachable_code.rs` | `visit_expr` — calls `arena.stmt_span()`, `arena.stmt()`, `arena.expr()` | Yes (LintRule) |
| 9 | `Walker` | `stdlib/diag/diag_walk.rs` | `visit_program`, `visit_binding`, `visit_par`, `visit_sel`, `visit_match`, `visit_ternary`, `visit_loop`, `visit_apply` — all call arena methods or dispatch functions | No |
| 10 | `FreeVarCollector` | `checker/capture.rs` | `visit_binding`, `visit_func`, `visit_block`, `visit_with`, `visit_loop`, `visit_par`, `visit_match` — all call dispatch/walk functions with arena. Also `visit_pattern_list`, `visit_pattern_record` on PatternVisitor | No |

Arena-access strategy per struct:
- **#1-6**: Don't use arena in hooks. Just remove the parameter.
- **#7-8** (lint rules behind `Box<dyn LintRule>`): Cannot add lifetime parameter. Add `arena: *const AstArena` field. Set from `run()` method. Access via `unsafe { &*self.arena }` in hooks. Safe because arena outlives the `run` call.
- **#9** (Walker): Not behind dyn. Add `arena: &'a AstArena` field. Set from `visit_program` (which receives `&program.arena`). Make struct `Walker<'a>`.
- **#10** (FreeVarCollector): Not behind dyn. Add `arena: &'a AstArena` field. Make struct `FreeVarCollector<'a>`. Update `free_vars(eid, arena)` to pass arena to constructor.

# All visit_program call sites (5 total)

1. `folder/validate_core.rs` line 26: `validator.visit_program(program);` — return value ignored
2. `stdlib/diag/mod.rs` line 60: `let _ = walker.visit_program(program);` — return value discarded
3. `stdlib/diag/mod.rs` line 66: `let _ = walker.visit_program(program);` — return value discarded
4. `stdlib/diag/mod.rs` line 83: `let _ = walker.visit_program(&program);` — return value discarded
5. `stdlib/diag/diag_walk.rs` line 113: `fn visit_program<P>(&mut self, program: &Program<P>) -> VisitAction` — override that does a custom pre-pass (lines 114-127) then calls `walk_program(self, program)` (line 129)

All 5 must change from `visitor.visit_program(program)` to `walk_program(&mut visitor, program)`.

# Walker.visit_program restructuring

Walker's current `visit_program` (diag_walk.rs lines 113-133) does two things in sequence:
1. A pre-pass over `program.stmts` to build `fn_nodes` (lines 114-127)
2. Delegates to `walk_program(self, program)` (line 129)

After the change, `visit_program` becomes a pure hook returning `VisitAction`. The pre-pass must move into `visit_program` (it runs before children are walked) and return `Descend` so `walk_program` handles the child traversal. The `walk_program` call on line 129 is removed — `walk_program` drives the traversal externally.

New Walker.visit_program:
```rust
fn visit_program<P>(&mut self, program: &Program<P>) -> VisitAction {
    self.arena = &program.arena as *const AstArena;
    let arena = unsafe { &*self.arena };
    let saved_fn = self.current_fn.take();
    for &sid in &program.stmts {
        let stmt = arena.stmt(sid);
        if let Stmt::Binding(b) = stmt
            && let BindTarget::Name(name) | BindTarget::Reassign(name) = &b.target
            && matches!(arena.expr(b.value), Expr::Func(_))
            && *name != "main"
            && !self.fn_nodes.contains_key(name)
        {
            let id = self.add_node("agent", name.to_string(), NodeKind::Agent);
            self.fn_nodes.insert(*name, id);
        }
    }
    self.current_fn = saved_fn;
    VisitAction::Descend
}
```

The callers in `diag/mod.rs` change from `let _ = walker.visit_program(program)` to `let _ = walk_program(&mut walker, program)`.

# Program construction sites (2 total)

1. `parser/mod.rs` line 84: `Program { stmts, arena, comments, file, _phase: PhantomData }`
2. `folder/desugar.rs` line 162: `Program { stmts: folded.stmts, arena: folded.arena, comments: folded.comments, file: folded.file, _phase: PhantomData }`

`walk_transform_program` in `visitor/walk_transform/mod.rs` lines 35-40 destructures and reconstructs Program — carries fields through.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/visitor/visitor_trait.rs` | All leave return `()`, remove arena from all sigs, visit_program becomes pure hook |
| `crates/lx/src/visitor/visitor_pattern_hooks.rs` | All leave return `()`, remove arena from all sigs |
| `crates/lx/src/visitor/visitor_type_hooks.rs` | All leave return `()`, remove arena from all sigs |
| `crates/lx/src/visitor/action.rs` | Remove `to_control_flow` if unused |
| `crates/lx/src/visitor/walk/mod.rs` | Update macros, stop propagating leave, stop passing arena to hooks, add visit_program dispatch to walk_program |
| `crates/lx/src/visitor/walk/walk_expr.rs` | Stop propagating leave, stop passing arena to hooks |
| `crates/lx/src/visitor/walk/walk_expr2.rs` | Stop propagating leave, stop passing arena to hooks |
| `crates/lx/src/visitor/walk/walk_pattern.rs` | Stop propagating leave, stop passing arena to hooks |
| `crates/lx/src/visitor/walk/walk_type.rs` | Stop propagating leave, stop passing arena to hooks |
| `crates/lx/src/folder/validate_core.rs` | Call walk_program instead of visit_program |
| `crates/lx/src/linter/rules/break_outside_loop.rs` | leave_expr returns `()`, remove arena from sigs |
| `crates/lx/src/linter/rules/empty_match.rs` | Add `arena: *const AstArena`, remove arena from visit_expr sig, access via self |
| `crates/lx/src/linter/rules/unreachable_code.rs` | Add `arena: *const AstArena`, remove arena from visit_expr sig, access via self |
| `crates/lx/src/linter/rules/duplicate_record_field.rs` | Remove arena from visit_expr sig |
| `crates/lx/src/linter/rules/single_branch_par.rs` | Remove arena from visit_expr sig |
| `crates/lx/src/linter/rules/redundant_propagate.rs` | Remove arena from visit_expr sig |
| `crates/lx/src/stdlib/diag/diag_walk.rs` | All leave return `()`, add arena raw pointer field, remove arena from all hook sigs, restructure visit_program |
| `crates/lx/src/stdlib/diag/mod.rs` | Lines 60, 66, 83: change visit_program → walk_program |
| `crates/lx/src/checker/capture.rs` | Add `arena: &'a AstArena` field to `FreeVarCollector`, make generic `<'a>`, remove arena from all hook sigs, update `free_vars` |

# Task List

### Task 1: Change all leave return types to () in the three visitor traits

In `crates/lx/src/visitor/visitor_trait.rs`, change every `leave_*` method (35 total) from `-> ControlFlow<()> { ControlFlow::Continue(()) }` to just `{ }` (no return type annotation — defaults to `()`).

In `crates/lx/src/visitor/visitor_pattern_hooks.rs`, change all 5 leave methods the same way.

In `crates/lx/src/visitor/visitor_type_hooks.rs`, change all 8 leave methods the same way.

Remove `use std::ops::ControlFlow;` from `visitor_trait.rs`, `visitor_pattern_hooks.rs`, `visitor_type_hooks.rs` if it becomes unused in each file.

### Task 2: Update walk_dispatch macros in walk/mod.rs

In `crates/lx/src/visitor/walk/mod.rs`, update `walk_dispatch_id!`: the Skip branch currently returns `v.$leave(...)`. Change to:

```rust
VisitAction::Skip => {
    v.$leave(id, node, span);
    ControlFlow::Continue(())
},
```

Note: arena is still passed to the walk function `$walk_name` on the Descend branch — only the `v.$visit` and `v.$leave` calls lose the arena argument.

Apply the same change to `walk_dispatch_id_slice!`.

### Task 3: Update dispatch_stmt, walk_program, dispatch_expr, and all walk functions in walk/mod.rs

**dispatch_stmt** (line 72): Skip branch calls `v.leave_stmt(...)` — change to statement + `ControlFlow::Continue(())`. After `walk_stmt(v, id, arena)?`, change `v.leave_stmt(id, stmt, span, arena)` to `v.leave_stmt(id, stmt, span); ControlFlow::Continue(())`.

**walk_program** (line 64): Change to include visit_program dispatch:

```rust
pub fn walk_program<V: AstVisitor + ?Sized, P>(v: &mut V, program: &Program<P>) -> ControlFlow<()> {
    let action = v.visit_program(program);
    match action {
        VisitAction::Stop => return ControlFlow::Break(()),
        VisitAction::Skip => {
            v.leave_program(program);
            return ControlFlow::Continue(());
        },
        VisitAction::Descend => {},
    }
    let arena = &program.arena;
    for &sid in &program.stmts {
        dispatch_stmt(v, sid, arena)?;
    }
    v.leave_program(program);
    ControlFlow::Continue(())
}
```

**dispatch_expr** (line 138): Same pattern — leave call becomes statement, return `ControlFlow::Continue(())`.

**walk_binding** (line 127): `v.leave_binding(...)` becomes statement, return `ControlFlow::Continue(())`.

**walk_trait_decl** (line 197): Same.
**walk_class_decl** (line 202): Same.
**walk_field_update** (line 207): Same.

For all `v.visit_*` and `v.leave_*` calls in this file, remove the `arena` argument. The walk functions still receive and use `arena` themselves — they just stop forwarding it to hooks.

### Task 4: Update walk/walk_expr.rs

Every concrete walk function (walk_literal through walk_match, 14 total) currently returns `v.leave_X(...)`. Change each to call leave as a statement, then return `ControlFlow::Continue(())`.

Remove `arena` from all `v.leave_X(...)` calls. Also remove `arena` from all `v.visit_X(...)` calls inside the `walk_dispatch_id!` and `walk_dispatch_id_slice!` macro invocations — these are generated by the macros updated in Task 2.

### Task 5: Update walk/walk_expr2.rs

Same pattern as Task 4 for all 14 concrete walk functions (walk_ternary through walk_with).

Also update `walk_propagate_dispatch` (line 24) and `walk_break_dispatch` (line 33) — hand-written dispatch functions. Their Skip branches call leave and return the result. Change to call leave as statement + `ControlFlow::Continue(())`.

Remove `arena` from all `v.visit_*` and `v.leave_*` calls.

### Task 6: Update walk/walk_pattern.rs

`walk_pattern_dispatch` (line 9): Skip branch — call leave as statement. Remove arena from `v.visit_pattern` and `v.leave_pattern` calls.

`walk_pattern` (line 20): leave_pattern becomes statement. Remove arena from all leaf pattern visit calls (`v.visit_pattern_literal`, `v.visit_pattern_bind`, `v.visit_pattern_wildcard`).

All 4 dispatch functions and 4 concrete walk functions — same pattern.

### Task 7: Update walk/walk_type.rs

`walk_type_expr_dispatch` (line 9): Skip branch. Remove arena from `v.visit_type_expr`, `v.leave_type_expr`.

`walk_type_expr` (line 20): leave_type_expr becomes statement. Remove arena from leaf type visits (`v.visit_type_named`, `v.visit_type_var`).

All 7 dispatch functions and 7 concrete walk functions — same pattern.

### Task 8: Convert visit_program to a pure hook

In `crates/lx/src/visitor/visitor_trait.rs`, replace visit_program:

```rust
fn visit_program<P>(&mut self, _program: &Program<P>) -> VisitAction {
    VisitAction::Descend
}
```

Remove the `walk_program` import from this file.

Change leave_program to:

```rust
fn leave_program<P>(&mut self, _program: &Program<P>) {}
```

### Task 9: Update CoreValidator

In `crates/lx/src/folder/validate_core.rs`:

Remove `_arena: &AstArena` from `visit_expr` signature. The hook only pattern-matches on `expr` which is still passed as a parameter.

Change `validate_core`:

```rust
pub(super) fn validate_core(program: &Program<Core>) {
    let mut validator = CoreValidator;
    let _ = walk_program(&mut validator, program);
}
```

Add `use crate::visitor::walk_program;` import.

### Task 10: Update all lint rules

For each lint rule, remove `arena`/`_arena` from all overridden hook signatures. Additionally:

**BreakOutsideLoop** (`break_outside_loop.rs`): Remove `_arena: &AstArena` from `visit_expr` and `leave_expr`. Change `leave_expr` return from `ControlFlow::Continue(())` to nothing (just the body).

**DuplicateRecordField** (`duplicate_record_field.rs`): Remove `_arena: &AstArena` from `visit_expr`.

**SingleBranchPar** (`single_branch_par.rs`): Remove `_arena: &AstArena` from `visit_expr`.

**RedundantPropagate** (`redundant_propagate.rs`): Remove `_arena: &AstArena` from `visit_expr`.

**EmptyMatch** (`empty_match.rs`): This rule actively uses arena. Add field `arena: *const AstArena` to the struct. Initialize to `std::ptr::null()` in `new()`. In `run()`, set before dispatch: `self.arena = arena as *const AstArena;`. In `visit_expr`, access via `let arena = unsafe { &*self.arena };`. Remove `arena` from `visit_expr` signature.

**UnreachableCode** (`unreachable_code.rs`): Same pattern as EmptyMatch — add `arena: *const AstArena` field, set in `run()`, access via unsafe in `visit_expr`.

### Task 11: Update Walker in diag_walk.rs

Add `arena: *const AstArena` field to Walker struct (Walker is constructed in multiple places in `diag/mod.rs` — update all constructors to set `arena: std::ptr::null()`).

Restructure `visit_program` as specified in the "Walker.visit_program restructuring" section above. Key change: the pre-pass over stmts stays in `visit_program`, but `walk_program(self, program)` is removed — return `VisitAction::Descend` instead.

For all 5 leave hooks (`leave_par`, `leave_sel`, `leave_match`, `leave_ternary`, `leave_loop`), change return type from `ControlFlow<()>` to `()` and remove the `ControlFlow::Continue(())` return.

For all visit hooks that use arena (`visit_binding`, `visit_par`, `visit_sel`, `visit_match`, `visit_ternary`, `visit_loop`, `visit_apply`), remove `arena: &AstArena` from the signature and instead access via `let arena = unsafe { &*self.arena };` at the top of each hook body.

In `crates/lx/src/stdlib/diag/mod.rs`, change all 3 call sites (lines 60, 66, 83) from `let _ = walker.visit_program(...)` to `let _ = walk_program(&mut walker, ...)`.

### Task 12: Update FreeVarCollector in capture.rs

Make `FreeVarCollector` generic: `struct FreeVarCollector<'a>` with field `arena: &'a AstArena`.

Update `new()` to `fn new(arena: &'a AstArena) -> Self` and store arena.

Update `free_vars`:

```rust
pub fn free_vars(eid: ExprId, arena: &AstArena) -> HashSet<Sym> {
    let mut collector = FreeVarCollector::new(arena);
    match dispatch_expr(&mut collector, eid, arena) {
        ControlFlow::Continue(()) | ControlFlow::Break(()) => {},
    }
    collector.free
}
```

Remove `arena: &AstArena` from all 8 AstVisitor hook signatures and 3 PatternVisitor hook signatures. Replace `arena` usage in each hook body with `self.arena`. Every call to `dispatch_stmt(self, s, arena)`, `dispatch_expr(self, eid, arena)`, `walk_binding(self, id, binding, span, arena)`, `walk_func(self, _id, func, span, arena)`, `walk_pattern(self, pid, pattern, pspan, arena)` changes `arena` to `self.arena`.

### Task 13: Verify

Run `just fmt` then `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

### Task 14: Commit

Run `just fmt` then `git add -A && git commit -m "refactor: visitor protocol overhaul — leave returns (), visit_program as hook, remove arena from hook signatures"`.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **`walk_children` is NOT a visitor hook** — it is infrastructure. Do NOT remove arena from `walk_children` signatures, walk function parameters, or `dispatch_*` function parameters. Only remove arena from `v.visit_*()` and `v.leave_*()` calls.
5. **The `AstTransformer` trait is separate** — do not modify transformer leave hooks. They return `(Node, SourceSpan)` and take `&mut AstArena`.
6. **For lint rules behind `Box<dyn LintRule>`** (EmptyMatch, UnreachableCode), use `arena: *const AstArena` raw pointer. Set in `run()`, access via `unsafe { &*self.arena }`. This is safe: arena outlives `run`.
7. **For internal types** (FreeVarCollector, Walker), prefer lifetime references when possible. Walker uses raw pointer because `visit_program` receives `&Program<P>` with a generic — extracting `&program.arena` and storing it with a lifetime tied to `P` is unwieldy. FreeVarCollector can use `&'a AstArena` because it receives arena via `free_vars(eid, arena)`.
8. **Walker's `visit_program` override** currently calls `walk_program(self, program)` on line 129. After this refactor, `walk_program` is the external entry point that calls `visit_program` internally. If Walker's `visit_program` still called `walk_program`, it would create infinite recursion. The `walk_program` call MUST be removed from Walker's `visit_program`.
9. **Search comprehensively**: `rg --type rust 'fn leave_' crates/lx/src/` and `rg --type rust '_arena: &AstArena' crates/lx/src/` to catch every remaining occurrence after changes.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/VISITOR_PROTOCOL_OVERHAUL.md" })
```

Then call `next_task` to begin.
