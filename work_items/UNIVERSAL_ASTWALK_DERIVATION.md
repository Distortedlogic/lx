# Goal

Make every AST-adjacent type derive `AstWalk` so the macro generates all traversal code. Replace tuple-pair fields in `WithKind` with named structs. Remove `#[walk(skip)]` from `TraitUnion` and `Use` on the `Stmt` enum. Derive `AstWalk` on all types currently in the `PASSTHROUGH_TYPES` escape hatch that aren't genuine primitives. Delete the hand-written `walk_impls.rs`. Shrink `PASSTHROUGH_TYPES` to only actual primitives.

# Why

- `walk_impls.rs` is 172 lines of hand-maintained `recurse_children`, `children`, and `walk_children` implementations for `WithKind`, `TraitDeclData`, and `ClassDeclData`. This is the exact code that `#[derive(AstWalk)]` generates for every other type. The only reason these can't derive is that `WithKind` contains `Vec<(ExprId, Sym)>` — tuple pairs that the macro's `classify_type` can't handle
- `TraitUnionDef` and `UseStmt` are in the `PASSTHROUGH_TYPES` list and their parent `Stmt` variants are marked `#[walk(skip)]`. If either type gains walkable children in the future, the skip annotation silently suppresses visiting them. The walk code in `walk_stmt` handles them with one-off logic that doesn't call leave hooks
- `PASSTHROUGH_TYPES` contains `UseKind`, `UseStmt`, `StmtTypeDef`, `TraitUnionDef` alongside genuine primitives. These are AST types that should derive `AstWalk` (producing no-op walk code for passthrough-only fields), not be hardcoded as exceptions
- `FieldDecl`, `AgentMethod`, and `ClassField` don't derive `AstWalk` despite having walkable `ExprId` fields, forcing manual implementation in `walk_impls.rs`

# What changes

**Named structs replace tuple pairs:** `WithKind::Resources { resources: Vec<(ExprId, Sym)> }` becomes `Vec<ResourceBinding>` where `ResourceBinding { expr: ExprId, name: Sym }`. `WithKind::Context { fields: Vec<(Sym, ExprId)> }` becomes `Vec<ContextField>` where `ContextField { name: Sym, value: ExprId }`. Both derive `AstWalk`.

**Derive AstWalk on all AST types:** `FieldDecl` (via `Field<D, C>`), `AgentMethod`, `ClassField`, `TraitDeclData`, `ClassDeclData`, `WithKind`, `TraitUnionDef`, `UseStmt`, `UseKind` all get `#[derive(AstWalk)]`. For types with only passthrough fields (no ExprId/StmtId/etc), the generated code is a no-op.

**Remove walk(skip):** `Stmt::TraitUnion` and `Stmt::Use` lose their `#[walk(skip)]` attributes. Add missing `leave_trait_union` and `leave_use` hooks to `AstVisitor`.

**Delete walk_impls.rs:** All manual implementations replaced by macro-generated code.

**Shrink PASSTHROUGH_TYPES:** Remove `UseKind`, `UseStmt`, `TraitUnionDef` from the list. After the AST_TYPE_FIDELITY work item, `StmtTypeDef` is also removed. The list retains only: `Sym`, `BinOp`, `UnaryOp`, `bool`, `i64`, `f64`, `usize`, `String`, `BigInt`.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/ast/mod.rs` | Remove `#[walk(skip)]` from `TraitUnion` and `Use`, add `AstWalk` derive to `WithKind` |
| `crates/lx/src/ast/types.rs` | Add `ResourceBinding`, `ContextField` structs; derive `AstWalk` on `TraitUnionDef`, `UseStmt`, `UseKind`, `FieldDecl` (via `Field`), `AgentMethod`, `ClassField`, `TraitDeclData`, `ClassDeclData` |
| `crates/lx/src/ast/walk_impls.rs` | Delete entirely |
| `crates/lx/src/visitor/visitor_trait.rs` | Add `leave_trait_union`, `leave_use` methods |
| `crates/lx/src/visitor/walk/mod.rs` | Update `walk_stmt` for `TraitUnion` and `Use` to use full dispatch pattern with leave hooks |
| `crates/lx-macros/src/field_strategy.rs` | Remove `UseKind`, `UseStmt`, `TraitUnionDef` from `PASSTHROUGH_TYPES` |
| `crates/lx/src/interpreter/exec_stmt.rs` | Update `WithKind` destructuring for named structs |
| `crates/lx/src/interpreter/mod.rs` | Update `WithKind` destructuring |
| `crates/lx/src/parser/stmt.rs` | Update `WithKind` construction for named structs |
| `crates/lx/src/formatter/emit_stmt.rs` | Update `WithKind` emission |
| `crates/lx/src/checker/visit_stmt.rs` | Update `WithKind` destructuring |
| `crates/lx/src/value/mod.rs` | Split FieldDef and TraitMethodDef from generic type aliases to concrete structs |

