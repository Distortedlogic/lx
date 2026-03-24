# Goal

Replace bare `usize` type aliases `ScopeId` and `DefinitionId` with newtype structs, and add reverse-lookup indexes to `SemanticModel` so `references_to()` is O(1) instead of O(n).

# Prerequisites

None.

# Why

- `pub type ScopeId = usize` and `pub type DefinitionId = usize` allow silent cross-domain mixing — passing a DefinitionId where a ScopeId is expected compiles without error
- `references_to(def)` iterates all references with `.filter()`, O(n) in reference count

# Verified callers

**`references_to`** — 1 caller:
- `linter/rules/unused_import.rs` line 59: `let refs = model.references_to(def_id);` then `refs.is_empty()`. Return type change from `Vec<ExprId>` to `&[ExprId]` is compatible — `.is_empty()` works on slices.
- Note: `def_id` comes from `enumerate()` on line 56: `model.definitions.iter().enumerate().find(...)`. The `enumerate()` yields `(usize, &DefinitionInfo)`. After newtype change, this raw `usize` must be wrapped: `DefinitionId::new(def_id)`.

**`names_in_scope`** — 1 caller:
- `checker/type_ops.rs` line 27: `let scope_names = self.sem.names_in_scope();`. This method is on `SemanticModelBuilder` (which has `scope_stack`), NOT on `SemanticModel`. The builder already has the `scope_definitions` index after this change.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/checker/semantic.rs` | Replace type aliases with newtypes, add indexes, update all methods |
| `crates/lx/src/linter/rules/unused_import.rs` | Wrap enumerate index: `DefinitionId::new(def_id)` on line 58 |

# Task List

### Task 1: Replace ScopeId type alias with newtype

In `crates/lx/src/checker/semantic.rs`, replace:

```rust
pub type ScopeId = usize;
```

with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeId(u32);

impl ScopeId {
    pub fn new(id: usize) -> Self {
        Self(id as u32)
    }
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
```

Update all usage sites in the same file:
- `Scope::parent: Option<ScopeId>` — already ScopeId, no change to the type
- `DefinitionInfo::scope: ScopeId` — already ScopeId, no change to the type
- `scope_stack: Vec<ScopeId>` — already Vec<ScopeId>, no change to the type
- `def_lookup: HashMap<(ScopeId, Sym), DefinitionId>` — already (ScopeId, Sym) key, no change
- `SemanticModelBuilder::new()` line 102-103: `scope_stack: vec![0]` → `scope_stack: vec![ScopeId::new(0)]`
- `push_scope` line 107: `let id = self.scopes.len();` → `let id = ScopeId::new(self.scopes.len());`
- `push_scope` line 109: `self.scopes.push(Scope { parent, span, kind });` — `parent` is already `Some(self.current_scope())` which returns ScopeId. No change.
- `push_scope` line 110: `self.scope_stack.push(id);` — `id` is now ScopeId. No change.

### Task 2: Replace DefinitionId type alias with newtype

In the same file, replace:

```rust
pub type DefinitionId = usize;
```

with:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefinitionId(u32);

impl DefinitionId {
    pub fn new(id: usize) -> Self {
        Self(id as u32)
    }
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
```

Update all usage sites in the same file:
- `add_definition` line 123: `let id = self.definitions.len();` → `let id = DefinitionId::new(self.definitions.len());`
- `set_definition_type` line 131: `self.definitions[id].ty = ...` → `self.definitions[id.index()].ty = ...`
- `resolve_in_scope` return type is already `Option<DefinitionId>` — no change
- `resolve_in_scope` body: `self.def_lookup.get(&(scope_id, name))` returns `Option<&DefinitionId>` — no change since DefinitionId is Copy
- `SemanticModel::type_of_def` line 73: `self.definitions[id].ty` → `self.definitions[id.index()].ty`
- `SemanticModel::references_to` line 82: `r.definition == def` — PartialEq is derived. No change.

Search with `rg --type rust 'definitions\[' crates/lx/src/checker/` for any direct index access that needs `.index()`.

### Task 3: Add reverse-lookup index for references

Add field to `SemanticModel`:

```rust
pub def_references: HashMap<DefinitionId, Vec<ExprId>>,
```

Add field to `SemanticModelBuilder`:

```rust
def_references: HashMap<DefinitionId, Vec<ExprId>>,
```

Initialize as `HashMap::new()` in `SemanticModelBuilder::new()`.

In `add_reference` (line 134), add:

```rust
self.def_references.entry(def_id).or_default().push(expr_id);
```

Update `build()` to pass `def_references: self.def_references` into SemanticModel.

Replace `references_to` on SemanticModel (line 81-83):

```rust
pub fn references_to(&self, def: DefinitionId) -> &[ExprId] {
    self.def_references.get(&def).map(|v| v.as_slice()).unwrap_or(&[])
}
```

### Task 4: Update names_in_scope on SemanticModelBuilder

Add field to `SemanticModelBuilder`:

```rust
scope_definitions: HashMap<ScopeId, Vec<DefinitionId>>,
```

Initialize as `HashMap::new()` in `new()`.

In `add_definition`, after `self.def_lookup.insert((scope, name), id);`, add:

```rust
self.scope_definitions.entry(scope).or_default().push(id);
```

Replace `names_in_scope` (line 151-163):

```rust
pub fn names_in_scope(&self) -> Vec<Sym> {
    let mut names = Vec::new();
    for &scope_id in &self.scope_stack {
        if let Some(defs) = self.scope_definitions.get(&scope_id) {
            for &def_id in defs {
                names.push(self.definitions[def_id.index()].name);
            }
        }
    }
    names.sort();
    names.dedup();
    names
}
```

### Task 5: Update unused_import.rs

In `crates/lx/src/linter/rules/unused_import.rs` line 56-58:

```rust
let def = model.definitions.iter().enumerate().find(|(_, d)| matches!(d.kind, DefKind::Import) && d.name == *name && d.span == span);

if let Some((def_id, _)) = def {
    let refs = model.references_to(def_id);
```

The `def_id` from `enumerate()` is a raw `usize`. Wrap it:

```rust
if let Some((idx, _)) = def {
    let refs = model.references_to(DefinitionId::new(idx));
```

Add `use crate::checker::semantic::DefinitionId;` to the imports.

### Task 6: Search for remaining raw indexing

Run `rg --type rust 'definitions\[|\.definitions\[' crates/lx/src/` to find any place that indexes `definitions` with a raw usize or DefinitionId without `.index()`. Fix each occurrence.

Run `rg --type rust 'scopes\[' crates/lx/src/checker/` for the same with scopes.

### Task 7: Verify

Run `just fmt` then `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

### Task 8: Commit

Run `just fmt` then `git add -A && git commit -m "refactor: newtype ScopeId/DefinitionId, add reverse-lookup indexes to SemanticModel"`.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **`references_to` return type changes from `Vec<ExprId>` to `&[ExprId]`** — the one caller (`unused_import.rs` line 59) calls `.is_empty()` on the result, which works on slices. No other callers exist.
5. **`unused_import.rs` line 56 uses `enumerate()` which yields `(usize, &T)`** — wrap the index with `DefinitionId::new(idx)` before passing to `references_to`.
6. **`names_in_scope` is on `SemanticModelBuilder`**, not `SemanticModel`** — the builder has `scope_stack`. Its one caller is `type_ops.rs` line 27 via `self.sem.names_in_scope()` where `self.sem` is a `SemanticModelBuilder`.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/SEMANTIC_MODEL_HARDEN.md" })
```

Then call `next_task` to begin.
