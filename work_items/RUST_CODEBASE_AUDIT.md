# Rust Codebase Quality Audit

## Goal

Fix violations identified by running the `rules/rust-audit.md` high-frequency and low-frequency checks against the current codebase. The audit found actionable violations in: file size limits, import hygiene, Cargo dependency hoisting, swallowed errors in the store subsystem and checker, string-typed enum fields in the diag module, a backward-compat serde attribute, and a swallowed WebSocket send error. Checks that found no violations are listed in the "Checks with no violations" section below.

## Why

- Four `.rs` files exceed the 300-line hard limit from CLAUDE.md, making them harder to navigate and violating the project's own rules
- ~90+ inline qualified paths at call sites across `lx-cli` add visual noise and violate the "no inline import paths" rule
- The `lx` crate lacks a prelude despite 7+ types being commonly co-imported by `lx-cli`, causing redundant import boilerplate
- 15 dependencies in `crates/lx/Cargo.toml` and 2 in `crates/lx-cli/Cargo.toml` bypass workspace version management, creating version drift risk
- The store subsystem silently discards file-write, serialization, and deserialization errors — persist can corrupt data, load can silently return empty state
- `DiagNode.kind`, `DiagEdge.style`, and `DiagEdge.edge_type` are strings with known fixed variants, defeating exhaustiveness checking and enabling typo bugs
- A `#[serde(default)]` backward-compat attribute exists despite the no-backward-compat rule

## What changes

### File splits (300-line limit)
- `parser/expr.rs` (477 → split): extract atom sub-parsers (string, list, record, map, paren, with, sections) into `parser/expr_atoms.rs`, keeping core framework and pratt chain in `expr.rs`
- `value/mod.rs` (390 → split): extract `Serialize`, `Deserialize`, and `From<serde_json::Value>` / `From<&LxVal> for serde_json::Value` impls into `value/serde_impl.rs`
- `stdlib/diag/mod.rs` (350 → split): extract echart generation into `diag/echart.rs`, mermaid generation into `diag/mermaid.rs`
- `parser/stmt.rs` (303 → trim): exceeds limit by 3 lines, trim by inlining or removing redundant code

### Import hygiene
- Add `crates/lx/src/prelude.rs` re-exporting: `lex`, `parse`, `Interpreter`, `LxVal`, `RuntimeCtx`, `LxError`, `Program`, `DiagLevel`, `check` (the checker function)
- Declare `pub mod prelude` in `crates/lx/src/lib.rs`
- In every `lx-cli/src/*.rs` file: replace inline qualified paths with `use` imports (prefer `use lx::prelude::*` where applicable, `use std::{env, fs, io}` for std paths)
- In `store_dispatch.rs`: use the existing `LxVal` import instead of `crate::value::LxVal` at call sites

### Cargo dependency hygiene
- Hoist 15 deps from `crates/lx/Cargo.toml` into `[workspace.dependencies]` in root `Cargo.toml`: `async-recursion`, `chumsky`, `futures`, `indexmap`, `lasso`, `miette`, `num-bigint`, `num-integer`, `num-traits`, `serde`, `similar`, `thiserror`, `tokio`, `tokio-tungstenite` (note: `logos` is already in workspace but not used via `workspace = true`)
- Hoist 2 deps from `crates/lx-cli/Cargo.toml`: `clap`, `miette` (miette already being hoisted from lx crate, just add `clap`)
- Convert all string shorthand deps to object notation in both root and crate-level Cargo.toml
- Replace crate-level version specs with `dep.workspace = true`

### Store error handling
- Change `persist()` signature to `fn persist(state: &StoreState, span: SourceSpan) -> Result<(), LxError>` — propagate serialization and write errors
- Change `load_from_disk()` to return `Result<IndexMap<String, LxVal>, String>` — propagate read and parse errors
- Change `store_len()` and `store_clone()` to return `Result` — propagate missing-store errors
- Fix `bi_save_to()` `.unwrap_or_default()` → `.map_err(...)?`
- Update all callers of these functions to handle errors