# Task List

### Task 1: Define ResourceBinding and ContextField structs

In `crates/lx/src/ast/types.rs`, add after the existing struct definitions:

```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct ResourceBinding {
    pub expr: ExprId,
    pub name: Sym,
}

#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct ContextField {
    pub name: Sym,
    pub value: ExprId,
}
```

Ensure `lx_macros::AstWalk` is imported. Check the existing imports at the top of `types.rs` — `AstWalk` may need to be added.

Add `pub use types::{ResourceBinding, ContextField};` to `ast/mod.rs` if types.rs items are re-exported via `pub use types::*` (they are — line 19).

### Task 2: Update WithKind to use named structs

In `crates/lx/src/ast/mod.rs`, change `WithKind`:

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum WithKind {
    Binding { name: Sym, value: ExprId, mutable: bool },
    Resources { resources: Vec<(ExprId, Sym)> },
    Context { fields: Vec<(Sym, ExprId)> },
}
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum WithKind {
    Binding { name: Sym, value: ExprId, mutable: bool },
    Resources { resources: Vec<ResourceBinding> },
    Context { fields: Vec<ContextField> },
}
```

Add `use super::types::{ResourceBinding, ContextField};` or rely on the wildcard re-export.

Fix every site that constructs or destructures `WithKind::Resources` and `WithKind::Context`. Complete enumeration:

**Construction sites:**

- `crates/lx/src/parser/stmt.rs` — where `WithKind::Resources` and `WithKind::Context` are constructed. Change:
  - `(expr, sym)` tuples to `ResourceBinding { expr, name: sym }`
  - `(sym, expr)` tuples to `ContextField { name: sym, value: expr }`
- `crates/lx/src/ast/walk_impls.rs` — `WithKind::Resources` and `WithKind::Context` in `recurse_children`, `children`, `walk_children` (this file is deleted in Task 8, but needs to compile until then)

**Destructuring sites:**

- `crates/lx/src/interpreter/exec_stmt.rs` — WithKind match arms. Change:
  - `for (e, sym) in resources` to `for rb in resources` accessing `rb.expr`, `rb.name`
  - `for (sym, e) in fields` to `for cf in fields` accessing `cf.name`, `cf.value`
- `crates/lx/src/interpreter/mod.rs` — WithKind match arms
- `crates/lx/src/checker/visit_stmt.rs` — WithKind match arms
- `crates/lx/src/formatter/emit_stmt.rs` — WithKind emission
- `crates/lx/src/folder/desugar.rs` — WithKind::Binding desugaring (only touches Binding variant, not Resources/Context)

### Task 3: Derive AstWalk on FieldDecl, AgentMethod, ClassField

In `crates/lx/src/ast/types.rs`:

`Field<D, C>` is instantiated twice: `FieldDecl = Field<ExprId, ExprId>` (AST, in `types.rs:45`) and `FieldDef = Field<LxVal, ConstraintExpr>` (runtime, in `value/mod.rs:20`). The proc macro cannot derive AstWalk on a generic struct. Split into two concrete structs.

If AST_TYPE_FIDELITY has been completed, FieldDecl already has `type_expr: TypeExprId`. If not, use `type_name: Sym` (passthrough).

**In `crates/lx/src/ast/types.rs`:**
Remove `Field<D, C>` and the `FieldDecl` type alias. Define FieldDecl as a concrete struct:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct FieldDecl {
    pub name: Sym,
    pub type_name: Sym,
    pub default: Option<ExprId>,
    pub constraint: Option<ExprId>,
}
```

**In `crates/lx/src/value/mod.rs`:**
Remove the `FieldDef = Field<LxVal, ConstraintExpr>` type alias. Define FieldDef as a concrete struct (no AstWalk):
```rust
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub name: Sym,
    pub type_name: Sym,
    pub default: Option<LxVal>,
    pub constraint: Option<ConstraintExpr>,
}
```

