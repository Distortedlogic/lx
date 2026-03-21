# Goal

Formalize the AST visitor pattern infrastructure to eliminate spaghetti: extract named structs from complex Stmt/Expr enum variants so context structs disappear, add missing visitor methods for every AST node, fix incomplete walk coverage, reorganize misnamed walk files, make capture.rs exhaustive, and add post-visit hooks to eliminate the save/restore anti-pattern in consumer code. Align with the rustc/syn convention: one visit method per AST node, one walk function per visit method, no parallel type hierarchies.

# Why

- Four ad-hoc context structs (`TraitDeclCtx`, `AgentDeclCtx`, `RefineCtx`, `MetaCtx`) in the visitor module duplicate fields from the AST. When fields change, both the AST variant and the context struct must be updated — a maintenance trap that will bite as the language evolves
- `Expr::Receive` is handled inline in `walk_expr` with a bare loop instead of delegating to a `visit_receive` method — consumers cannot override Receive traversal behavior
- `Stmt::TraitUnion` is silently dropped in `walk_stmt` — no visitor callback fires, any analysis pass that needs trait union information gets nothing
- `walk_binding` never visits `BindTarget::Pattern` — destructuring bindings like `(a, b) = expr` have their patterns silently skipped by every visitor consumer
- `visit_stream_ask` defaults to calling `walk_agent_ask` instead of a dedicated `walk_stream_ask` — overriding ask behavior unexpectedly affects stream behavior
- `walk_pattern.rs` (266 lines) is 80% expression walk functions with only 4 pattern walks at the bottom — anyone looking for expression walk logic will never find it in a file called "walk_pattern"
- `capture.rs:134` uses `_ => {}` silently ignoring `WithResource`, `WithContext`, `Refine`, `Meta`, `Shell`, `Yield`, `Emit`, `StreamAsk`, `Loop`, `Break`, `Assert`, `NamedArg`, `Slice`, `Receive` — free variable analysis is incomplete for any program using these constructs
- The diagnostic walker (`diag_walk.rs`) has 8+ instances of a save-context/visit-children/restore-context anti-pattern because the visitor has no post-visit hook — this is exactly the problem enter/exit hooks solve
- No existing Rust crate covers these needs. `derive_visitor` and similar crates are too simplistic for a custom language AST with 40+ expression variants and domain-specific nodes. The rustc hand-written trait + walk function pattern IS the established convention for production language implementations — the code just needs to follow it properly

# What changes

**AST struct extraction (Tasks 1-2):** Extract the fields of `Stmt::TraitDecl` (8 fields), `Stmt::AgentDecl` (7 fields), `Stmt::ClassDecl` (5 fields), `Expr::Refine` (6 fields), and `Expr::Meta` (6 fields) into named structs: `TraitDeclData`, `AgentDeclData`, `ClassDeclData`, `RefineDef`, `MetaDef`. Update the enum variants to hold these structs. Propagate the change to every match/construction site: parser, interpreter, checker, visitor walk, diag_walk.

**Visitor trait rewrite (Tasks 3-4):** Remove all four context structs. Change visitor methods to accept references to the new AST structs directly. Add `visit_receive` and `visit_trait_union`. Fix `walk_binding` to visit `BindTarget::Pattern`. Add `walk_stream_ask` so `visit_stream_ask` has its own walk function. Rearrange method signatures so every compound AST node has a consistent visit+walk pair.

**Consumer update (Task 5):** Update `diag_walk.rs` and `diag_walk_expr.rs` to match the new visitor method signatures.

**File reorganization (Task 6):** Move the ~20 expression walk functions currently misplaced in `walk_pattern.rs` into the expression walk file. Keep only the 5 actual pattern walk functions in `walk_pattern.rs`.

**Exhaustive variant handling (Task 7):** Remove the wildcard catch-all in `capture.rs`. Add explicit handling for every `Expr` variant. For variants containing sub-expressions (`WithResource`, `WithContext`, `Refine`, `Meta`, `Shell`, `Yield`, `Emit`, `StreamAsk`, `Loop`, `Break`, `Assert`, `NamedArg`, `Slice`, `Receive`), recurse into their sub-expressions. For true leaf variants, handle as explicit no-ops.

