# Goal

Replace lossy scalar representations of type information in the AST with proper arena-allocated type expressions. `FieldDecl.type_name: Sym` becomes `FieldDecl.type_expr: TypeExprId`, and `StmtTypeDef.variants: Vec<(Sym, usize)>` becomes `Vec<TypeVariant>` with fully resolved type parameters. This makes trait field types and type definition variants traversable, span-tracked, and composable through the standard visitor/transformer/checker infrastructure.

# Why

- `FieldDecl.type_name: Sym` stores type annotations as bare interned strings. Everywhere else in the AST — `Param.type_ann: Option<TypeExprId>`, `Binding.type_ann: Option<TypeExprId>`, `ExprFunc.ret_type: Option<TypeExprId>` — type annotations are arena-allocated `TypeExpr` nodes with spans. Trait field types are the only exception. This means the typechecker cannot resolve applied types (`List[Int]`), record types, function types, or fallible types in trait field positions — `named_to_type()` in `checker/mod.rs` only handles six hardcoded primitive names and falls back to `unknown()` for everything else
- `StmtTypeDef.variants: Vec<(Sym, usize)>` stores only a constructor name and argument count. The typechecker cannot validate that constructor arguments match declared type parameter kinds, cannot report spans on individual type arguments, and cannot distinguish `Maybe[Int]` from `Maybe[Str]` at the definition site
- Both types are marked `#[walk(skip)]` on the Stmt enum, meaning visitor passes silently ignore their contents. Any future analysis pass (e.g., "find all references to type X") would miss these sites

# What changes

**FieldDecl:** `type_name: Sym` becomes `type_expr: TypeExprId`. The parser allocates a `TypeExpr::Named(sym)` in the arena for simple names (the common case) and a full `TypeExpr` tree for complex types. The checker resolves via the standard type expression path instead of `named_to_type()`. The formatter emits via `emit_type_expr()` instead of `write(type_name.as_str())`. The interpreter resolves type names from the arena instead of passing raw Syms.

**Generic split:** `Field<D, C>` is instantiated twice: `FieldDecl = Field<ExprId, ExprId>` (AST) and `FieldDef = Field<LxVal, ConstraintExpr>` (runtime, in `value/mod.rs:20`). Same for `MethodSpec<F>`: `TraitMethodDecl = MethodSpec<FieldDecl>` (AST) and `TraitMethodDef = MethodSpec<FieldDef>` (runtime, `value/mod.rs:27`). Changing `type_name` to `type_expr` on the generic breaks the runtime side. The generic is split into two concrete structs: `FieldDecl` (AST, with `type_expr: TypeExprId`) and `FieldDef` (runtime, keeps `type_name: Sym`). Same for `TraitMethodDecl` and `TraitMethodDef`.

**StmtTypeDef:** `variants: Vec<(Sym, usize)>` becomes `variants: Vec<TypeVariant>` where `TypeVariant { name: Sym, fields: Vec<TypeExprId> }`. The parser allocates type parameters as `TypeExpr` nodes. The formatter emits via `emit_type_expr()` per field. The interpreter accesses `variant.fields.len()` for arity. `#[walk(skip)]` is removed from `Stmt::TypeDef` since the new struct has walkable children.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/ast/types.rs` | Change `Field.type_name` to `type_expr: TypeExprId`, add `TypeVariant` struct, change `StmtTypeDef.variants` |
| `crates/lx/src/ast/mod.rs` | Remove `#[walk(skip)]` from `Stmt::TypeDef`, derive AstWalk on StmtTypeDef |
| `crates/lx/src/parser/stmt.rs` | Allocate `TypeExpr` nodes for field types and variant parameters |
| `crates/lx/src/parser/stmt_class.rs` | Update FieldDecl construction to use TypeExprId |
| `crates/lx/src/checker/visit_stmt.rs` | Resolve `FieldDecl.type_expr` via standard type expression path instead of `named_to_type()` |
| `crates/lx/src/checker/mod.rs` | Remove or simplify `named_to_type()` |
| `crates/lx/src/formatter/emit_stmt.rs` | Emit type expressions via `emit_type_expr()` |
| `crates/lx/src/interpreter/exec_stmt.rs` | Access `variant.fields.len()` for arity, resolve type_expr from arena |
| `crates/lx/src/interpreter/type_apply.rs` | Resolve `type_expr` from arena instead of passing raw Sym |
| `crates/lx/src/ast/walk_impls.rs` | Update `TraitDeclData.recurse_children`/`children`/`walk_children` for new FieldDecl shape |
| `crates/lx/src/visitor/visitor_trait.rs` | Add `leave_type_def` method |
| `crates/lx-macros/src/field_strategy.rs` | Remove `StmtTypeDef` from `PASSTHROUGH_TYPES` |
| `crates/lx/src/value/mod.rs` | Split `FieldDef` and `TraitMethodDef` from generic type aliases to concrete structs |