**Same for MethodSpec<F>:** `MethodSpec<F>` is instantiated as `TraitMethodDecl = MethodSpec<FieldDecl>` (AST) and `TraitMethodDef = MethodSpec<FieldDef>` (runtime, `value/mod.rs:27`). Remove the generic. Define concrete structs:

In `types.rs`:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct TraitMethodDecl {
    pub name: Sym,
    pub input: Vec<FieldDecl>,
    pub output: Sym,
}
```

In `value/mod.rs`:
```rust
#[derive(Debug, Clone)]
pub struct TraitMethodDef {
    pub name: Sym,
    pub input: Vec<FieldDef>,
    pub output: Sym,
}
```

**AgentMethod:**

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct AgentMethod {
    pub name: Sym,
    pub handler: ExprId,
}
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct AgentMethod {
    pub name: Sym,
    pub handler: ExprId,
}
```

**ClassField:**

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ClassField {
    pub name: Sym,
    pub default: ExprId,
}
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct ClassField {
    pub name: Sym,
    pub default: ExprId,
}
```

### Task 4: Derive AstWalk on TraitDeclData and ClassDeclData

In `crates/lx/src/ast/types.rs`:

**TraitDeclData:**

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDeclData {
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct TraitDeclData {
```

For this to work, every field type must be either a known ID type, a Vec/Option of an ID type, a type that derives AstWalk, or in `PASSTHROUGH_TYPES`.

Fields of `TraitDeclData`:
- `name: Sym` — passthrough
- `type_params: Vec<Sym>` — passthrough (Vec of passthrough)
- `entries: Vec<TraitEntry>` — `TraitEntry` must derive AstWalk (see below)
- `methods: Vec<TraitMethodDecl>` — `TraitMethodDecl = MethodSpec<FieldDecl>`, needs AstWalk
- `defaults: Vec<AgentMethod>` — now derives AstWalk (Task 3)
- `requires: Vec<Sym>` — passthrough
- `description: Option<Sym>` — passthrough
- `tags: Vec<Sym>` — passthrough
- `exported: bool` — passthrough

**TraitEntry** must derive AstWalk:

Remove `Box` from `TraitEntry::Field(Box<FieldDecl>)` → `TraitEntry::Field(FieldDecl)`. FieldDecl is ~32 bytes; TraitEntry inline at that size is acceptable for a non-hot-path data structure. This avoids needing to add Box support to the macro. Update all construction sites: `TraitEntry::Field(Box::new(fd))` → `TraitEntry::Field(fd)` and all destructuring sites: `TraitEntry::Field(f)` where `f` was `&Box<FieldDecl>` now becomes `&FieldDecl`.

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum TraitEntry {
    Field(Box<FieldDecl>),
    Spread(Sym),
}
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum TraitEntry {
    Field(FieldDecl),
    Spread(Sym),
}
```

**TraitMethodDecl** already made concrete in Task 3. No additional work needed here.

**ClassDeclData:**

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclData {
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct ClassDeclData {
```

All fields: `name: Sym`, `type_params: Vec<Sym>`, `traits: Vec<Sym>`, `fields: Vec<ClassField>` (now AstWalk), `methods: Vec<AgentMethod>` (now AstWalk), `exported: bool`. All handled.

### Task 5: Derive AstWalk on TraitUnionDef, UseStmt, UseKind

In `crates/lx/src/ast/types.rs`:

**TraitUnionDef:**

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TraitUnionDef {
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct TraitUnionDef {
```

All fields are passthrough (Sym, Vec<Sym>, bool). Generated code will be no-ops.

**UseStmt:**

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct UseStmt {
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct UseStmt {
```

Fields: `path: Vec<Sym>`, `kind: UseKind`. Both passthrough once UseKind derives AstWalk.

**UseKind:**

From:
```rust
#[derive(Debug, Clone, PartialEq)]
pub enum UseKind {
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub enum UseKind {
```

All variants contain only `Sym` or `Vec<Sym>`. Generated code will be no-ops.

### Task 6: Remove walk(skip) from Stmt::TraitUnion and Stmt::Use, add leave hooks

In `crates/lx/src/ast/mod.rs`:

Remove `#[walk(skip)]` from both:

```rust
#[walk(skip)]
TraitUnion(TraitUnionDef),
```
becomes:
```rust
TraitUnion(TraitUnionDef),
```

```rust
#[walk(skip)]
Use(UseStmt),
```
becomes:
```rust
Use(UseStmt),
```

In `crates/lx/src/visitor/visitor_trait.rs`:

Add after `visit_trait_union` (around line 37):
```rust
fn leave_trait_union(&mut self, _id: StmtId, _def: &TraitUnionDef, _span: SourceSpan) {}
```

Add after `visit_use` (around line 43):
```rust
fn leave_use(&mut self, _id: StmtId, _stmt: &UseStmt, _span: SourceSpan) {}
```

In `crates/lx/src/visitor/walk/mod.rs`:

Update the `Stmt::TraitUnion` arm in `walk_stmt` (around lines 125-129):

From:
```rust
Stmt::TraitUnion(def) => {
    let action = v.visit_trait_union(id, def, span);
    if action.is_stop() {
        return ControlFlow::Break(());
    }
},
```

To:
```rust
Stmt::TraitUnion(def) => {
    let action = v.visit_trait_union(id, def, span);
    match action {
        VisitAction::Stop => return ControlFlow::Break(()),
        VisitAction::Skip => {},
        VisitAction::Descend => {
            def.walk_children(v, arena)?;
        },
    }
    v.leave_trait_union(id, def, span);
},
```

Update the `Stmt::Use` arm similarly (around lines 134-138):

From:
```rust
Stmt::Use(use_stmt) => {
    let action = v.visit_use(id, use_stmt, span);
    if action.is_stop() {
        return ControlFlow::Break(());
    }
},
```

To:
```rust
Stmt::Use(use_stmt) => {
    let action = v.visit_use(id, use_stmt, span);
    match action {
        VisitAction::Stop => return ControlFlow::Break(()),
        VisitAction::Skip => {},
        VisitAction::Descend => {
            use_stmt.walk_children(v, arena)?;
        },
    }
    v.leave_use(id, use_stmt, span);
},
```

### Task 7: Update PASSTHROUGH_TYPES

In `crates/lx-macros/src/field_strategy.rs`:

Change the `PASSTHROUGH_TYPES` array (line 48-49):

From:
```rust
const PASSTHROUGH_TYPES: &[&str] =
    &["Sym", "BinOp", "UnaryOp", "bool", "i64", "f64", "usize", "String", "BigInt", "UseKind", "UseStmt", "StmtTypeDef", "TraitUnionDef"];
```

To:
```rust
const PASSTHROUGH_TYPES: &[&str] =
    &["Sym", "BinOp", "UnaryOp", "bool", "i64", "f64", "usize", "String", "BigInt"];
```

All removed types now derive `AstWalk` and will be classified as `WalkableStruct` by the macro. Their generated walk code is no-ops since they contain only passthrough fields.

### Task 8: Delete walk_impls.rs

Delete `crates/lx/src/ast/walk_impls.rs` entirely.

In `crates/lx/src/ast/mod.rs`, remove the `mod walk_impls;` declaration (line 7).

The macro-generated `recurse_children`, `children`, and `walk_children` on `WithKind`, `TraitDeclData`, and `ClassDeclData` now replace all the hand-written implementations.

Verify: the two helper functions `recurse_field_decl` and `recurse_agent_methods` at the bottom of `walk_impls.rs` are no longer needed — the macro generates per-field recursion inline. `recurse_field_decl` and `recurse_agent_methods` are file-local functions only called within `walk_impls.rs`. No external callers exist.

### Task 9: Compile, format, and verify

Run `just fmt` to format all changed files.

Run `just diagnose`. Fix all compilation errors. Common issues:
- Tuple destructuring sites missed in Task 2 — the compiler will identify them.
- Missing imports for `ResourceBinding`/`ContextField` at use sites.

Run `just test` to verify all existing tests pass.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Do not add, skip, reorder, or combine tasks.**
3. **Generic structs cannot derive AstWalk** — the proc macro operates on concrete syntax, not monomorphized types. Replace `Field<D, C>` and `MethodSpec<F>` with concrete structs before deriving.
4. **The macro crate must compile before the main crate.** Changes to `lx-macros` are picked up automatically by cargo's dependency tracking.
5. **After deleting `walk_impls.rs`**, any leftover references to `recurse_field_decl` or `recurse_agent_methods` will cause compile errors. Search for these function names and remove calls.
6. If AST_TYPE_FIDELITY has not been completed, FieldDecl will have `type_name: Sym` instead of `type_expr: TypeExprId`. The macro will generate correct code either way — `Sym` is passthrough and `TypeExprId` is walkable.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/UNIVERSAL_ASTWALK_DERIVATION.md" })
```

Then call `next_task` to begin.