**Post-visit hooks (Tasks 8-9):** Add `visit_*_post` methods to `AstVisitor` for nodes that contain child scopes: `visit_agent_decl_post`, `visit_block_post`, `visit_func_post`, `visit_par_post`, `visit_sel_post`, `visit_match_post`, `visit_loop_post`, `visit_with_post`, `visit_with_resource_post`, `visit_with_context_post`, `visit_refine_post`. All default to no-ops. Walk functions call the post method after visiting children. Refactor `diag_walk` to use post hooks instead of manual save/restore.

# Files affected

**New files:**
- None (all changes to existing files)

**Modified files — AST:**
- `crates/lx/src/ast/mod.rs` — Stmt enum uses new struct variants
- `crates/lx/src/ast/expr_types.rs` — Expr::Refine and Expr::Meta use RefineDef/MetaDef
- `crates/lx/src/ast/types.rs` — Define TraitDeclData, AgentDeclData, ClassDeclData, RefineDef, MetaDef

**Modified files — Visitor:**
- `crates/lx/src/visitor/mod.rs` — Remove context structs, rewrite trait methods, add post-visit hooks
- `crates/lx/src/visitor/walk/mod.rs` — Update walk_stmt, walk_expr for new enum shapes; add walk_receive, walk_trait_union, walk_stream_ask; fix walk_binding; add post-visit calls
- `crates/lx/src/visitor/walk/walk_helpers.rs` — Absorb expression walks from walk_pattern.rs, renamed or expanded
- `crates/lx/src/visitor/walk/walk_pattern.rs` — Shrink to pattern-only walks
- `crates/lx/src/visitor/walk/walk_type.rs` — No structural changes (type walks are clean)

**Modified files — Consumers:**
- `crates/lx/src/stdlib/diag/diag_walk.rs` — New visitor signatures, post-visit hooks
- `crates/lx/src/stdlib/diag/diag_walk_expr.rs` — New Refine/Meta shapes
- `crates/lx/src/checker/stmts.rs` — Match on new Stmt struct variants
- `crates/lx/src/checker/synth.rs` — Match on new Expr struct variants (Refine, Meta)
- `crates/lx/src/checker/capture.rs` — Exhaustive Expr handling, no wildcard
- `crates/lx/src/interpreter/exec_stmt.rs` — Match on new Stmt struct variants
- `crates/lx/src/interpreter/refine.rs` — Match on RefineDef
- `crates/lx/src/interpreter/meta.rs` — Match on MetaDef

**Modified files — Parser (construction sites):**
- Parser files that construct Stmt::TraitDecl, Stmt::AgentDecl, Stmt::ClassDecl, Expr::Refine, Expr::Meta — update to construct the new structs

# Task List

## Task 1: Define named structs for complex AST variants

**Subject:** Extract TraitDeclData, AgentDeclData, ClassDeclData, RefineDef, MetaDef from inline enum fields
**ActiveForm:** Defining named AST structs

In `crates/lx/src/ast/types.rs`, define five new structs holding the fields currently inline in their respective enum variants:

`TraitDeclData` — name: String, entries: Vec\<TraitEntry\>, methods: Vec\<TraitMethodDecl\>, defaults: Vec\<AgentMethod\>, requires: Vec\<String\>, description: Option\<String\>, tags: Vec\<String\>, exported: bool.

`AgentDeclData` — name: String, traits: Vec\<String\>, uses: Vec\<(String, String)\>, init: Option\<SExpr\>, on: Option\<SExpr\>, methods: Vec\<AgentMethod\>, exported: bool.

`ClassDeclData` — name: String, traits: Vec\<String\>, fields: Vec\<ClassField\>, methods: Vec\<AgentMethod\>, exported: bool.

`RefineDef` — initial: Box\<SExpr\>, grade: Box\<SExpr\>, revise: Box\<SExpr\>, threshold: Box\<SExpr\>, max_rounds: Box\<SExpr\>, on_round: Option\<Box\<SExpr\>\>.

`MetaDef` — task: Box\<SExpr\>, strategies: Box\<SExpr\>, attempt: Box\<SExpr\>, evaluate: Box\<SExpr\>, select: Option\<Box\<SExpr\>\>, on_switch: Option\<Box\<SExpr\>\>.

All structs derive Debug, Clone. Do NOT yet change the enum variants — that happens in the next task.

Verify: `just diagnose` passes.

## Task 2: Refactor Stmt and Expr enums to use named structs

**Subject:** Replace inline fields in Stmt and Expr variants with the new named structs
**ActiveForm:** Refactoring enum variants to use named structs