### Diag enum types
- Define `NodeKind` enum with variants: `Agent`, `Tool`, `Decision`, `Fork`, `Join`, `Loop`, `Resource`, `User`, `Io`, `Type`
- Define `EdgeStyle` enum with variants: `Solid`, `Dashed`, `Double`
- Define `EdgeType` enum with variants: `Agent`, `Stream`, `Data`, `Io`, `Exec`
- All three enums go in `diag_types.rs`, derive `Debug, Clone, Copy, PartialEq, Eq`
- Replace `kind: String` → `kind: NodeKind`, `style: String` → `style: EdgeStyle`, `edge_type: String` → `edge_type: EdgeType` in struct definitions
- Update all match arms in `mod.rs` (now split into `echart.rs` and `mermaid.rs`) and `diag_walk.rs`/`diag_walk_expr.rs` to use enum variants
- Add `impl NodeKind { fn as_str(&self) -> &'static str }` and similar for the rendering functions that need string output

### Value serde error handling
- Fix `From<&LxVal> for serde_json::Value` (`value/mod.rs:373`): currently uses `.unwrap_or(serde_json::Value::Null)` which silently returns Null on serialization failure. Since `From` cannot return errors, replace with a `to_json_value(&self) -> Result<serde_json::Value, String>` method on `LxVal` and update callers (`store/mod.rs:50`, `store_dispatch.rs:132`) to use the fallible method. Keep the `From` impl as a convenience that calls the method and panics (or keep `.unwrap_or(Null)` but add a code comment explaining the limitation is due to the `From` trait contract)

### Miscellaneous
- Remove `#[serde(default)]` from `lockfile.rs:7`
- Fix `ws.rs:141`: on pong send failure, break the receive loop or return an error

### Checker swallowed unification errors
- `checker/synth_helpers.rs:153,157`: `let _ = self.table.unify(...)` discards `Result<(), String>` from type unification — genuine swallowed error that masks type-check failures. Fix: propagate or log the unification error.

### Out of scope (separate work items)
- Error swallowing in `manifest.rs`: multiple functions silently return empty HashMaps on file/parse errors — CLI discovery logic where graceful degradation may be intentional
- `agent_cmd.rs:56`: `serde_json::to_string(&j).unwrap_or_default()` — debug output serialization, empty string on failure is acceptable

### Checks with no violations found
The following `rules/rust-audit.md` checks were investigated and yielded no actionable violations:
- **Self-assignments**: no `let x = x` patterns found
- **#[allow(...)] macros**: none in codebase
- **&String / &Vec parameters**: no function parameters use these (only local variable type annotations)
- **&Arc<T> parameters**: `&Arc<RuntimeCtx>` is required by the `SyncBuiltinFn` function pointer type signature — some builtins clone the Arc, so all must accept `&Arc<RuntimeCtx>`
- **Trait with single implementation**: all traits have multiple impls
- **Newtype with no added behavior**: `ValueKey` has proper Hash/Eq impls
- **Unnecessary boxing**: all `Box<>` uses are justified (recursive types, dyn Future)
- **Backwards compatibility code**: no deprecated/legacy/migration code found (except the serde attribute addressed above)
- **String-based enum matching**: no `.to_string() == "..."` or `.as_str() == "..."` patterns on enum values
- **Duplicate types / field spreading**: no struct pairs share 3+ identical fields
- **Re-exports from non-defining crates**: all re-exports are legitimate aggregation patterns
- **Re-export-only modules**: all modules with re-exports also contain logic
- **Extraneous .context()**: no `.context()` calls found in codebase
- **`let _ =` patterns (non-error)**: `let _ = &args[0]` in env/time/cron and `let _ = name` in interpreter are discarding non-error values (reference assertions and unused bindings), not swallowing errors. `let _ = self.synth(...)` in checker discards a `Type` return value for side-effect-only calls, not a `Result`

## How it works

**File splits** are mechanical — move functions and their dependencies into new files, add `mod` declarations, re-export with `pub use` where needed. The parser split extracts atom sub-parsers (lines 119-370 of `expr.rs`: `string_parser`, `list_parser`, `block_or_record_parser`, `looks_like_record`, `record_fields`, `map_parser`, `paren_parser`, `param_parser`, `section_op`, `with_parser`) into `expr_atoms.rs` (~252 lines), keeping the core framework (`ident`, `type_name`, `skip_semis`, `semi_sep`, `expr_parser`, `stmts_block`) and pratt chain (`dot_rhs`, `pratt_expr`, `QRhs`) in `expr.rs` (~224 lines). The value split moves all serde trait impls while keeping the type definition in `mod.rs`.

