# Implementation Plan

Architecture, crate choices, and build strategy for `lx`.

## Architecture

```
crates/lx/          -- core library (lexer, parser, type checker, interpreter)
crates/lx-cli/      -- `lx` binary (run, fmt, test, check, agent, diagram)
```

Two crates. The library is the language engine. The CLI is a thin shell. This split lets the lx engine be embedded (MCP server, agent processes, editor integration) without pulling in CLI deps.

## Why Hand-Written Lexer + Pratt Parser

Parser generators were evaluated. Hand-written wins for lx:
- **Modal lexing**: `$` → shell mode, `r/` → regex mode, `"..."` → interpolation. Needs a state machine.
- **Pratt parsing**: 17-level precedence table with postfix `^`, sections `(* 2)`, three `?` modes maps directly to binding powers.
- **Error recovery**: synchronization points are trivial hand-written.
- **Small grammar**: ~9 keywords, ~20 operators. ~500 lines of parser (split across files).

## Crate Choices

| Crate | Purpose |
|-------|---------|
| `miette` + `thiserror` | Error diagnostics with source context |
| `clap` v4 derive | CLI argument parsing |
| `num-bigint` / `num-traits` / `num-integer` | Arbitrary-precision integers |
| `indexmap` | Ordered maps (records, maps) |
| `regex` | `r/pattern/` literals + `std/re` |
| `serde_json` (preserve_order) | JSON conversion, agent/MCP protocol |
| `pulldown-cmark` | `std/md` markdown parsing |
| `reqwest` (blocking, json) | `std/mcp` HTTP transport, `std/http` |
| `chrono` | `std/time` timestamp formatting/parsing |
| `cron` | `std/cron` cron expression parsing + scheduling |
| `strum` (derive) | Enum Display/IntoStaticStr derives |
| `dashmap` | Concurrent registries (agent, mcp, tool defs, trace stores) |
| `parking_lot` | Fast Mutex for Env, module cache |

## Data Flow

```
source: &str
  → Lexer → Vec<Token>
  → Parser → Ast
  → Checker → Ast + TypeInfo  (optional, `lx check` only)
  → Interpreter → Value       (tree-walking, receives &Arc<RuntimeCtx>)
```

## Key Types

```
Token { kind: TokenKind, span: Span }
Span { offset: u32, len: u16 }
Expr — enum of all expression forms (~30 variants)
Value — enum of runtime values (~20 variants)
Env — scope chain (HashMap<String, Value> + parent Arc)
RuntimeCtx — backend traits (AI, HTTP, shell, emit, yield, log)
BuiltinFn: fn(&[Value], Span, &Arc<RuntimeCtx>) -> Result<Value, LxError>
```

## Scale

~17800 lines of Rust across 2 crates: lexer, parser, checker, interpreter, AST, builtins, 29 stdlib modules, backends, CLI. 45 test files. 62 spec files.

## Concurrency

Currently **sequential**. `par`/`sel`/`pmap` evaluate in order. Real async (tokio) is planned.

## Build Phases

See [implementation-phases.md](implementation-phases.md) for the detailed phase breakdown.
