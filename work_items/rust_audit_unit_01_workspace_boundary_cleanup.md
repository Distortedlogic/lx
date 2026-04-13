# Unit 01: Workspace Boundary Cleanup

## Goal

Remove the verified Rust-audit boundary violations that are already localized and dependency-safe: eliminate the `crates/lx` re-export shim, move its consumers to direct imports from defining crates, and normalize the one crate-local dependency version that is still pinned outside `[workspace.dependencies]`.

## Preconditions

- No earlier unit is required.
- This unit should land before Units 02-04 so later audit sweeps do not have to carry the extra `lx` shim or mixed dependency style.

## Verified Findings

- `crates/lx/src/lib.rs` contains only a `prelude` module that re-exports items defined in other crates:
  - `lx_parser::lexer::lex`
  - `lx_parser::parser::{ParseResult, parse}`
  - `lx_span::{error::ParseError, source::FileId}`
  - `lx_desugar::desugar`
  - `lx_checker::{CheckResult, DiagLevel, Diagnostic, check}`
  - `lx_linter::{RuleRegistry, lint}`
  - `lx_fmt::format`
  - `lx_eval::{interpreter::Interpreter, runtime::{RuntimeCtx, ToolDecl}}`
  - `lx_value::error::LxError`
- The `lx` crate is only consumed by:
  - `crates/lx-cli/src/agent_cmd.rs`
  - `crates/lx-cli/src/check.rs`
  - `crates/lx-cli/src/fmt.rs`
  - `crates/lx-cli/src/main.rs`
  - `crates/lx-cli/src/run.rs`
  - `crates/lx-cli/src/testing.rs`
  - `crates/lx/tests/formatter_roundtrip.rs`
- `crates/lx-cli/Cargo.toml` is the only manifest that depends on `lx = { path = "../lx" }`.
- `crates/lx-desktop/Cargo.toml` still declares `pulldown-cmark = { version = "0.12", default-features = false, features = ["html"] }` directly instead of using the already-hoisted workspace dependency.
- `crates/lx-desktop/src/components/markdown_body.rs` is the only Rust call site in `lx-desktop` that uses `pulldown_cmark`.
- `crates/lx-eval/src/lib.rs` re-exports `lx_span::LX_MANIFEST`, which is another non-defining re-export and currently has no in-workspace consumers.

## Files to Modify

- `Cargo.toml`
- `crates/lx-cli/Cargo.toml`
- `crates/lx-cli/src/agent_cmd.rs`
- `crates/lx-cli/src/check.rs`
- `crates/lx-cli/src/fmt.rs`
- `crates/lx-cli/src/main.rs`
- `crates/lx-cli/src/run.rs`
- `crates/lx-cli/src/testing.rs`
- `crates/lx-desktop/Cargo.toml`
- `crates/lx-eval/src/lib.rs`
- `crates/lx-fmt/Cargo.toml`
- `crates/lx-fmt/tests/formatter_roundtrip.rs`
- `crates/lx/tests/formatter_roundtrip.rs`
- `crates/lx/Cargo.toml`
- `crates/lx/src/lib.rs`

## Steps

### Step 1: Normalize the workspace dependency declaration

In `crates/lx-desktop/Cargo.toml`, replace the crate-local `pulldown-cmark` version pin with the workspace dependency form while preserving the desktop-specific feature flags:

- Replace:
  - `pulldown-cmark = { version = "0.12", default-features = false, features = ["html"] }`
- With:
  - `pulldown-cmark = { workspace = true, default-features = false, features = ["html"] }`

Do not add a second `pulldown-cmark` entry to the workspace root. `Cargo.toml` already hoists the version there. After the manifest edit, keep `crates/lx-desktop/src/components/markdown_body.rs` compiling against the hoisted version with no behavior change.

### Step 2: Remove the `lx` shim crate from the workspace

In `Cargo.toml`, remove `crates/lx` from `[workspace].members`.

Delete the shim crate files:

- `crates/lx/Cargo.toml`
- `crates/lx/src/lib.rs`