**Store error propagation** changes `persist()` from fire-and-forget to fallible. Every store mutation that calls `persist()` (set, update, remove, clear, merge) already returns `Result<LxVal, LxError>`, so they just need `persist(&s, span)?` with a `SourceSpan` threaded through. The `get_store`/`get_store_mut` helpers already centralize the "store: not found" error — no const extraction needed. `load_from_disk` callers (`bi_create`, `bi_load`) also already return Result. `store_len` and `store_clone` are called from `builtins/register.rs` (`bi_len`/`bi_empty`) and `interpreter/apply.rs` respectively — these also return Result, so propagation is straightforward.

**Diag enums** replace string matching with exhaustive enum matching. The `Walker` in `diag_walk.rs` and `diag_walk_expr.rs` constructs `DiagNode`/`DiagEdge` values — these call sites change from `kind: "agent".into()` to `kind: NodeKind::Agent`. The `classify_call` function in `diag_walk_expr.rs` currently returns `&'static str` — it must change to return `NodeKind` directly. The rendering functions in the new `mermaid.rs` and `echart.rs` match on enum variants directly — all wildcard `_ =>` arms that previously caught unknown strings must be removed since the enum makes matching exhaustive.

## Files affected

| File | Change |
|------|--------|
| `Cargo.toml` (root) | Add 15 workspace deps (incl. lasso, clap), convert 8 string shorthand to object notation |
| `crates/lx/Cargo.toml` | Replace 15 direct versions with `workspace = true` |
| `crates/lx/src/lib.rs` | Add `pub mod prelude` |
| `crates/lx/src/prelude.rs` | **New** — re-exports of common types |
| `crates/lx/src/parser/expr.rs` | Split: keep core framework + pratt chain, extract atoms |
| `crates/lx/src/parser/expr_atoms.rs` | **New** — atom sub-parsers (string, list, record, map, paren, with, sections) |
| `crates/lx/src/parser/stmt.rs` | Split: move class_decl_parser out |
| `crates/lx/src/parser/stmt_class.rs` | **New** — class declaration parser |
| `crates/lx/src/parser/mod.rs` | Add new submodule declarations |
| `crates/lx/src/value/mod.rs` | Move serde impls out, fix `From<&LxVal>` swallowed error |
| `crates/lx/src/value/serde_impl.rs` | **New** — Serialize/Deserialize/From impls |
| `crates/lx/src/stdlib/diag/mod.rs` | Keep builtins + extract/convert, move rendering out |
| `crates/lx/src/stdlib/diag/echart.rs` | **New** — echart JSON generation |
| `crates/lx/src/stdlib/diag/mermaid.rs` | **New** — mermaid generation |
| `crates/lx/src/stdlib/diag/diag_types.rs` | Add `NodeKind`, `EdgeStyle`, `EdgeType` enums |
| `crates/lx/src/stdlib/diag/diag_walk.rs` | Use enum types instead of strings |
| `crates/lx/src/stdlib/diag/diag_walk_expr.rs` | Use enum types instead of strings |
| `crates/lx/src/stdlib/diag/diag_helpers.rs` | Use enum types if applicable |
| `crates/lx/src/stdlib/store/mod.rs` | Error propagation in persist/load, extract const |
| `crates/lx/src/stdlib/store/store_dispatch.rs` | Error propagation, fix inline imports, fix bi_save_to |
| `crates/lx/src/builtins/register.rs` | Update store_len callers to handle Result |
| `crates/lx/src/interpreter/apply.rs` | Update store_clone caller to handle Result |
| `crates/lx/src/checker/synth_helpers.rs` | Fix swallowed unification errors |
| `crates/lx/src/stdlib/ws.rs` | Handle pong send failure |
| `crates/lx-cli/src/lockfile.rs` | Remove `#[serde(default)]` |
| `crates/lx-cli/src/main.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/check.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/manifest.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/run.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/agent_cmd.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/init.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/install_ops.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/install.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/testing.rs` | Replace inline imports with use statements |
| `crates/lx-cli/src/listing.rs` | Replace inline imports with use statements |
| `crates/lx-cli/Cargo.toml` | Replace `clap` and `miette` direct versions with `workspace = true` |

## Task List

### Task 1: Hoist Cargo dependencies to workspace and fix notation

**Files:** `Cargo.toml` (root), `crates/lx/Cargo.toml`, `crates/lx-cli/Cargo.toml`

