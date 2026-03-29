# Unit 10: lx facade + lx-cli

## Scope

Convert the `lx` crate into a pure re-export facade (no source code of its own, just `pub use` from sub-crates). Wire lint-after-check in lx-cli's `check.rs` since the lint integration was removed from checker in unit 6.

## Prerequisites

- Units 1-9 all complete
- `lx` crate already has re-export shims from each prior unit
- lx-cli compiles and all tests pass via re-exports

## Steps

### Step 1: Finalize lx/src/lib.rs as pure facade

Replace `crates/lx/src/lib.rs` with a clean re-export file that imports from all sub-crates. No code, no constants, no module declarations that point to local source files — only `pub use` and `pub mod` that resolve to re-export shim files.

New `crates/lx/src/lib.rs`:

```rust
pub use lx_span::sym;

pub mod source {
    pub use lx_span::source::*;
    pub use lx_ast::source::*;
}

pub use lx_ast::ast;
pub use lx_ast::visitor;

pub use lx_parser::lexer;
pub use lx_parser::parser;

pub use lx_desugar::folder;

pub use lx_fmt::formatter;

pub mod checker {
    pub use lx_checker::*;
}

pub mod linter {
    pub use lx_linter::*;
}

pub mod value {
    pub use lx_value::*;
}

pub mod env {
    pub use lx_value::Env;
}

pub mod error {
    pub use lx_value::error::*;
}

pub mod event_stream {
    pub use lx_value::{entry_to_lxval, EventStream, SpanInfo, StreamEntry};
}

pub use lx_value::{BuiltinCtx, ExternalStreamSink, ModuleExports, ToolModuleHandle};

pub use lx_eval::builtins;
pub use lx_eval::interpreter;
pub use lx_eval::mcp_client;
pub use lx_eval::mcp_stream_sink;
pub use lx_eval::runtime;
pub use lx_eval::stdlib;
pub use lx_eval::tool_module;
pub use lx_eval::{LX_MANIFEST, PLUGIN_MANIFEST};
```

Rationale for `pub mod` wrappers vs `pub use`:

- **`source`**: `pub use lx_span::source;` only re-exports `FileId`, `Comment`, `CommentStore`, `CommentPlacement`. The AST-dependent types (`GlobalExprId`, `GlobalStmtId`, `GlobalPatternId`, `GlobalTypeExprId`, `GlobalNodeId`, `AttachedComment`, `CommentMap`) live in `lx_ast::source`. A `pub mod source` shim re-exports from both crates so `lx::source::GlobalExprId` and `lx::source::FileId` both work.
- **`checker`/`linter`**: `pub use lx_checker as checker;` does not allow `lx::checker::submodule` access. Use `pub mod checker { pub use lx_checker::*; }` to flatten the crate contents under the module name.
- **`value`/`env`/`event_stream`**: Unit 8's lx-value declares `mod value;`, `mod env;`, `mod event_stream;` as private modules. `pub use lx_value::value;` fails because `value` is not a public module. The items are re-exported at the lx-value crate root. Use `pub mod` wrappers that re-export the specific items downstream code expects.
- **`mcp_stream_sink`**: Declared `pub mod` in lx-eval (see unit 9) so lx-cli can construct `McpStreamSink` for the external stream sink.

### Step 2: Delete all re-export shim files

Delete these shim files from `crates/lx/src/` that were created during units 1-9:

- `crates/lx/src/sym.rs` (shim from unit 1)
- `crates/lx/src/source.rs` (shim from unit 2)
- `crates/lx/src/checker.rs` (shim from unit 6)
- `crates/lx/src/linter.rs` (shim from unit 7)
- `crates/lx/src/formatter.rs` (shim from unit 5)
- `crates/lx/src/value.rs` (shim from unit 8)
- `crates/lx/src/error.rs` (shim from unit 8)
- `crates/lx/src/env.rs` (shim from unit 8)
- `crates/lx/src/event_stream.rs` (shim from unit 8)
- `crates/lx/src/interpreter.rs` (shim from unit 9)
- `crates/lx/src/builtins.rs` (shim from unit 9)
- `crates/lx/src/stdlib.rs` (shim from unit 9)
- `crates/lx/src/runtime.rs` (shim from unit 9)
- `crates/lx/src/mcp_client.rs` (shim from unit 9)
- `crates/lx/src/tool_module.rs` (shim from unit 9)