In `crates/lx/src/ast/mod.rs`, change:
- `Stmt::TraitDecl { name, entries, ... }` → `Stmt::TraitDecl(TraitDeclData)`
- `Stmt::AgentDecl { name, traits, ... }` → `Stmt::AgentDecl(AgentDeclData)`
- `Stmt::ClassDecl { name, traits, ... }` → `Stmt::ClassDecl(ClassDeclData)`

In `crates/lx/src/ast/mod.rs` (the Expr enum):
- `Expr::Refine { initial, grade, ... }` → `Expr::Refine(Box<RefineDef>)`
- `Expr::Meta { task, strategies, ... }` → `Expr::Meta(Box<MetaDef>)`

Then fix every compilation error. The compiler will identify every match arm and construction site that needs updating. At each match site, change destructuring from `Stmt::TraitDecl { name, entries, ... }` to `Stmt::TraitDecl(data)` and access fields as `data.name`, `data.entries`, etc. At each construction site, wrap fields in the struct constructor. Search the parser, interpreter (`exec_stmt.rs`, `refine.rs`, `meta.rs`), checker (`stmts.rs`, `synth.rs`, `synth_helpers.rs`), visitor walks (`walk/mod.rs`), and diag walker (`diag_walk.rs`, `diag_walk_expr.rs`).

Verify: `just diagnose` passes. `just test` passes.

## Task 3: Rewrite AstVisitor trait — remove context structs, add missing methods

**Subject:** Eliminate ad-hoc context structs, accept AST struct refs, add visit_receive and visit_trait_union
**ActiveForm:** Rewriting AstVisitor trait

In `crates/lx/src/visitor/mod.rs`:

Remove `TraitDeclCtx`, `AgentDeclCtx`, `RefineCtx`, `MetaCtx` entirely.

Change visitor method signatures:
- `visit_trait_decl(&mut self, ctx: &TraitDeclCtx, span)` → `visit_trait_decl(&mut self, data: &TraitDeclData, span: Span)`
- `visit_agent_decl(&mut self, ctx: &AgentDeclCtx, span)` → `visit_agent_decl(&mut self, data: &AgentDeclData, span: Span)`
- `visit_class_decl(&mut self, name, traits, fields, methods, exported, span)` → `visit_class_decl(&mut self, data: &ClassDeclData, span: Span)`
- `visit_refine(&mut self, ctx: &RefineCtx, span)` → `visit_refine(&mut self, def: &RefineDef, span: Span)`
- `visit_meta(&mut self, ctx: &MetaCtx, span)` → `visit_meta(&mut self, def: &MetaDef, span: Span)`

Add two new methods with empty defaults:
- `fn visit_receive(&mut self, arms: &[ReceiveArm], span: Span)` — default calls `walk_receive(self, arms, span)`
- `fn visit_trait_union(&mut self, _def: &TraitUnionDef, _span: Span)` — empty default

Change `visit_stream_ask` default from `walk_agent_ask(self, target, msg, span)` to `walk_stream_ask(self, target, msg, span)`.

Verify: `just diagnose` passes.

## Task 4: Update walk functions for new enum shapes and missing walks

**Subject:** Fix walk_stmt, walk_expr, walk_binding; add walk_receive, walk_trait_union, walk_stream_ask
**ActiveForm:** Updating walk functions

In `crates/lx/src/visitor/walk/mod.rs`:

Update `walk_stmt`:
- `Stmt::TraitDecl(data) => v.visit_trait_decl(data, span)` — no context struct construction
- `Stmt::AgentDecl(data) => v.visit_agent_decl(data, span)` — no context struct construction
- `Stmt::ClassDecl(data) => v.visit_class_decl(data, span)` — no parameter explosion
- `Stmt::TraitUnion(def) => v.visit_trait_union(def, span)` — was silently dropped, now dispatches

Update `walk_expr`:
- `Expr::Refine(def) => v.visit_refine(def, span)` — no context struct construction
- `Expr::Meta(def) => v.visit_meta(def, span)` — no context struct construction
- `Expr::Receive(arms) => v.visit_receive(arms, span)` — was inline loop, now delegates

Update `walk_binding`: Add a branch for `BindTarget::Pattern(pat)` that calls `v.visit_pattern(&pat.node, pat.span)`. Currently the pattern in destructuring bindings is never walked.

Update `walk_agent_decl` to accept `&AgentDeclData` instead of `&AgentDeclCtx`. Access fields via `data.init`, `data.on`, `data.methods`.

Update `walk_class_decl` to accept `&ClassDeclData` instead of separate params. Access fields via `data.fields`, `data.methods`.

