# lx Crate Split -- Full Plan Overview

## Goal

Split the monolith `lx` crate into 9 focused crates plus a thin `lx` facade. The `lx` crate re-exports everything from the sub-crates so downstream code (`lx-cli`, `lx-api`, `lx-desktop`, `lx-mobile`) keeps compiling unchanged.

## DAG

```
lx-span
  |
lx-ast
  |
  +--- lx-parser
  +--- lx-desugar
  +--- lx-checker --- lx-linter
  +--- lx-fmt
  +--- lx-value --- lx-eval
  |
  lx (facade, re-exports everything)
  |
  lx-cli
```

## Units

| Unit | Crate | Contents |
|------|-------|----------|
| 1 | lx-span | `sym.rs`, `FileId`/`Comment`/`CommentStore`/`CommentPlacement` from `source.rs`, `ParseError` extracted from `error.rs`, constants `PLUGIN_MANIFEST`/`LX_MANIFEST` |
| 2 | lx-ast | `ast/`, `visitor/`, `Global*Id`/`AttachedComment`/`CommentMap` from `source.rs` |
| 3 | lx-parser | `lexer/`, `parser/` |
| 4 | lx-desugar | `folder/` |
| 5 | lx-fmt | `formatter/` |
| 6 | lx-checker | `checker/` (remove `lint()` call) |
| 7 | lx-linter | `linter/` |
| 8 | lx-value | `value/`, `error.rs` (`LxError`), `env.rs`, `ModuleExports`, `EventStream`, traits (`BuiltinCtx`, `ToolModuleHandle`, `ExternalStreamSink`) |
| 9 | lx-eval | `interpreter/`, `builtins/`, `stdlib/`, `runtime/`, `mcp_client`, `tool_module` |
| 10 | lx facade + lx-cli | Wire everything together, update `lx-cli` to depend on sub-crates |

## Re-export Strategy

After each unit moves code out of `lx`, the `lx` crate's `lib.rs` adds re-exports so all `use lx::X` paths continue to work. Example after unit 1:

```rust
pub use lx_span::sym;
pub use lx_span::source; // re-export the partial source module
```

Modules that remain in `lx` and previously used `use crate::sym::Sym` continue to work because the re-export makes `crate::sym` resolve through `pub use lx_span::sym`.

## Execution Order

Units must be executed in order 1-10. Each unit's prerequisite is the completion of the previous unit. Each unit ends with `just diagnose` passing cleanly.

## Workspace Pattern

Each new crate follows the same pattern:
1. Create `crates/<name>/Cargo.toml` with `edition.workspace = true`, `license.workspace = true`, `[lints] workspace = true`
2. Add `"crates/<name>"` to `members` in workspace `Cargo.toml`
3. Add `<name> = { path = "../<name>" }` to `crates/lx/Cargo.toml` dependencies
4. Move source files
5. Add `pub use <name>::module;` re-exports to `crates/lx/src/lib.rs`