Also delete any remaining empty directories:

- `crates/lx/src/ast/`
- `crates/lx/src/visitor/`
- `crates/lx/src/lexer/`
- `crates/lx/src/parser/`
- `crates/lx/src/folder/`
- `crates/lx/src/formatter/`
- `crates/lx/src/checker/`
- `crates/lx/src/linter/`
- `crates/lx/src/value/`
- `crates/lx/src/interpreter/`
- `crates/lx/src/builtins/`
- `crates/lx/src/stdlib/`
- `crates/lx/src/runtime/`
- `crates/lx/src/lexer/`
- `crates/lx/src/parser/`
- `crates/lx/src/folder/`

### Step 3: Clean up lx/Cargo.toml

The `lx` crate should depend on all sub-crates and nothing else. Replace the dependencies section:

```toml
[dependencies]
lx-span = { path = "../lx-span" }
lx-ast = { path = "../lx-ast" }
lx-parser = { path = "../lx-parser" }
lx-desugar = { path = "../lx-desugar" }
lx-checker = { path = "../lx-checker" }
lx-linter = { path = "../lx-linter" }
lx-fmt = { path = "../lx-fmt" }
lx-value = { path = "../lx-value" }
lx-eval = { path = "../lx-eval" }
```

Remove all direct third-party dependencies (`chrono`, `chumsky`, `dashmap`, etc.) since lx is now a facade that re-exports, not a crate with its own source code.

### Step 4: Wire lint-after-check in lx-cli

In unit 6, the `lint()` call was removed from `checker::check()` and `checker::check_with_imports()`. The lx-cli `check.rs` module must now call lint after check.

Edit `crates/lx-cli/src/check.rs`:

Add import:

```rust
use lx::linter::{RuleRegistry, lint};
```

Modify `check_file()` — after `let result = check(&program, source_arc);`, add lint:

```rust
let result = check(&program, source_arc);
let mut diagnostics = result.diagnostics;
let mut registry = RuleRegistry::default_rules();
let lint_diags = lint(&program, &result.semantic, &mut registry);
diagnostics.extend(lint_diags);
let result = CheckResult { diagnostics, source: result.source, semantic: result.semantic };
```

Wait — `CheckResult` fields may not be public for reconstruction. Check the struct definition:

```rust
pub struct CheckResult {
    pub diagnostics: Vec<Diagnostic>,
    pub source: Arc<str>,
    pub semantic: SemanticModel,
}
```

All fields are `pub`, so reconstruction works.

Apply the same pattern everywhere `check()` or `check_with_imports()` is called in `check.rs`:

1. In `check_file()` (line 31): after `let result = check(&program, source_arc);`
2. In `recheck_source()` (line 68): after `Ok(check(&program, fixed_arc))`
3. In `check_workspace()` loop (line 141): after `let result = check(&program, source_arc);`

For each call site, insert the lint integration:

```rust
fn lint_after_check(result: CheckResult, program: &lx::ast::Program<lx::ast::Core>) -> CheckResult {
    let mut diagnostics = result.diagnostics;
    let mut registry = RuleRegistry::default_rules();
    let lint_diags = lint(program, &result.semantic, &mut registry);
    diagnostics.extend(lint_diags);
    CheckResult { diagnostics, source: result.source, semantic: result.semantic }
}
```

Add this helper function to `check.rs` and call it at each check site:

In `check_file()`:
```rust
let result = lint_after_check(check(&program, source_arc), &program);
```