The replacement for that crate is not a new aggregator crate. The audit rule here is direct-import usage from defining crates, not moving the same re-export surface somewhere else.

### Step 3: Remove the `lx` dependency from `lx-cli`

In `crates/lx-cli/Cargo.toml`, delete the `lx = { path = "../lx" }` dependency line.

Do not compensate by creating a new internal prelude module in `lx-cli`. Import only the concrete items each file actually uses.

### Step 4: Rewrite every `lx::prelude` consumer to direct imports

Update the CLI source files so they import from defining crates instead of `lx::prelude`:

- `crates/lx-cli/src/agent_cmd.rs`
  - Import `lex` from `lx_parser::lexer`
  - Import `parse` from `lx_parser::parser`
  - Import `FileId` from `lx_span::source`
  - Import `desugar` from `lx_desugar`
  - Import `Interpreter` from `lx_eval::interpreter`
  - Import `RuntimeCtx` from `lx_eval::runtime`
  - Import `LxError` from `lx_value::error`
- `crates/lx-cli/src/run.rs`
  - Import `lex`, `parse`, `FileId`, `desugar`, `Interpreter`, `RuntimeCtx`, and `LxError` directly from their defining crates
  - Do not recreate a wildcard import
- `crates/lx-cli/src/fmt.rs`
  - Import `lex`, `parse`, `FileId`, and `format` directly
- `crates/lx-cli/src/check.rs`
  - Import `lex`, `parse`, `FileId`, `desugar`
  - Import `check`, `CheckResult`, and `DiagLevel` from `lx_checker`
  - Import `lint` and `RuleRegistry` from `lx_linter`
- `crates/lx-cli/src/main.rs`
  - Replace `use lx::prelude::RuntimeCtx;` with `use lx_eval::runtime::{RuntimeCtx, ToolDecl};`
  - Update the manifest tool mapping to construct `ToolDecl` directly instead of `lx::prelude::ToolDecl`
- `crates/lx-cli/src/testing.rs`
  - Replace `use lx::prelude::RuntimeCtx;` with `use lx_eval::runtime::RuntimeCtx;`

When a file needs only one or two symbols, keep the import list short. Do not introduce any new crate-local prelude module as a replacement.

### Step 5: Move the formatter roundtrip integration test to the defining crate

`crates/lx/tests/formatter_roundtrip.rs` currently exists only because the removed `lx` crate re-exported parser and formatter APIs. Move that test into the formatter crate:

- Create `crates/lx-fmt/tests/formatter_roundtrip.rs`
- Port the existing test logic unchanged in behavior
- Replace `use lx::prelude::*;` with direct imports:
  - `lx_fmt::format`
  - `lx_parser::lexer::lex`
  - `lx_parser::parser::parse`
  - `lx_span::source::FileId`

Keep the fixture directory walk against `../../tests` unless the move requires a path adjustment from the new test location. If the relative path changes, update only the path string needed for the moved integration test.

After the move:

- Delete `crates/lx/tests/formatter_roundtrip.rs`
- If the new integration test needs any extra dev-dependencies beyond the existing `lx-parser` entry in `crates/lx-fmt/Cargo.toml`, add only the minimal missing ones

### Step 6: Remove the unused `LX_MANIFEST` re-export

Delete `pub use lx_span::LX_MANIFEST;` from `crates/lx-eval/src/lib.rs`.

Do not replace it with another non-defining re-export. If any compile errors appear, update those call sites to import `lx_span::LX_MANIFEST` directly.

## Verification

1. Run `cargo test -p lx-cli --tests`.
2. Run `cargo test -p lx-fmt --test formatter_roundtrip`.
3. Run `just test`.
4. Run `just rust-diagnose`.
5. Run `rg -n 'use lx::prelude|lx = \\{ path = \"\\.\\./lx\" \\}|pub use lx_span::LX_MANIFEST' Cargo.toml crates tests crates/*/Cargo.toml`.
6. Confirm the final grep returns no matches.
