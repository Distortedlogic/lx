# Unit 7: lx-linter

## Scope

Extract `crates/lx/src/linter/` into a new `crates/lx-linter` crate. The linter depends on lx-checker (for `Diagnostic`, `DiagLevel`, `SemanticModel`) and lx-ast (for AST types and visitor).

## Prerequisites

- Units 1-2 complete (lx-span, lx-ast exist)
- Unit 6 complete (lx-checker exists, exports `Diagnostic`, `DiagLevel`, `semantic::SemanticModel`, `diagnostics::DiagnosticKind`)

## Steps

### Step 1: Create crate skeleton

Create `crates/lx-linter/Cargo.toml`:

```toml
[package]
edition.workspace = true
license.workspace = true
name = "lx-linter"
version = "0.1.0"

[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }
lx-checker = { path = "../lx-checker" }
miette.workspace = true

[lints]
workspace = true
```

Create `crates/lx-linter/src/lib.rs`:

```rust
mod registry;
mod rule;
pub mod rules;
mod runner;

pub use registry::RuleRegistry;
pub use rule::{LintRule, RuleCategory};
pub use runner::lint;
```

### Step 2: Move linter source files

| Source | Destination |
|--------|-------------|
| `crates/lx/src/linter/registry.rs` | `crates/lx-linter/src/registry.rs` |
| `crates/lx/src/linter/rule.rs` | `crates/lx-linter/src/rule.rs` |
| `crates/lx/src/linter/runner.rs` | `crates/lx-linter/src/runner.rs` |
| `crates/lx/src/linter/rules/` (entire directory) | `crates/lx-linter/src/rules/` |

Rule files to move:

| Source | Destination |
|--------|-------------|
| `crates/lx/src/linter/rules/mod.rs` | `crates/lx-linter/src/rules/mod.rs` |
| `crates/lx/src/linter/rules/break_outside_loop.rs` | `crates/lx-linter/src/rules/break_outside_loop.rs` |
| `crates/lx/src/linter/rules/duplicate_record_field.rs` | `crates/lx-linter/src/rules/duplicate_record_field.rs` |
| `crates/lx/src/linter/rules/empty_match.rs` | `crates/lx-linter/src/rules/empty_match.rs` |
| `crates/lx/src/linter/rules/mut_never_mutated.rs` | `crates/lx-linter/src/rules/mut_never_mutated.rs` |
| `crates/lx/src/linter/rules/redundant_propagate.rs` | `crates/lx-linter/src/rules/redundant_propagate.rs` |
| `crates/lx/src/linter/rules/single_branch_par.rs` | `crates/lx-linter/src/rules/single_branch_par.rs` |
| `crates/lx/src/linter/rules/unreachable_code.rs` | `crates/lx-linter/src/rules/unreachable_code.rs` |
| `crates/lx/src/linter/rules/unused_import.rs` | `crates/lx-linter/src/rules/unused_import.rs` |

### Step 3: Rewrite imports in all moved files

The `rule.rs` file imports:

```rust
use crate::ast::{AstArena, Core, Expr, ExprId, Program, Stmt, StmtId};
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;
```

These become:

```rust
use lx_ast::ast::{AstArena, Core, Expr, ExprId, Program, Stmt, StmtId};
use lx_checker::Diagnostic;
use lx_checker::semantic::SemanticModel;
```

The `runner.rs` file imports:

```rust
use crate::ast::Core;
use crate::checker::Diagnostic;
use crate::checker::semantic::SemanticModel;
use crate::linter::rules::mut_never_mutated::check_unused_mut;
use crate::visitor::prelude::*;
```

These become:

```rust
use lx_ast::ast::Core;
use lx_checker::Diagnostic;
use lx_checker::semantic::SemanticModel;
use crate::rules::mut_never_mutated::check_unused_mut;
use lx_ast::visitor::prelude::*;
```

The `registry.rs` file has only `use super::rule::LintRule;` which stays the same.

Full import rewrite patterns for all files:

| Old import | New import |
|------------|-----------|
| `crate::ast::{...}` | `lx_ast::ast::{...}` |
| `crate::checker::Diagnostic` | `lx_checker::Diagnostic` |
| `crate::checker::DiagLevel` | `lx_checker::DiagLevel` |
| `crate::checker::semantic::SemanticModel` | `lx_checker::semantic::SemanticModel` |
| `crate::checker::diagnostics::DiagnosticKind` | `lx_checker::diagnostics::DiagnosticKind` |
| `crate::visitor::prelude::*` | `lx_ast::visitor::prelude::*` |
| `crate::visitor::{AstVisitor, VisitAction}` | `lx_ast::visitor::{AstVisitor, VisitAction}` |
| `crate::sym::Sym` | `lx_span::sym::Sym` |
| `crate::sym::intern` | `lx_span::sym::intern` |
| `crate::linter::rules::*` | `crate::rules::*` |

### Step 4: Add lx-linter to workspace and lx crate

In `/home/entropybender/repos/lx/Cargo.toml`, add `"crates/lx-linter"` to `workspace.members`.

In `crates/lx/Cargo.toml`, add:

```toml
lx-linter = { path = "../lx-linter" }
```

### Step 5: Re-export shim in lx crate

Delete the entire `crates/lx/src/linter/` directory. Create `crates/lx/src/linter.rs`:

```rust
pub use lx_linter::*;
```

The `pub mod linter;` line in `crates/lx/src/lib.rs` remains unchanged (it now resolves to the `linter.rs` file).

### Step 6: Verify downstream consumers

`crates/lx-cli/src/check.rs` does NOT directly import from `lx::linter` (the lint integration was inside checker, which was removed in unit 6). No changes needed in lx-cli for this unit.

The `lx` crate itself no longer calls `lint()` from `check()` (removed in unit 6). The re-export shim ensures that any code reaching for `lx::linter::*` still compiles.

## Files touched

| Action | File |
|--------|------|
| CREATE | `crates/lx-linter/Cargo.toml` |
| CREATE | `crates/lx-linter/src/lib.rs` |
| MOVE+EDIT | `registry.rs`, `rule.rs`, `runner.rs` |
| MOVE+EDIT | `rules/mod.rs` and 8 rule files |
| DELETE | `crates/lx/src/linter/` directory (all files) |
| CREATE | `crates/lx/src/linter.rs` (re-export shim) |
| EDIT | `crates/lx/Cargo.toml` (add lx-linter dep) |
| EDIT | `/home/entropybender/repos/lx/Cargo.toml` (add workspace member) |

## Verification

1. `just diagnose` passes
2. `lx::linter::RuleRegistry` is accessible
3. `lx::linter::lint` is accessible
4. `lx::linter::LintRule` is accessible
5. lx-linter depends on lx-checker but NOT on lx (no cycle)
6. No file exceeds 300 lines