In root `Cargo.toml`, add these entries under `[workspace.dependencies]` in object notation. Preserve any features specified in the crate-level Cargo.toml:

- `async-recursion = { version = "1.1.1" }`
- `clap = { version = "4", features = ["derive"] }`
- `chumsky = { version = "0.12", features = ["pratt"] }`
- `futures = { version = "0.3.32" }`
- `indexmap = { version = "2.13.0" }`
- `lasso = { version = "0.7.3", features = ["multi-threaded"] }`
- `miette = { version = "7.6.0", features = ["fancy"] }`
- `num-bigint = { version = "0.4.6" }`
- `num-integer = { version = "0.1.46" }`
- `num-traits = { version = "0.2.19" }`
- `similar = { version = "2.7.0" }`
- `thiserror = { version = "2.0.18" }`
- `tokio = { version = "1.50.0", features = ["macros", "rt-multi-thread", "sync", "time"] }`
- `tokio-tungstenite = { version = "0.29.0", features = ["native-tls"] }`

Also update existing workspace entries that use string shorthand to object notation:
- `chrono = "0.4.44"` → `chrono = { version = "0.4.44" }`
- `dashmap = "6.1.0"` → `dashmap = { version = "6.1.0" }`
- `fastrand = "2"` → `fastrand = { version = "2" }`
- `itertools = "0.14"` → `itertools = { version = "0.14" }`
- `logos = "0.16.1"` → `logos = { version = "0.16.1" }`
- `parking_lot = "0.12.5"` → `parking_lot = { version = "0.12.5" }`
- `pulldown-cmark = "0.13.2"` → `pulldown-cmark = { version = "0.13.2" }`
- `toml = "0.9.8"` → `toml = { version = "0.9.8" }`

Update the `serde` entry: root already has `serde = { version = "1", features = ["derive"] }`. The crate has `serde = { version = "1.0.228", features = ["derive"] }`. Use the more specific version: update root to `serde = { version = "1.0.228", features = ["derive"] }`.

In `crates/lx/Cargo.toml`, replace all 15 direct-version entries with workspace references:
- `async-recursion.workspace = true`
- `chumsky.workspace = true`
- `futures.workspace = true`
- `indexmap.workspace = true`
- `lasso.workspace = true`
- `logos.workspace = true` (already in workspace, just switch to `workspace = true`)
- `miette.workspace = true`
- `num-bigint.workspace = true`
- `num-integer.workspace = true`
- `num-traits.workspace = true`
- `serde.workspace = true`
- `similar.workspace = true`
- `thiserror.workspace = true`
- `tokio.workspace = true`
- `tokio-tungstenite.workspace = true`

In `crates/lx-cli/Cargo.toml`, replace direct versions with workspace references:
- `clap.workspace = true`
- `miette.workspace = true`

### Task 2: Format after Cargo changes

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 3: Commit Cargo changes

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: hoist all Cargo deps to workspace and fix string shorthand notation"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 4: Define diag enum types in diag_types.rs

**File:** `crates/lx/src/stdlib/diag/diag_types.rs`

Add three enums before the existing struct definitions:

```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Agent,
    Tool,
    Decision,
    Fork,
    Join,
    Loop,
    Resource,
    User,
    Io,
    Type,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStyle {
    Solid,
    Dashed,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Agent,
    Stream,
    Data,
    Io,
    Exec,
}
```

Add `impl` blocks for each with an `as_str(&self) -> &'static str` method that returns the lowercase string representation (e.g., `NodeKind::Agent => "agent"`, `EdgeStyle::Dashed => "dashed"`). Also add `as_str` for `NodeKind::Io => "io"` and `NodeKind::Type => "type"`.

Change the struct fields:
- `DiagNode.kind`: `String` → `NodeKind`
- `DiagEdge.style`: `String` → `EdgeStyle`
- `DiagEdge.edge_type`: `String` → `EdgeType`

### Task 5: Update diag_walk.rs and diag_walk_expr.rs to use enum types

**Files:** `crates/lx/src/stdlib/diag/diag_walk.rs`, `crates/lx/src/stdlib/diag/diag_walk_expr.rs`, `crates/lx/src/stdlib/diag/diag_helpers.rs`