Update `walk_refine` to accept `&RefineDef` instead of `&RefineCtx`. Access fields via `def.initial`, `def.grade`, etc.

Update `walk_meta` to accept `&MetaDef` instead of `&MetaCtx`.

Add `walk_receive`: iterate arms, call `v.visit_expr` on each arm's handler.

Add `walk_trait_union`: empty (no children to walk).

Add `walk_stream_ask`: body identical to `walk_agent_ask` — visit target and msg. This exists so `visit_stream_ask` has its own dedicated walk function.

Verify: `just diagnose` passes.

## Task 5: Update diag_walk for new visitor API

**Subject:** Fix diag_walk.rs and diag_walk_expr.rs for changed visitor method signatures
**ActiveForm:** Updating diag_walk for new visitor API

In `crates/lx/src/stdlib/diag/diag_walk.rs`:

Update `visit_trait_decl` — change parameter from `&TraitDeclCtx` to `&TraitDeclData`, access `data.name`.

Update `visit_agent_decl` — change parameter from `&AgentDeclCtx` to `&AgentDeclData`, access `data.name`, `data.init`, `data.on`, `data.methods` directly.

Update `visit_class_decl` — change parameter from the 6-param list to `&ClassDeclData`.

In `crates/lx/src/stdlib/diag/diag_walk_expr.rs`:

Update the `Expr::Refine` arm in `visit_expr_diag` — access fields through `def.initial`, `def.grade`, `def.revise` instead of separate destructured fields.

Verify: `just diagnose` passes. `just test` passes.

## Task 6: Reorganize walk file structure

**Subject:** Move expression walks out of walk_pattern.rs into the expression walk file
**ActiveForm:** Reorganizing walk file structure

`walk_pattern.rs` currently contains ~20 expression walk functions and only 5 pattern walk functions. Move all expression walk functions (`walk_func`, `walk_match`, `walk_ternary`, `walk_propagate`, `walk_coalesce`, `walk_slice`, `walk_named_arg`, `walk_loop`, `walk_break`, `walk_assert`, `walk_par`, `walk_sel`, `walk_agent_send`, `walk_agent_ask`, `walk_stream_ask`, `walk_emit`, `walk_yield`, `walk_with`, `walk_with_resource`, `walk_with_context`, `walk_refine`, `walk_meta`, `walk_shell`, `walk_receive`) into `walk_helpers.rs`.

Rename `walk_helpers.rs` to `walk_expr.rs` since it will now contain all expression walk functions (both the simple ones already there and the compound ones moved from walk_pattern.rs).

After the move, `walk_pattern.rs` should contain only: `walk_pattern`, `walk_pattern_tuple`, `walk_pattern_list`, `walk_pattern_record`, `walk_pattern_constructor`.

Update `walk/mod.rs` to reference the renamed module.

Verify: `just diagnose` passes.

## Task 7: Fix capture.rs — exhaustive Expr variant handling

**Subject:** Remove wildcard catch-all, explicitly handle every Expr variant in free variable analysis
**ActiveForm:** Making capture.rs exhaustive over Expr variants

In `crates/lx/src/checker/capture.rs`, replace the `_ => {}` catch-all at the end of `collect_free` with explicit arms for every remaining Expr variant:

Variants that contain sub-expressions — recurse into them:
- `Expr::WithResource { resources, body }` — recurse into resource exprs, recurse into body stmts with new bound scope
- `Expr::WithContext { fields, body }` — recurse into field exprs, recurse into body stmts
- `Expr::Refine(def)` — recurse into initial, grade, revise, threshold, max_rounds, on_round
- `Expr::Meta(def)` — recurse into task, strategies, attempt, evaluate, select, on_switch
- `Expr::Shell { parts, .. }` — recurse into StrPart::Interp expressions
- `Expr::Yield { value }` — recurse into value
- `Expr::Emit { value }` — recurse into value
- `Expr::StreamAsk { target, msg }` — recurse into target and msg
- `Expr::Loop(stmts)` — recurse into stmts via `free_vars_stmts`
- `Expr::Break(val)` — recurse into value if Some
- `Expr::Assert { expr, msg }` — recurse into expr and msg
- `Expr::NamedArg { value, .. }` — recurse into value
- `Expr::Slice { expr, start, end }` — recurse into all present sub-expressions
- `Expr::Receive(arms)` — recurse into each arm's handler

