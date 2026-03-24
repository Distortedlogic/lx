# Goal

Introduce file-qualified node ID types (`GlobalExprId`, `GlobalStmtId`, `GlobalPatternId`, `GlobalTypeExprId`) that pair a `FileId` with a local arena index, enabling cross-module type checking and diagnostics without ID collisions.

# Prerequisites

None. Introduces new types alongside existing ones — does not replace local IDs in single-file paths.

# Why

- Each `Program<Phase>` owns its own `AstArena`. An `ExprId` from file A and an `ExprId` from file B can have the same raw index. Cross-module analysis that stores or compares node IDs from different files will silently mix them.

# Verified ModuleSignature construction sites (7 total)

| File | Line | Context |
|------|------|---------|
| `checker/module_graph.rs` | 101 | `extract_signature()` — from semantic data |
| `checker/stdlib_sigs.rs` | 23 | `empty_sig()` — 11 stdlib modules without signatures |
| `checker/stdlib_sigs.rs` | 53 | `build_math()` |
| `checker/stdlib_sigs.rs` | 78 | `build_fs()` |
| `checker/stdlib_sigs.rs` | 97 | `build_env()` |
| `checker/stdlib_sigs.rs` | 118 | `build_channel()` |
| `checker/stdlib_sigs.rs` | 136 | `build_time()` |

All 7 construct `ModuleSignature { bindings, types, traits, type_arena }` with 4 fields. Adding `file` means updating all 7.

# Files affected

| File | Change |
|------|--------|
| `crates/lx/src/source.rs` | Add GlobalExprId, GlobalStmtId, GlobalPatternId, GlobalTypeExprId, GlobalNodeId |
| `crates/lx/src/checker/module_graph.rs` | Add `file: Option<FileId>` to ModuleSignature, update line 101 |
| `crates/lx/src/checker/stdlib_sigs.rs` | Add `file: None` to all 6 construction sites (lines 23, 53, 78, 97, 118, 136) |

# Task List

### Task 1: Define global ID types

In `crates/lx/src/source.rs`, add after the `FullSpan` definition (line 55):

```rust
use crate::ast::{ExprId, StmtId, PatternId, TypeExprId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalExprId {
    pub file: FileId,
    pub local: ExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalStmtId {
    pub file: FileId,
    pub local: StmtId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalPatternId {
    pub file: FileId,
    pub local: PatternId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalTypeExprId {
    pub file: FileId,
    pub local: TypeExprId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GlobalNodeId {
    Expr(GlobalExprId),
    Stmt(GlobalStmtId),
    Pattern(GlobalPatternId),
    TypeExpr(GlobalTypeExprId),
}

impl GlobalExprId {
    pub fn new(file: FileId, local: ExprId) -> Self {
        Self { file, local }
    }
}

impl GlobalStmtId {
    pub fn new(file: FileId, local: StmtId) -> Self {
        Self { file, local }
    }
}

impl GlobalPatternId {
    pub fn new(file: FileId, local: PatternId) -> Self {
        Self { file, local }
    }
}

impl GlobalTypeExprId {
    pub fn new(file: FileId, local: TypeExprId) -> Self {
        Self { file, local }
    }
}
```

### Task 2: Add NodeId::in_file conversion

In `crates/lx/src/source.rs`, add after the global ID definitions:

```rust
impl crate::ast::NodeId {
    pub fn in_file(self, file: FileId) -> GlobalNodeId {
        match self {
            crate::ast::NodeId::Expr(id) => GlobalNodeId::Expr(GlobalExprId::new(file, id)),
            crate::ast::NodeId::Stmt(id) => GlobalNodeId::Stmt(GlobalStmtId::new(file, id)),
            crate::ast::NodeId::Pattern(id) => GlobalNodeId::Pattern(GlobalPatternId::new(file, id)),
            crate::ast::NodeId::TypeExpr(id) => GlobalNodeId::TypeExpr(GlobalTypeExprId::new(file, id)),
        }
    }
}
```

This `impl` block is valid because both `NodeId` and `source.rs` are in the same crate (`lx`).

### Task 3: Add file field to ModuleSignature

In `crates/lx/src/checker/module_graph.rs`, add field to ModuleSignature (line 12):

```rust
pub struct ModuleSignature {
    pub file: Option<crate::source::FileId>,
    pub bindings: HashMap<Sym, TypeId>,
    pub types: HashMap<Sym, Vec<Sym>>,
    pub traits: HashMap<Sym, Vec<(Sym, TypeId)>>,
    pub type_arena: TypeArena,
}
```

Update `extract_signature` (line 101) — this function has access to `program.file`:

```rust
ModuleSignature { file: Some(program.file), bindings, types, traits, type_arena }
```

Note: `extract_signature` receives `program: &Program<Core>` which has the `file: FileId` field. Use `Some(program.file)`.

### Task 4: Update all stdlib signature construction sites

In `crates/lx/src/checker/stdlib_sigs.rs`, add `file: None` to all 6 construction sites:

Line 23 (`empty_sig`):
```rust
ModuleSignature { file: None, bindings: HashMap::new(), types: HashMap::new(), traits: HashMap::new(), type_arena: TypeArena::new() }
```

Line 53 (`build_math`):
```rust
ModuleSignature { file: None, bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
```

Line 78 (`build_fs`):
```rust
ModuleSignature { file: None, bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
```

Line 97 (`build_env`):
```rust
ModuleSignature { file: None, bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
```

Line 118 (`build_channel`):
```rust
ModuleSignature { file: None, bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
```

Line 136 (`build_time`):
```rust
ModuleSignature { file: None, bindings: b, types: HashMap::new(), traits: HashMap::new(), type_arena: ta }
```

### Task 5: Verify

Run `just fmt` then `just diagnose`. Fix all errors and warnings. Re-run until clean. Then run `just test`. Fix any failures.

### Task 6: Commit

Run `just fmt` then `git add -A && git commit -m "feat: add GlobalExprId/GlobalNodeId types for cross-file node references"`.

---

## CRITICAL REMINDERS

1. **NEVER run raw cargo commands.** Use `just fmt`, `just test`, `just diagnose`.
2. **Run commands VERBATIM.** Do not append pipes, redirects, or shell operators.
3. **Do not add, skip, reorder, or combine tasks.**
4. **Do not replace local IDs in existing single-file code paths.** Global IDs are new types used only at cross-file boundaries.
5. **`source.rs` imports from `ast` module** — `use crate::ast::{ExprId, StmtId, PatternId, TypeExprId};`. This is a one-directional dependency (source → ast). The `impl crate::ast::NodeId` block in `source.rs` is valid within the same crate.
6. **`ExprId` is `la_arena::Idx<Spanned<Expr>>`** — it derives `Copy`, `PartialEq`, `Eq`, `Hash` via la_arena. The global wrapper composes these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/CROSS_FILE_NODE_IDS.md" })
```

Then call `next_task` to begin.