Replace all string literal assignments for `kind`, `style`, and `edge_type` fields with enum variants. For example:
- `kind: "agent".into()` → `kind: NodeKind::Agent`
- `style: "solid".into()` → `style: EdgeStyle::Solid`
- `edge_type: "exec".into()` → `edge_type: EdgeType::Exec`

In `diag_walk_expr.rs`, change `classify_call` to return `NodeKind` instead of `&'static str`. Update `handle_call` and all callers of `classify_call` to pass `NodeKind` values directly instead of string literals. The `kind` parameter in `handle_call` changes from `&str` to `NodeKind`.

Import `NodeKind`, `EdgeStyle`, `EdgeType` from `diag_types` in each file that needs them. Remove any `.into()` calls that were converting string literals to `String` for these fields.

### Task 6: Split diag/mod.rs — extract echart.rs and mermaid.rs

**Files:** `crates/lx/src/stdlib/diag/mod.rs` (modify), `crates/lx/src/stdlib/diag/echart.rs` (new), `crates/lx/src/stdlib/diag/mermaid.rs` (new)

From `mod.rs`, move `graph_to_echart_json` (and its helper closures `kind_to_category`, `kind_to_symbol`) into a new file `diag/echart.rs`. Make it `pub(crate) fn graph_to_echart_json(graph: &Graph) -> String`. Add necessary imports (`serde_json`, `std::collections::HashMap`, `std::collections::VecDeque`, and the `diag_types` types).

From `mod.rs`, move `to_mermaid`, `node_shape`, and `emit_subgraph` into a new file `diag/mermaid.rs`. Make `to_mermaid` `pub(crate)`. Add necessary imports.

In `mod.rs`, add `mod echart;` and `mod mermaid;`, import the moved functions with `use echart::graph_to_echart_json;` and `use mermaid::to_mermaid;`.

Update all match arms in the moved functions to use `NodeKind`, `EdgeStyle`, `EdgeType` enum variants instead of `.as_str()` string comparisons. The `kind_to_category` and `kind_to_symbol` closures become `match` on `NodeKind` variants. The edge style/type matches in both mermaid and echart become matches on `EdgeStyle`/`EdgeType` variants. Remove all wildcard `_ =>` match arms that previously caught unknown strings — enum matching is exhaustive, so wildcards must be removed to get compiler enforcement of new variants.

After this split, `mod.rs` should contain only the builtin function wrappers (`bi_extract`, `bi_extract_file`, `bi_to_mermaid`, `bi_to_graph_chart`), the public extraction functions (`extract_mermaid`, `extract_echart_json`), the graph extract/convert functions (`extract_graph`, `graph_to_value`, `value_to_graph`, `node_to_value`, `edge_to_value`, `value_to_node`, `value_to_edge`), and the `build()` function.

Update `value_to_node` and `value_to_edge` in `mod.rs` to parse string fields from LxVal records into enum variants. Add a helper like `fn parse_node_kind(s: &str) -> Result<NodeKind, LxError>` that matches known strings to enum variants and returns an error for unknown values. For `value_to_edge`, the `edge_type` field currently uses `.unwrap_or_else(|| "exec".into())` — change this to default to `EdgeType::Exec` and parse other values via the helper. Same approach for `style` (default `EdgeStyle::Solid`).

### Task 7: Format after diag changes

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 8: Commit diag changes

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: replace diag string types with enums and split mod.rs into echart/mermaid"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 9: Split value/mod.rs — extract serde_impl.rs and fix From swallowed error

**Files:** `crates/lx/src/value/mod.rs` (modify), `crates/lx/src/value/serde_impl.rs` (new)

Move the `impl Serialize for LxVal` block, the `impl<'de> Deserialize<'de> for LxVal` block (the custom deserializer visitor), the `impl From<serde_json::Value> for LxVal`, and the `impl From<&LxVal> for serde_json::Value` block into `crates/lx/src/value/serde_impl.rs`.

In the new file, add all necessary imports: `serde::{Serialize, Serializer, Deserialize, Deserializer}`, `serde::ser::SerializeMap`, `serde::de::{self, Visitor, MapAccess, SeqAccess}`, `super::LxVal`, `super::ValueKey`, `indexmap::IndexMap`, `num_bigint::BigInt`, `std::sync::Arc`, and any other types referenced by the moved impls.

