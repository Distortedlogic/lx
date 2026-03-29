# Unit 6: lx-checker

## Scope

Extract `crates/lx/src/checker/` into a new `crates/lx-checker` crate. Remove the `lint()` integration from `check()` and `check_with_imports()` so that lx-checker has no dependency on lx-linter. Lint-after-check wiring moves to lx-cli in unit 10.

## Prerequisites

- Units 1-2 complete (lx-span, lx-ast exist and are published in workspace)
- `crates/lx-ast` re-exports: `ast`, `visitor`, `source` (GlobalExprId, GlobalStmtId, AttachedComment, CommentMap)
- `crates/lx-span` re-exports: `sym`, `source` (FileId, Comment, CommentStore, CommentPlacement), constants

## Steps

### Step 1: Create crate skeleton

Create `crates/lx-checker/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-checker"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }
ena.workspace = true
itertools.workspace = true
la-arena.workspace = true
miette.workspace = true
num-bigint.workspace = true
similar.workspace = true

[lints]
workspace = true
```

Create `crates/lx-checker/src/lib.rs`:

```rust
mod capture;
mod check_expr;
pub mod diagnostics;
mod exhaust;
mod exhaust_core;
mod exhaust_types;
mod generics;
mod infer_pattern;
pub mod module_graph;
mod narrowing;
pub mod semantic;
mod stdlib_sigs;
pub(crate) mod suggest;
mod synth_compound;
mod synth_control;
pub mod type_arena;
pub mod type_error;
mod type_ops;
pub mod types;
pub mod unification;
mod visit_stmt;

use std::collections::HashMap;
use std::sync::Arc;

use la_arena::ArenaMap;

use lx_ast::ast::{AstArena, Core, ExprId, Program, Stmt, StmtId, TypeExpr, TypeExprId};
use lx_ast::visitor::{AstVisitor, VisitAction};
use lx_span::sym::Sym;
use diagnostics::{DiagnosticKind, Fix};
use miette::SourceSpan;
use module_graph::ModuleSignature;
use narrowing::NarrowingEnv;
use semantic::{SemanticModel, SemanticModelBuilder};
use type_arena::{TypeArena, TypeId};
use type_error::TypeError;
use types::{Type, Variant};
use unification::UnificationTable;
```

### Step 2: Move all checker source files

Move these files from `crates/lx/src/checker/` to `crates/lx-checker/src/`:

| Source | Destination |
|--------|-------------|
| `crates/lx/src/checker/capture.rs` | `crates/lx-checker/src/capture.rs` |
| `crates/lx/src/checker/check_expr.rs` | `crates/lx-checker/src/check_expr.rs` |
| `crates/lx/src/checker/diagnostics.rs` | `crates/lx-checker/src/diagnostics.rs` |
| `crates/lx/src/checker/exhaust.rs` | `crates/lx-checker/src/exhaust.rs` |
| `crates/lx/src/checker/exhaust_core.rs` | `crates/lx-checker/src/exhaust_core.rs` |
| `crates/lx/src/checker/exhaust_types.rs` | `crates/lx-checker/src/exhaust_types.rs` |
| `crates/lx/src/checker/generics.rs` | `crates/lx-checker/src/generics.rs` |
| `crates/lx/src/checker/infer_pattern.rs` | `crates/lx-checker/src/infer_pattern.rs` |
| `crates/lx/src/checker/module_graph.rs` | `crates/lx-checker/src/module_graph.rs` |
| `crates/lx/src/checker/narrowing.rs` | `crates/lx-checker/src/narrowing.rs` |
| `crates/lx/src/checker/semantic.rs` | `crates/lx-checker/src/semantic.rs` |
| `crates/lx/src/checker/stdlib_sigs.rs` | `crates/lx-checker/src/stdlib_sigs.rs` |
| `crates/lx/src/checker/suggest.rs` | `crates/lx-checker/src/suggest.rs` |
| `crates/lx/src/checker/synth_compound.rs` | `crates/lx-checker/src/synth_compound.rs` |
| `crates/lx/src/checker/synth_control.rs` | `crates/lx-checker/src/synth_control.rs` |
| `crates/lx/src/checker/type_arena.rs` | `crates/lx-checker/src/type_arena.rs` |
| `crates/lx/src/checker/type_error.rs` | `crates/lx-checker/src/type_error.rs` |
| `crates/lx/src/checker/type_ops.rs` | `crates/lx-checker/src/type_ops.rs` |
| `crates/lx/src/checker/types.rs` | `crates/lx-checker/src/types.rs` |
| `crates/lx/src/checker/unification.rs` | `crates/lx-checker/src/unification.rs` |
| `crates/lx/src/checker/visit_stmt.rs` | `crates/lx-checker/src/visit_stmt.rs` |

### Step 3: Remove lint() calls from check functions

In `crates/lx-checker/src/lib.rs`, the `check()` function currently has:

```rust
use crate::linter::{RuleRegistry, lint};
```

and lines:

```rust
let mut registry = RuleRegistry::default_rules();
let lint_diags = lint(program, &semantic, &mut registry);
diagnostics.extend(lint_diags);
```