In `recheck_source()`:
```rust
Ok(lint_after_check(check(&program, fixed_arc), &program))
```

In `check_workspace()`:
```rust
let result = lint_after_check(check(&program, source_arc), &program);
```

### Step 5: Update lx-cli Cargo.toml

If lx-cli currently depends only on `lx`, no changes needed — the facade handles everything. Verify `crates/lx-cli/Cargo.toml` has:

```toml
lx = { path = "../lx" }
```

### Step 6: Verify lx-cli main.rs imports

All existing imports in `crates/lx-cli/src/main.rs` use `lx::` prefix:

- `use lx::runtime::RuntimeCtx;` — works via facade
- `lx::value::LxVal` — works via facade
- `lx::runtime::ControlChannelState` — works via facade
- `lx::runtime::ControlYieldBackend` — works via facade
- `lx::mcp_client::McpClient` — works via facade

In `main.rs` `setup_external_stream()`, the call:

```rust
lx::mcp_client::McpClient::spawn(&command).await
```

works via facade. But `ctx.event_stream.set_external_client(client_arc)` now needs an `Arc<dyn ExternalStreamSink>` instead of `Arc<tokio::sync::Mutex<McpClient>>`. Update:

```rust
match lx::mcp_client::McpClient::spawn(&command).await {
    Ok(client) => {
        let client_arc = Arc::new(tokio::sync::Mutex::new(client));
        let sink = Arc::new(lx::mcp_stream_sink::McpStreamSink::new(client_arc));
        ctx.event_stream.set_external_client(sink);
    },
    Err(e) => {
        eprintln!("[stream:external] failed to connect to '{command}': {e}");
    },
}
```

`mcp_stream_sink` is declared `pub mod` in lx-eval (unit 9) and re-exported via the lx facade (`pub use lx_eval::mcp_stream_sink;` in step 1's lib.rs). No additional changes needed.

### Step 7: Final workspace Cargo.toml

Verify `/home/entropybender/repos/lx/Cargo.toml` has all workspace members:

```toml
members = [
    "crates/lx",
    "crates/lx-api",
    "crates/lx-cli",
    "crates/lx-desktop",
    "crates/lx-macros",
    "crates/lx-mobile",
    "crates/lx-span",
    "crates/lx-ast",
    "crates/lx-parser",
    "crates/lx-desugar",
    "crates/lx-checker",
    "crates/lx-linter",
    "crates/lx-value",
    "crates/lx-eval",
    "crates/lx-fmt",
]
```

(lx-fmt is from unit 5, included for completeness.)

## Files touched

| Action | File |
|--------|------|
| REWRITE | `crates/lx/src/lib.rs` (pure facade) |
| DELETE | All shim files in `crates/lx/src/` (12+ files) |
| DELETE | All empty directories in `crates/lx/src/` |
| REWRITE | `crates/lx/Cargo.toml` (sub-crate deps only) |
| EDIT | `crates/lx-cli/src/check.rs` (add lint-after-check wiring) |
| EDIT | `crates/lx-cli/src/main.rs` (update set_external_client call to use McpStreamSink) |

## Verification

1. `just diagnose` passes with zero warnings
2. `just test` passes (all .lx suite tests)
3. `crates/lx/src/` contains only `lib.rs` — no other source files
4. `lx` crate `Cargo.toml` has zero third-party dependencies (only sub-crate path deps)
5. `lx-cli` compiles and `lx check`, `lx test`, `lx run`, `lx fmt`, `lx agent` all work
6. Lint diagnostics appear in `lx check` output (lint-after-check wiring works)
7. External stream integration works (`lx run --control stdin` with stream config)
8. No file exceeds 300 lines
9. The full DAG is: lx-span -> lx-ast -> {lx-parser, lx-desugar, lx-checker -> lx-linter, lx-fmt, lx-value -> lx-eval} -> lx