Fix the `From<&LxVal> for serde_json::Value` impl: the current code uses `serde_json::to_value(v).unwrap_or(serde_json::Value::Null)` which silently discards serialization errors. Add a `pub fn to_json_value(&self) -> Result<serde_json::Value, String>` method on `LxVal` that calls `serde_json::to_value(self).map_err(|e| e.to_string())`. Update the store callers (`store/mod.rs:50` and `store_dispatch.rs:132`) to use `record.to_json_value().map_err(...)?` instead of `serde_json::Value::from(&record)`. Keep the `From` impl for convenience in non-error-path contexts.

In `value/mod.rs`, add `mod serde_impl;`. Remove the moved impl blocks and any `use` imports that are now only used by the moved code.

After the split, verify `value/mod.rs` is under 300 lines.

### Task 10: Trim parser/stmt.rs under 300 lines

**Files:** `crates/lx/src/parser/stmt.rs` (modify), `crates/lx/src/parser/stmt_class.rs` (new), `crates/lx/src/parser/mod.rs` (modify)

`parser/stmt.rs` is 303 lines, exceeding the 300-line limit by 3. Move the `class_decl_parser` function and its private `ClassMember` enum (lines 249-303) into a new file `parser/stmt_class.rs`. Make the function `pub(super)`. Add necessary imports (`chumsky::*`, AST types, `TokenKind`, `Span`, `ss`). In `parser/mod.rs`, add `mod stmt_class;`. In `stmt.rs`, import and call the moved function. After the move, `stmt.rs` should be ~252 lines.

### Task 11: Split parser/expr.rs — extract atom sub-parsers

**Files:** `crates/lx/src/parser/expr.rs` (modify), `crates/lx/src/parser/expr_atoms.rs` (new), `crates/lx/src/parser/mod.rs` (modify)

The file is 477 lines. The natural split boundary is between the atom sub-parsers (lines 119-370) and the core framework + pratt chain.

Move these functions into `parser/expr_atoms.rs`: `string_parser`, `list_parser`, `block_or_record_parser`, `looks_like_record`, `record_fields`, `map_parser`, `paren_parser`, `param_parser`, `section_op`, `with_parser`. These are self-contained atom parsers that each take the recursive `expr` parser as a parameter. This is ~252 lines.

Keep in `expr.rs`: `ident`, `type_name`, `skip_semis`, `semi_sep`, `expr_parser` (the recursive entry point), `stmts_block`, `dot_rhs`, `pratt_expr`, `QRhs`. This is ~224 lines.

In `expr_atoms.rs`, add all needed imports: `chumsky::input::ValueInput`, `chumsky::prelude::*`, `super::{Span, ss, token_to_binop}`, the AST types used by the parsers, and `crate::lexer::token::TokenKind`. Each function should be `pub(super)` so `expr.rs` can call them.

In `expr.rs`, add `use expr_atoms::*;` (or import each function individually) and keep calling them in `expr_parser` exactly as before.

In `parser/mod.rs`, add `mod expr_atoms;`.

After the split, verify `expr.rs` is under 300 lines.

### Task 12: Format after file splits

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 13: Commit file splits

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: split oversized files (value/serde, parser/expr, parser/stmt, diag) under 300-line limit"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 14: Fix store/mod.rs error handling — persist and load

**File:** `crates/lx/src/stdlib/store/mod.rs`

Change `persist()` (line 55) signature to `pub(super) fn persist(state: &StoreState, span: SourceSpan) -> Result<(), LxError>`. Replace `.unwrap_or_default()` on serialization (line 59) with `.map_err(|e| LxError::runtime(format!("store.persist: serialize failed: {e}"), span))?`. Replace `let _ = std::fs::write(...)` (line 60) with `std::fs::write(path, pretty).map_err(|e| LxError::runtime(format!("store.persist: write failed: {e}"), span))?`. Return `Ok(())` at end.

Change `load_from_disk()` (line 63) to return `Result<IndexMap<String, LxVal>, String>`. Replace `let Ok(content) = ... else { return IndexMap::new() }` with `let content = std::fs::read_to_string(path).map_err(|e| format!("read: {e}"))?`. Same for JSON parse. On non-Record, return `Err(...)`. In `bi_create` (line 85), if path doesn't exist use empty IndexMap; if exists call `load_from_disk` and propagate error via `.map_err(|e| LxError::runtime(format!("store.create: {e}"), span))?`.

Update all `persist(&s)` call sites in this file to `persist(&s, span)?`.