Leaf variants — explicit no-ops:
- `Expr::Literal(_)` — no free variables in literals (string interpolation free vars are already in the Expr tree as Ident nodes within StrPart::Interp)
- `Expr::TypeConstructor(_)` — constructor names are not variable references
- `Expr::Section(section)` — recurse into operand in Right/Left variants; BinOp/Field/Index are no-ops

After this change, the compiler will enforce exhaustiveness when new Expr variants are added.

Verify: `just diagnose` passes. `just test` passes.

## Task 8: Add post-visit hooks to AstVisitor

**Subject:** Add visit_*_post methods for scope-managing nodes so consumers can run logic after children are visited
**ActiveForm:** Adding post-visit hooks to AstVisitor

In `crates/lx/src/visitor/mod.rs`, add these methods to `AstVisitor`, all with empty default implementations:

- `fn visit_agent_decl_post(&mut self, _data: &AgentDeclData, _span: Span) {}`
- `fn visit_class_decl_post(&mut self, _data: &ClassDeclData, _span: Span) {}`
- `fn visit_block_post(&mut self, _stmts: &[SStmt], _span: Span) {}`
- `fn visit_func_post(&mut self, _params: &[Param], _ret_type: Option<&SType>, _body: &SExpr, _span: Span) {}`
- `fn visit_par_post(&mut self, _stmts: &[SStmt], _span: Span) {}`
- `fn visit_sel_post(&mut self, _arms: &[SelArm], _span: Span) {}`
- `fn visit_match_post(&mut self, _scrutinee: &SExpr, _arms: &[MatchArm], _span: Span) {}`
- `fn visit_loop_post(&mut self, _stmts: &[SStmt], _span: Span) {}`
- `fn visit_with_post(&mut self, _name: &str, _value: &SExpr, _body: &[SStmt], _mutable: bool, _span: Span) {}`
- `fn visit_with_resource_post(&mut self, _resources: &[(SExpr, String)], _body: &[SStmt], _span: Span) {}`
- `fn visit_with_context_post(&mut self, _fields: &[(String, SExpr)], _body: &[SStmt], _span: Span) {}`
- `fn visit_refine_post(&mut self, _def: &RefineDef, _span: Span) {}`
- `fn visit_ternary_post(&mut self, _cond: &SExpr, _then_: &SExpr, _else_: Option<&SExpr>, _span: Span) {}`

In the corresponding walk functions (`walk_block`, `walk_func`, `walk_par`, `walk_sel`, `walk_match`, `walk_loop`, `walk_with`, `walk_with_resource`, `walk_with_context`, `walk_refine`, `walk_agent_decl`, `walk_class_decl`, `walk_ternary`), add a call to the post-visit method after all children have been visited.

Verify: `just diagnose` passes. `just test` passes (post hooks are no-ops by default, so existing behavior is unchanged).

## Task 9: Refactor diag_walk to use post-visit hooks

**Subject:** Replace manual save/restore context pattern in diag_walk with post-visit hooks
**ActiveForm:** Refactoring diag_walk to use post-visit hooks

In `crates/lx/src/stdlib/diag/diag_walk.rs`, the `visit_agent_decl` and `visit_binding` (for Func case) methods manually save `self.context` and `self.current_fn` before visiting children and restore them after. This is the pattern that post-visit hooks eliminate.

Refactor `visit_agent_decl`: Move the context restoration (`self.context = saved`) into `visit_agent_decl_post`. The pre-visit sets context, the walk visits children, the post-visit restores context.

In `crates/lx/src/stdlib/diag/diag_walk_expr.rs`, the `visit_expr_diag` function has save/restore for `Par`, `Sel`, `Match`, `Ternary`, `Loop`, `Refine`, and `cron.every` handler. For each of these, the pattern is: save context → set new context → visit children → restore context. Move the restore into the corresponding post-visit hook implementations on Walker.

For nodes where diag_walk overrides `visit_expr` (which dispatches through `visit_expr_diag`), add corresponding `visit_*_post` overrides that restore `self.context` to the saved value. The saved value can be stored in a `Vec<String>` stack field on `Walker` (push in pre-visit, pop in post-visit) instead of local variables.

Verify: `just diagnose` passes. `just test` passes.

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
mcp__workflow__load_work_item({ path: "work_items/VISITOR_PATTERN_CLEANUP.md" })
```

Then call `next_task` to begin. After completing each task's implementation, call `complete_task` to format, commit, and run diagnostics. Repeat until all tasks are done.
