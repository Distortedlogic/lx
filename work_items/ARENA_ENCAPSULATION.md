# Goal

Make `AstArena` fields private (`pub(crate)`) so all access goes through typed accessor methods, preventing external code from coupling to the internal arena storage layout.

# Why

- The four `Arena` fields in `AstArena` (`exprs`, `stmts`, `patterns`, `type_exprs`) are `pub`, allowing any code to bypass the typed accessor methods and directly index into arenas. If the storage strategy changes, every direct access site breaks
- The accessor methods (`expr()`, `stmt()`, `alloc_expr()`, etc.) provide a stable interface — direct field access circumvents it

# Verified violations

There are exactly **4 direct field accesses** outside of `arena.rs` itself, all in `crates/lx/src/ast/parent_map.rs`:

1. **Line 9**: `arena.stmts.iter()` — iterates all stmts to build parent map
2. **Line 15**: `arena.exprs.iter()` — iterates all exprs to build parent map
3. **Line 21**: `arena.patterns.iter()` — iterates all patterns to build parent map
4. **Line 27**: `arena.type_exprs.iter()` — iterates all type_exprs to build parent map

All four use `.iter()` which returns `(Idx<T>, &T)` pairs. No existing accessor method provides this. Four new iterator methods are needed.

All accesses within `arena.rs` itself are legitimate — they implement the accessor methods.

# What changes

1. Add 4 iterator accessor methods to `AstArena`
2. Change the 4 field accesses in `parent_map.rs` to use the new methods
3. Change all four field visibilities from `pub` to `pub(crate)`

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/ast/arena.rs` | Add 4 iterator methods, change field visibility |
| `crates/lx/src/ast/parent_map.rs` | Replace 4 direct field accesses with iterator methods |

# Task List

### Task 1: Add iterator accessor methods

In `crates/lx/src/ast/arena.rs`, add these 4 methods to the `impl AstArena` block:

```rust
pub fn iter_exprs(&self) -> impl Iterator<Item = (ExprId, &Spanned<Expr>)> {
    self.exprs.iter()
}

pub fn iter_stmts(&self) -> impl Iterator<Item = (StmtId, &Spanned<Stmt>)> {
    self.stmts.iter()
}

pub fn iter_patterns(&self) -> impl Iterator<Item = (PatternId, &Spanned<Pattern>)> {
    self.patterns.iter()
}

pub fn iter_type_exprs(&self) -> impl Iterator<Item = (TypeExprId, &Spanned<TypeExpr>)> {
    self.type_exprs.iter()
}
```

### Task 2: Update parent_map.rs

In `crates/lx/src/ast/parent_map.rs`, replace:

- `arena.stmts.iter()` → `arena.iter_stmts()`
- `arena.exprs.iter()` → `arena.iter_exprs()`
- `arena.patterns.iter()` → `arena.iter_patterns()`
- `arena.type_exprs.iter()` → `arena.iter_type_exprs()`

### Task 3: Change field visibility

In `crates/lx/src/ast/arena.rs`, change all four fields from `pub` to `pub(crate)`:

```rust
pub struct AstArena {
    pub(crate) exprs: Arena<Spanned<Expr>>,
    pub(crate) stmts: Arena<Spanned<Stmt>>,
    pub(crate) patterns: Arena<Spanned<Pattern>>,
    pub(crate) type_exprs: Arena<Spanned<TypeExpr>>,
}
```

### Task 4: Format and commit

Run `just fmt` then `git add -A && git commit -m "refactor: make AstArena fields pub(crate), add accessor methods"`.

### Task 5: Verify

Run `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/ARENA_ENCAPSULATION.md" })
```

Then call `next_task` to begin.