### Task 15: Fix store_dispatch.rs error handling and inline imports

**Files:** `crates/lx/src/stdlib/store/store_dispatch.rs`, `crates/lx/src/builtins/register.rs`, `crates/lx/src/interpreter/apply.rs`

Change `store_len()` (line 91) to `pub fn store_len(id: u64) -> Result<usize, String>`. Replace `STORES.get(&id).map(|s| s.data.len()).unwrap_or(0)` with `Ok(STORES.get(&id).ok_or("store not found")?.data.len())`. Update callers in `builtins/register.rs`: in `bi_len` (line 42), change `crate::stdlib::store_len(*id)` to `crate::stdlib::store_len(*id).map_err(|e| LxError::runtime(e, span))?`. Same for `bi_empty` (line 57), change `crate::stdlib::store_len(*id) == 0` to `(crate::stdlib::store_len(*id).map_err(|e| LxError::runtime(e, span))?) == 0`.

Change `store_clone()` (line 95) to `pub fn store_clone(id: u64) -> Result<u64, String>`. Replace `.unwrap_or_default()` with `.ok_or("store not found")?.data.clone()`. Return `Ok(new_id)`. Update caller in `interpreter/apply.rs` (line 98) to handle the Result.

Fix `bi_save_to` line 133: replace `.unwrap_or_default()` with `.map_err(|e| LxError::runtime(format!("store.save: serialize: {e}"), span))?`.

Replace `crate::value::LxVal` at call sites in `update_nested_record` and `object_update_nested` with the already-imported `LxVal`.

### Task 16: Format after store changes

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 17: Commit store changes

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: propagate errors in store persist/load instead of swallowing them"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 18: Fix checker swallowed unification errors

**File:** `crates/lx/src/checker/synth_helpers.rs`

At lines 153 and 157, `let _ = self.table.unify(...)` discards `Result<(), String>` from type unification. These are genuine swallowed errors — a unification failure means types don't match but the checker silently continues.

Read the surrounding context to understand why unification was being silently discarded. If the unification is best-effort (non-critical for checker correctness), log the error via the checker's diagnostic mechanism. If unification failure should propagate, change the function to return the error.

The safest fix: replace `let _ = self.table.unify(&a, &b);` with `if let Err(e) = self.table.unify(&a, &b) { self.diag(e, span); }` using whatever diagnostic method the checker provides. If no such method exists, simply remove the `let _ =` and use `let _err = self.table.unify(...)` to make the intent explicit while keeping the warning visible.

### Task 19: Format after checker fix

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 20: Commit checker fix

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: propagate checker unification errors instead of swallowing"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 21: Create lx prelude and fix all inline imports in lx-cli

**Files:** `crates/lx/src/prelude.rs` (new), `crates/lx/src/lib.rs`, all `crates/lx-cli/src/*.rs` files

Create `crates/lx/src/prelude.rs` with:
```
pub use crate::ast::Program;
pub use crate::checker::{check, DiagLevel};
pub use crate::error::LxError;
pub use crate::interpreter::Interpreter;
pub use crate::lexer::lex;
pub use crate::parser::parse;
pub use crate::runtime::RuntimeCtx;
pub use crate::value::LxVal;
```

In `crates/lx/src/lib.rs`, add `pub mod prelude;`.

In each `lx-cli/src/*.rs` file:
- Add `use lx::prelude::*;` where multiple lx types are used
- Add `use std::{env, fs, io};` (or the specific subset needed) to replace `std::env::current_dir()`, `std::fs::read_to_string()`, `std::io::stdin()`, etc.
- Replace all inline qualified paths at call sites with the short imported names
- For `std::path::Path`, add `use std::path::Path;`

