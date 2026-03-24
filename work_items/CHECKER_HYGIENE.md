# Goal

Replace `HashMap<ExprId, TypeId>` in the checker with `ArenaMap` for O(1) non-hashing lookups, and make the checker's expression dispatch exhaustive so new AST variants cause compile errors instead of silently falling through.

# Why

- `ExprId` is an `la_arena::Idx` — a dense integer index. Using `HashMap` for it incurs hashing overhead and poor cache locality. `la_arena::ArenaMap` (available in la_arena 0.3.1, already a workspace dependency) is specifically designed for mapping arena indices to values with O(1) lookup and no hashing
- The checker's `check_expr` method (in `check_expr.rs` line 46) has a `_ =>` catch-all that falls through to `synth_expr`. If a new Expr variant is added, it silently falls through to synthesis instead of producing a compile error

# ArenaMap API differences from HashMap

The la_arena 0.3.1 `ArenaMap` API takes `Idx<T>` by value (not by reference, since Idx is Copy):

- `HashMap::get(&id)` → `ArenaMap::get(id)` — pass by value, not reference
- `HashMap::contains_key(&id)` → `ArenaMap::contains_idx(id)` — different method name AND pass by value
- `HashMap::insert(id, v)` → `ArenaMap::insert(id, v)` — same signature
- `HashMap::new()` → `ArenaMap::default()` — use Default trait
- Indexing `map[id]` works on both

# Verified catch-all locations

The following match statements have wildcard catch-all arms that must be made exhaustive:

1. **`check_expr.rs` line 46** — `check_expr()` matches Func, List, Match, Block; `_ =>` falls through to synth_expr
2. **`check_expr.rs` line 136** — `check_block()` matches `Stmt::Expr(e)`; `_ =>` falls through to check_stmt
3. **`module_graph.rs` line 91** — `extract_signature()` matches Binding, TypeDef, TraitDecl, ClassDecl with guards; `_ => {}` silently ignores other variants

`synth_expr` (defined in `type_ops.rs` lines 11-139 as `synth_expr_inner`) is **already exhaustive** — it explicitly handles all 28 Expr variants with no catch-all. No changes needed there.

`check_stmt` (in `visit_stmt.rs`) is **already exhaustive** — all Stmt variants handled. No changes needed.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/checker/mod.rs` | Change expr_types from HashMap to ArenaMap, update all access sites |
| `crates/lx/src/checker/semantic.rs` | Change expr_types from HashMap to ArenaMap in SemanticModel and build() |
| `crates/lx/src/checker/module_graph.rs` | Update expr_types access from `.get(&id)` to `.get(id)`, make extract_signature exhaustive |
| `crates/lx/src/checker/check_expr.rs` | Make check_expr and check_block exhaustive |

# Task List

### Task 1: Replace HashMap<ExprId, TypeId> with ArenaMap in Checker

In `crates/lx/src/checker/mod.rs`:

Add `use la_arena::ArenaMap;` to imports.

Change `expr_types: HashMap<ExprId, TypeId>` (line 73) to `expr_types: ArenaMap<ExprId, TypeId>`.

Change initialization in `Checker::new()` (line 92) from `expr_types: HashMap::new()` to `expr_types: ArenaMap::default()`.

The `record_type` method (line 99-101) calls `self.expr_types.insert(id, ty)` — ArenaMap::insert has the same signature, no change needed.

### Task 2: Replace HashMap<ExprId, TypeId> with ArenaMap in SemanticModel

In `crates/lx/src/checker/semantic.rs`:

The `SemanticModel` struct (line 60) has `pub expr_types: HashMap<ExprId, TypeId>`. Change to `pub expr_types: ArenaMap<ExprId, TypeId>`.

Add `use la_arena::ArenaMap;` to imports.

The `build()` method receives `expr_types: HashMap<ExprId, TypeId>` — change parameter type to `ArenaMap<ExprId, TypeId>`.

The `type_of_expr` method (line 68) calls `self.expr_types.get(&id).copied()` — change to `self.expr_types.get(id).copied()` (remove the `&`).

In `crates/lx/src/checker/module_graph.rs`, find the access `semantic.expr_types.get(&b.value).copied()` (line 75) — change to `semantic.expr_types.get(b.value).copied()`.

Search with `rg --type rust 'expr_types' crates/lx/src/` to find any other access sites and update `.get(&id)` to `.get(id)` everywhere.

### Task 3: Make check_expr dispatch exhaustive

In `crates/lx/src/checker/check_expr.rs`, replace the `_ =>` catch-all (line 46) with explicit arms for every remaining Expr variant:

```rust
Expr::Literal(_)
| Expr::Ident(_)
| Expr::TypeConstructor(_)
| Expr::Binary(_)
| Expr::Unary(_)
| Expr::Pipe(_)
| Expr::Apply(_)
| Expr::Section(_)
| Expr::FieldAccess(_)
| Expr::Tuple(_)
| Expr::Record(_)
| Expr::Map(_)
| Expr::Ternary(_)
| Expr::Propagate(_)
| Expr::Coalesce(_)
| Expr::Slice(_)
| Expr::NamedArg(_)
| Expr::Loop(_)
| Expr::Break(_)
| Expr::Assert(_)
| Expr::Par(_)
| Expr::Sel(_)
| Expr::Timeout(_)
| Expr::Emit(_)
| Expr::Yield(_)
| Expr::With(_) => {
    // same body as current _ => arm
}
```

In the same file, replace the `_ =>` catch-all in `check_block` (line 136) with explicit arms:

```rust
Stmt::Binding(_)
| Stmt::TypeDef(_)
| Stmt::TraitUnion(_)
| Stmt::TraitDecl(_)
| Stmt::ClassDecl(_)
| Stmt::FieldUpdate(_)
| Stmt::Use(_) => {
    self.check_stmt(last, arena);
    self.type_arena.unit()
}
```

### Task 4: Make extract_signature exhaustive

In `crates/lx/src/checker/module_graph.rs`, replace the `_ => {}` catch-all (line 91) in `extract_signature` with explicit arms:

```rust
Stmt::TraitUnion(_)
| Stmt::FieldUpdate(_)
| Stmt::Use(_)
| Stmt::Expr(_) => {}
```

Also add the non-guarded fallthrough for Binding, TypeDef, TraitDecl, ClassDecl (the existing arms have guards like `if b.exported` — the non-exported case needs explicit handling too). Verify each guarded arm has a fallthrough or is covered by the explicit no-op arms.

### Task 5: Format and commit

Run `just fmt` then `git add -A && git commit -m "refactor: use ArenaMap for expr_types, make checker dispatch exhaustive"`.

### Task 6: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **ArenaMap::get takes Idx by value, not by reference.** Every `.get(&id)` must become `.get(id)`.
5. **ArenaMap uses `contains_idx` not `contains_key`.** Search for any `contains_key` calls and rename.
6. **Do not use `#[allow(unreachable_patterns)]`** — the point is compile-time exhaustiveness checking.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/CHECKER_HYGIENE.md" })
```

Then call `next_task` to begin.