Delete the `use crate::linter::{RuleRegistry, lint};` import line (line 43 of the original `checker/mod.rs`).

In `check()`: remove the three lines after `let mut diagnostics = checker.diagnostics;` that create a registry and call lint. The function becomes:

```rust
pub fn check(program: &Program<Core>, source: Arc<str>) -> CheckResult {
    let mut checker = Checker::new(&program.arena);
    checker.check_program(program);
    let semantic = checker.sem.build(checker.expr_types, checker.type_defs, checker.trait_fields, checker.type_arena);
    let diagnostics = checker.diagnostics;
    CheckResult { diagnostics, source, semantic }
}
```

In `check_with_imports()`: same removal. The function becomes:

```rust
pub fn check_with_imports(program: &Program<Core>, source: Arc<str>, import_signatures: HashMap<Sym, ModuleSignature>) -> CheckResult {
    // ... setup code unchanged ...
    checker.check_program(program);
    let semantic = checker.sem.build(checker.expr_types, checker.type_defs, checker.trait_fields, checker.type_arena);
    let diagnostics = checker.diagnostics;
    CheckResult { diagnostics, source, semantic }
}
```

### Step 4: Rewrite imports in all moved files

Every file that uses `crate::sym::Sym` changes to `lx_span::sym::Sym`.
Every file that uses `crate::sym::intern` changes to `lx_span::sym::intern`.
Every file that uses `crate::ast::*` changes to `lx_ast::ast::*`.
Every file that uses `crate::visitor::*` changes to `lx_ast::visitor::*`.
Every file that uses `crate::checker::*` changes to `crate::*` (since checker IS the crate root now).

Specific patterns:

| Old import | New import |
|------------|-----------|
| `crate::sym::Sym` | `lx_span::sym::Sym` |
| `crate::sym::intern` | `lx_span::sym::intern` |
| `crate::ast::{...}` | `lx_ast::ast::{...}` |
| `crate::sym::{self, Sym}` | `lx_span::sym::{self, Sym}` |
| `crate::visitor::{AstVisitor, VisitAction}` | `lx_ast::visitor::{AstVisitor, VisitAction}` |
| `crate::visitor::prelude::*` | `lx_ast::visitor::prelude::*` |
| `crate::visitor::{walk_binding, walk_func}` | `lx_ast::visitor::{walk_binding, walk_func}` |
| `crate::visitor::walk_program(self, program)` (inline call in mod.rs) | `lx_ast::visitor::walk_program(self, program)` |
| `crate::sym::intern(name.as_str())` (inline calls in mod.rs lines 150, 180) | `lx_span::sym::intern(name.as_str())` |
| `crate::checker::Diagnostic` | `crate::Diagnostic` |
| `crate::checker::DiagLevel` | `crate::DiagLevel` |
| `crate::checker::diagnostics::*` | `crate::diagnostics::*` |
| `crate::checker::semantic::*` | `crate::semantic::*` |
| `crate::checker::types::*` | `crate::types::*` |
| `crate::checker::type_arena::*` | `crate::type_arena::*` |
| `crate::checker::unification::*` | `crate::unification::*` |

### Step 5: Add lx-checker to workspace and lx crate

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-checker"` to `workspace.members`.

In `crates/lx/Cargo.toml`, add:

```toml
lx-checker = { path = "../lx-checker" }
```

### Step 6: Re-export shim in lx crate

Replace the contents of `crates/lx/src/checker/mod.rs` (delete all 22 submodule files from `crates/lx/src/checker/`) with a re-export facade. Create `crates/lx/src/checker.rs`:

```rust
pub use lx_checker::*;
```

Delete `crates/lx/src/checker/` directory entirely and replace the `pub mod checker;` line in `crates/lx/src/lib.rs` with a file-level module (the `checker.rs` file created above handles it).

### Step 7: Verify lx-cli still compiles

`lx-cli/src/check.rs` imports:

```rust
use lx::checker::diagnostics::Applicability;
use lx::checker::{CheckResult, DiagLevel, Diagnostic, check};
```

These continue to work through the re-export shim. No changes needed in lx-cli for this unit.

## Files touched

| Action | File |
|--------|------|
| CREATE | `crates/lx-checker/Cargo.toml` |
| CREATE | `crates/lx-checker/src/lib.rs` |
| MOVE+EDIT | 21 files from `crates/lx/src/checker/*.rs` to `crates/lx-checker/src/*.rs` |
| DELETE | `crates/lx/src/checker/` directory (all 22 files including mod.rs) |
| CREATE | `crates/lx/src/checker.rs` (re-export shim) |
| EDIT | `crates/lx/Cargo.toml` (add lx-checker dep) |
| EDIT | `/home/entropybender/repos/lx/Cargo.toml` (add workspace member) |

## Verification

1. `just diagnose` passes
2. `lx::checker::check` is callable from lx-cli
3. `lx::checker::check_with_imports` is callable from lx-cli
4. `lx::checker::diagnostics::Applicability` is accessible from lx-cli
5. lx-checker has NO dependency on lx-linter (confirm `Cargo.toml`)
6. No file exceeds 300 lines