# Task List

### Task 1: Define TypeVariant struct and split Field/MethodSpec generics

In `crates/lx/src/ast/types.rs` (which already has `use lx_macros::AstWalk;` at line 1):

Add a new struct after `StmtTypeDef`:

```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct TypeVariant {
    pub name: Sym,
    pub fields: Vec<TypeExprId>,
}
```

Split `Field<D, C>` into a concrete `FieldDecl` struct (AST side). Remove the generic `Field<D, C>` and the `FieldDecl` type alias. Replace with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: Sym,
    pub type_expr: TypeExprId,
    pub default: Option<ExprId>,
    pub constraint: Option<ExprId>,
}
```

Leave the runtime `FieldDef` in `value/mod.rs` — it keeps `type_name: Sym` (see Task 4).

Split `MethodSpec<F>` into a concrete `TraitMethodDecl` struct (AST side). Remove the generic `MethodSpec<F>` and the `TraitMethodDecl` type alias. Replace with:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TraitMethodDecl {
    pub name: Sym,
    pub input: Vec<FieldDecl>,
    pub output: Option<TypeExprId>,
}
```

Leave the runtime `TraitMethodDef` in `value/mod.rs` — it keeps its own concrete struct (see Task 4).

Change `StmtTypeDef`:

From:
```rust
pub struct StmtTypeDef {
    pub name: Sym,
    pub type_params: Vec<Sym>,
    pub variants: Vec<(Sym, usize)>,
    pub exported: bool,
}
```

To:
```rust
#[derive(Debug, Clone, PartialEq, AstWalk)]
pub struct StmtTypeDef {
    pub name: Sym,
    pub type_params: Vec<Sym>,
    pub variants: Vec<TypeVariant>,
    pub exported: bool,
}
```

Remove the existing `#[derive(Debug, Clone, PartialEq)]` on StmtTypeDef and replace with the version that includes `AstWalk`.

### Task 2: Remove walk(skip) from Stmt::TypeDef and update PASSTHROUGH_TYPES

In `crates/lx/src/ast/mod.rs`, remove the `#[walk(skip)]` attribute from `Stmt::TypeDef(StmtTypeDef)`:

From:
```rust
#[walk(skip)]
TypeDef(StmtTypeDef),
```

To:
```rust
TypeDef(StmtTypeDef),
```

In `crates/lx-macros/src/field_strategy.rs`, remove `"StmtTypeDef"` from the `PASSTHROUGH_TYPES` array (line 48-49). It will now be treated as a `WalkableStruct` since it derives `AstWalk`.

In `crates/lx/src/visitor/visitor_trait.rs`, add a `leave_type_def` method to the `AstVisitor` trait after `visit_type_def` (around line 27):

```rust
fn leave_type_def(&mut self, _id: StmtId, _def: &StmtTypeDef, _span: SourceSpan) {}
```

In `crates/lx/src/visitor/walk/mod.rs`, update the `Stmt::TypeDef` arm in `walk_stmt` (lines 119-124). Currently it just calls `visit_type_def` and checks for Stop. Change it to use the full dispatch pattern with walk_children and leave:

From:
```rust
Stmt::TypeDef(def) => {
    let action = v.visit_type_def(id, def, span);
    if action.is_stop() {
        return ControlFlow::Break(());
    }
},
```

To:
```rust
Stmt::TypeDef(def) => {
    let action = v.visit_type_def(id, def, span);
    match action {
        VisitAction::Stop => return ControlFlow::Break(()),
        VisitAction::Skip => {},
        VisitAction::Descend => {
            def.walk_children(v, arena)?;
        },
    }
    v.leave_type_def(id, def, span);
},
```