Specific files and their inline paths to fix:
- `main.rs`: `std::path::Path::new`, `std::env::current_dir`, `std::fs::read_to_string`, `std::io::stdin`, `lx::error::LxError::Sourced`, `lx::stdlib::diag::extract_mermaid`, `std::fs::write`
- `check.rs`: `lx::checker::check`, `lx::checker::DiagLevel::*`, `lx::error::LxError::type_err`, `std::env::current_dir`, `std::fs::read_dir`
- `manifest.rs`: `std::fs::read_to_string`, `std::env::current_dir`, `std::fs::read_dir`
- `run.rs`: `lx::lexer::lex`, `lx::parser::parse`, `lx::value::LxVal::Unit`, `std::fs::read_to_string`
- `agent_cmd.rs`: `std::fs::read_to_string`, `std::io::stdin`, `lx::lexer::lex`, `lx::parser::parse`, `lx::runtime::RuntimeCtx`, `lx::interpreter::Interpreter`, `lx::value::LxVal::from`, `std::io::stdout`
- `init.rs`: `std::fs::*`, `std::env::*`
- `install_ops.rs`: `std::fs::canonicalize`, `std::os::unix::fs::symlink`, `std::fs::create_dir_all`, `std::fs::read_dir`, `std::fs::copy`
- `install.rs`: `std::env::current_dir`, `std::fs::create_dir_all`, `std::fs::write`, `std::fs::remove_dir_all`
- `testing.rs`: `lx::error::LxError`, inline `std::` paths
- `listing.rs`: any inline std paths

### Task 22: Fix remaining miscellaneous violations

**Files:** `crates/lx-cli/src/lockfile.rs`, `crates/lx/src/stdlib/ws.rs`

In `lockfile.rs`: remove `#[serde(default)]` from line 7 (the attribute on the `package` field of `LockFile`).

In `ws.rs` around line 141: replace `let _ = sink.lock().await.send(Message::Pong(payload)).await;` with:
```
if sink.lock().await.send(Message::Pong(payload)).await.is_err() {
    WS_CONNS.remove(&id);
    return Ok(LxVal::Err(Box::new(LxVal::str("pong send failed"))));
}
```

### Task 23: Format after import and misc fixes

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 24: Commit import and misc fixes

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "refactor: add lx prelude, fix inline imports, remove serde backward-compat, fix ws error handling"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

### Task 25: Run tests

Run the following command verbatim, exactly as written, with no modifications: `just test`. Do NOT pipe, redirect, or append shell operators. If any tests fail, fix all failures and re-run until all tests pass.

### Task 26: Run diagnostics

Run the following command verbatim, exactly as written, with no modifications: `just diagnose`. Do NOT pipe, redirect, or append shell operators. Fix ALL reported errors AND warnings. Re-run `just diagnose` until the output is completely clean with zero errors and zero warnings.

### Task 27: Final format

Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, or append shell operators.

### Task 28: Final commit

Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "fix: final verification cleanup"`. Do NOT pipe, redirect, or append shell operators. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

### Task 29: Remove work item file

Run the following command verbatim, exactly as written, with no modifications: `rm work_items/RUST_CODEBASE_AUDIT.md && git add -A && git commit -m "chore: remove completed work item"`. Do NOT pipe, redirect, or append shell operators. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **NEVER run raw cargo commands.** No `cargo check`, `cargo test`, `cargo clippy`, `cargo fmt`. ALWAYS use `just fmt`, `just test`, `just diagnose`, `just fix`. The `just` recipes include additional steps that raw `cargo` skips.
2. **Between implementation tasks, ONLY run `just fmt` + `git commit`.** Do NOT run `just test`, `just diagnose`, or any compilation/verification command between implementation tasks. These are expensive and only belong in the FINAL VERIFICATION section at the end.
3. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
4. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
5. **`just fmt` is not `cargo fmt`.** `just fmt` runs `dx fmt` + `cargo fmt` + `eclint`. Running `cargo fmt` alone is wrong.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.

## Task Loading Instructions

Read each `### Task N:` entry from the `## Task List` section above. For each task, call `TaskCreate` with:
- `subject`: The task heading text (after `### Task N:`) — copied VERBATIM, not paraphrased
- `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded. Every sentence, every command, every instruction must be transferred exactly as written. Do NOT omit lines, rephrase instructions, drop the "verbatim" language from command instructions, or inject your own wording.
- `activeForm`: A present-continuous form of the subject (e.g., "Hoisting Cargo dependencies to workspace")

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execution rules:
- Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done
- Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa
- Do not run any command not specified in the current task
- Do not "pre-check" compilation between implementation tasks — the task list already has verification in the correct places
- If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata to git commit commands.
- Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section
- When a task description says "Run the following command verbatim, exactly as written, with no modifications" — that phrase and the command after it must appear identically in the loaded task. Do not drop the "verbatim" instruction or rephrase the command.
- Do NOT append shell operators to commands — no pipes (`|`), no redirects (`>`/`2>&1`), no subshells. The command in the task description is the complete command string.