### Task 3: Update parser to allocate TypeExpr nodes for field types

In `crates/lx/src/parser/stmt.rs`:

The parser has `type_ann::type_parser()` in `type_ann.rs` which returns `TypeExprId` via `arena.borrow_mut().alloc_type_expr(...)`. Use this parser for trait field types instead of the current `type_name()` ident parser.

For variant parsing: the current `type_def_parser` at lines 162-180 uses `any().filter(...).repeated().collect::<Vec<_>>().map(|toks| toks.len())` to count tokens. Replace this with: for each token, parse as type_name, allocate `TypeExpr::Named(sym)` via `arena.borrow_mut().alloc_type_expr(TypeExpr::Named(sym), ss(ctx.span()))`.

The StmtTypeDef construction (line 69) changes from passing raw `variants` to passing `Vec<TypeVariant>`.

For `FieldDecl` construction in the trait body parser (around line 237):

From:
```rust
.map(|((name, typ), default)| TraitBodyItem::Field(FieldDecl {
    name, type_name: typ, default, constraint: None
}))
```

To:
```rust
.map(|((name, typ_id), default)| TraitBodyItem::Field(FieldDecl {
    name, type_expr: typ_id, default, constraint: None
}))
```

The `typ` value must now be a `TypeExprId` instead of `Sym`. Replace the ident-based type parser with `type_ann::type_parser()` for trait field types.

### Task 4: Update value/mod.rs for runtime FieldDef and TraitMethodDef

**No changes needed in `stmt_class.rs`.** `FieldDecl` is only constructed in `stmt.rs:237`. `stmt_class.rs` constructs `ClassField` (a different struct) and does not use `FieldDecl`. However, `value/mod.rs` needs updating: change `FieldDef` from `Field<LxVal, ConstraintExpr>` type alias to a concrete struct with `type_name: Sym` (keeping the original field name). Change `TraitMethodDef` from `MethodSpec<FieldDef>` to a concrete struct.

### Task 5: Update checker to resolve type_expr via standard path

In `crates/lx/src/checker/visit_stmt.rs` (around lines 56-60):

Currently trait field types are resolved via `named_to_type()`:
```rust
let ty = self.named_to_type(f.type_name);
```

Change to resolve through the standard type expression infrastructure. The checker should already have a method for resolving `TypeExprId` to `TypeId` (used for `Binding.type_ann` and `Param.type_ann`). Use that same method:

```rust
let ty = self.resolve_type_expr(f.type_expr);
```

The checker resolves type annotations via `self.resolve_type_ann(ann)` (at `visit_stmt.rs:199`). This method already handles all TypeExpr variants. For `TypeExpr::Named`, it calls `self.named_to_type(name.as_str())`. Since FieldDecl.type_expr is now a TypeExprId, call `self.resolve_type_ann(f.type_expr)` directly.

In `crates/lx/src/checker/mod.rs`:

The `named_to_type()` function (around lines 213-223) maps six hardcoded string names to types. After this change, it may have no remaining callers. If so, remove it. If other code still uses it, leave it but add the method input FieldDecl resolution to use the standard path.

### Task 6: Update formatter for type_expr emission

In `crates/lx/src/formatter/emit_stmt.rs`:

For trait field type emission (around lines 99-107), change:
```rust
self.write(f.type_name.as_str());
```
To:
```rust
self.emit_type_expr(f.type_expr);
```

The formatter has `emit_type_expr(id: TypeExprId)` at `emit_type.rs:6`. Call `self.emit_type_expr(f.type_expr)` directly.

For method input type emission (around lines 118-126), apply the same change: replace `input.type_name.as_str()` with `self.emit_type_expr(input.type_expr)`.

For type definition variant emission (around lines 53-69), change:
```rust
for (name, arity) in &td.variants {
    self.write("| ");
    self.write(name.as_str());
    for _ in 0..*arity { self.write(" _"); }
}
```
To:
```rust
for variant in &td.variants {
    self.write("| ");
    self.write(variant.name.as_str());
    for &field_type in &variant.fields {
        self.write(" ");
        self.emit_type_expr(field_type);
    }
}
```

### Task 7: Update interpreter for new types

In `crates/lx/src/interpreter/exec_stmt.rs` (around line 70):

Change the `StmtTypeDef` destructuring:
```rust
Stmt::TypeDef(StmtTypeDef { variants, .. }) => {
    for (ctor_name, arity) in variants { ... }
}
```
To:
```rust
Stmt::TypeDef(StmtTypeDef { variants, .. }) => {
    for variant in variants {
        let ctor_name = variant.name;
        let arity = variant.fields.len();
        // ... rest of body unchanged, using ctor_name and arity
    }
}
```

In `crates/lx/src/interpreter/type_apply.rs` (around line 55):

Runtime `FieldDef` keeps `type_name: Sym`. The interpreter constructs `FieldDef` from `FieldDecl` by extracting the Sym from the arena: `let type_name = match self.arena.type_expr(f.type_expr) { TypeExpr::Named(sym) => *sym, _ => f.name };`. Use `LxError::runtime(msg, span)` for errors.

In `crates/lx/src/interpreter/exec_stmt.rs` (around line 93):

Same pattern — change `type_name: f.type_name` to extract the sym from the arena.

### Task 8: Update walk_impls.rs for new FieldDecl shape

In `crates/lx/src/ast/walk_impls.rs`:

The `TraitDeclData` manual walk implementations reference `FieldDecl` fields. Since `type_name: Sym` was passthrough (not walked) and `type_expr: TypeExprId` IS walked, the manual impls need updating.

In `TraitDeclData.recurse_children` (around line 57):
Add `type_expr: walk_transform_type_expr(t, field.type_expr, arena)` to the `recurse_field_decl` helper function (line 160-167).

In `TraitDeclData.children` (around line 83-101):
Add `result.push(NodeId::TypeExpr(f.type_expr))` for each field's type expression.

In `TraitDeclData.walk_children` (around line 103-128):
Add `walk_type_expr_dispatch(v, f.type_expr, arena)?` for each field's type expression. Add the necessary import for `walk_type_expr_dispatch`.

Apply the same changes to the method input FieldDecl handling in the same functions.

The helper `recurse_field_decl` (lines 160-167) changes from:
```rust
fn recurse_field_decl<T: AstTransformer + ?Sized>(t: &mut T, field: &FieldDecl, arena: &mut AstArena) -> FieldDecl {
    FieldDecl {
        name: field.name,
        type_name: field.type_name,
        default: field.default.map(|d| walk_transform_expr(t, d, arena)),
        constraint: field.constraint.map(|c| walk_transform_expr(t, c, arena)),
    }
}
```
To:
```rust
fn recurse_field_decl<T: AstTransformer + ?Sized>(t: &mut T, field: &FieldDecl, arena: &mut AstArena) -> FieldDecl {
    FieldDecl {
        name: field.name,
        type_expr: walk_transform_type_expr(t, field.type_expr, arena),
        default: field.default.map(|d| walk_transform_expr(t, d, arena)),
        constraint: field.constraint.map(|c| walk_transform_expr(t, c, arena)),
    }
}
```

Add `use crate::visitor::walk_transform::walk_transform_type_expr;` to the imports at the top of `walk_impls.rs`.

### Task 9: Compile, format, and verify

Run `just fmt` to format all changed files.

Run `just diagnose` to compile and lint. Fix all errors — the compiler will identify any remaining sites that reference `type_name` or the old `(Sym, usize)` variant shape. Follow each error to its source and update.

Run `just test` to verify all existing tests pass. Fix any failures caused by the structural changes.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Do not add, skip, reorder, or combine tasks.**
3. **`Field<D, C>` is split** — the generic is replaced by concrete `FieldDecl` (AST, with `type_expr: TypeExprId`) and `FieldDef` (runtime, keeps `type_name: Sym`). Same for `MethodSpec<F>` -> `TraitMethodDecl` / `TraitMethodDef`.
4. **Search globally for `type_name`** after Task 1 — the compiler will catch struct field accesses but grep will catch string-based references in tests or documentation.
5. **The type annotation parser in `type_ann.rs`** likely already produces `TypeExprId`. Reuse it for trait field types instead of building a new parser.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/AST_TYPE_FIDELITY.md" })
```

Then call `next_task` to begin.
