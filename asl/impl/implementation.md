# Implementation Plan

Architecture, crate choices, and phased build plan for `lx`.

## Architecture

```
crates/lx/          -- core library (lexer, parser, type checker, interpreter)
crates/lx-cli/      -- `lx` binary (run, fmt, test, check, build, repl, notebook, watch, init)
```

Two crates. The library is the language engine — everything from source text to execution. The CLI is a thin shell that wires up subcommands to library calls. This split lets the lx engine be embedded in other tools (MCP server, editor integration, REPL) without pulling in CLI deps.

## Why Hand-Written Lexer + Pratt Parser

Parser generators (lalrpop, pest, chumsky) and lexer generators (logos) were evaluated. A hand-written approach wins for lx specifically:

**Lexing has modal transitions.** `$` switches to shell mode (raw text until newline). `r/` switches to regex mode. `$$` and `$^` are single tokens. `"..."` has `{expr}` interpolation which re-enters expression mode mid-string. Logos can handle simple tokens but the mode switching requires a hand-written state machine anyway — at that point logos adds complexity without reducing code.

**Pratt parsing is the natural fit for lx's precedence table.** The 17-level precedence table with postfix `^`, sections `(* 2)`, and the three `?` modes maps directly to a Pratt parser. Each precedence level is a number. Adding/changing operators means changing a number. Parser generators require encoding precedence in grammar rules, which is indirect and harder to modify.

**Error recovery requires hand-written control.** The spec says "recover after first error, report up to 5." Synchronization points (find the next `}`, `;`, or newline and resume) are trivial in a hand-written parser but require framework-specific APIs in generators.

**The grammar is small.** 9 keywords, ~20 operators, ~6 expression forms. This is not C++ or Rust. A hand-written parser for lx is ~400-500 lines (split across files), well within maintenance bounds.

## Crate Choices

### Core Pipeline (lexer → parser → checker → interpreter)

**No external crate for lexing or parsing.** Hand-written, as argued above. The lexer is a state machine over `&str` producing `Token` values with `Span` (byte offset + length). The parser is recursive descent with Pratt precedence climbing.

### Error Diagnostics: `miette`

[miette](https://github.com/zkat/miette) produces the exact diagnostic format lx needs: source spans, underlined expressions, "expected/got/fix" labels. Used by stilts in reference/. Supports both human-readable and JSON output (`--json` flag maps to `miette::JSONReportHandler`). The alternative (ariadne) is also good but miette's JSON support matches the spec requirement for `lx run --json`.

### Arbitrary Precision Integers: `num-bigint` + `num-traits`

The spec says integers are arbitrary precision by default. `num-bigint` is the standard Rust crate for this. `num-traits` provides the `Zero`, `One`, and arithmetic traits. Together they're ~15k lines of well-tested bigint math. No reason to hand-write this.

### Regex Runtime: `regex`

Already in the workspace. Powers `r/pattern/flags` at runtime. The lx regex literal compiles to a `regex::Regex` value.

### Async Runtime: `tokio`

Already the workspace standard. Powers `par`/`sel`/`pmap` via `tokio::task::JoinSet` (structured concurrency). Shell commands via `tokio::process::Command`. The lx interpreter runs on tokio and uses it for all concurrent and I/O operations.

### HTTP Client: `reqwest`

Already in the workspace. Powers `std/net/http`. The lx runtime wraps reqwest calls.

### JSON/TOML/YAML/CSV: `serde_json`, `toml`, `serde_yaml`, `csv`

Standard ecosystem crates. lx values serialize to/from serde's data model. A lx map becomes a serde map, a lx list becomes a serde sequence.

### REPL: `rustyline`

Line editing, history, and completion for `lx repl`. Lightweight, well-maintained. The alternative (reedline) is heavier but has more features — rustyline is sufficient for v1.

### File Watching: `notify`

Powers `lx watch`. Cross-platform filesystem event notifications. Already battle-tested.

### Hashing: `sha2`, `md-5`, `hmac`

Powers `std/crypto`. Standard RustCrypto crates.

### Random: `rand`

Already in reference/. Powers `std/rand`.

### Data Ecosystem (Phase 11)

- **polars** — already in reference/. Powers `std/df`. LazyFrame maps to lx's lazy evaluation model.
- **rusqlite** + **duckdb** — powers `std/db`. SQLite for transactional, DuckDB for analytical queries.
- **ndarray** — powers `std/num`. Contiguous typed arrays with SIMD-friendly operations.
- **candle-core** / **ort** — powers `std/ml`. Local ML inference (embeddings, classification).
- **charming** — already in reference/. Powers `std/plot`. SVG chart generation.

### Not Needed (v1)

- **cranelift / LLVM** — AOT compilation (`lx build`) is v2. v1 is interpreted.
- **tree-sitter** — lx doesn't need incremental parsing. Full parse on every run.
- **serde derive on AST** — AST nodes don't need serialization. Only lx *values* do.

## Data Flow

```
source: &str
  → Lexer → Vec<Token>        (or streaming iterator)
  → Parser → Ast              (tree of Expr/Stmt nodes)
  → Checker → Ast + TypeInfo  (bidirectional type inference, warnings)
  → Interpreter → Value       (tree-walking execution)
```

Each stage is a separate module. Each stage's output is the next stage's input. The checker is optional (`lx run` can skip it for speed; `lx check` runs it alone).

## Key Types

```
Token { kind: TokenKind, span: Span }
Span { offset: u32, len: u16 }
Expr — enum of all expression forms (Literal, Binary, Pipe, Match, Shell, Par, Sel, ...)
Stmt — Binding | ExprStmt
Value — enum of runtime values (Int, Float, Str, Bool, List, Record, Map, Set, Tuple, Fn, ...)
Env — scope chain (HashMap<String, Value> + parent pointer)
```

`Expr` is the big enum — one variant per grammar production. `Value` is the runtime representation. The interpreter walks `Expr` and produces `Value`.

## Concurrency Implementation

`par { a; b; c }` compiles to: spawn each expression as a tokio task via `JoinSet`, await all, cancel on first error.

`sel { expr1 -> handler1; expr2 -> handler2 }` compiles to: spawn each expression, `tokio::select!` on the first to complete, cancel others, run the winning handler.

`pmap f xs` compiles to: spawn `f(x)` for each element via `JoinSet`, collect results in order.

The interpreter's `eval` function is `async fn eval(&mut self, expr: &Expr) -> Result<Value>`. Everything is async from the start — synchronous operations just `.await` immediately.

## Module Structure

```
crates/lx/src/
  lib.rs              -- pub use of all modules
  span.rs             -- Span, source location types
  token.rs            -- TokenKind enum, Token struct
  lexer.rs            -- Lexer state machine (modes: normal, shell, regex, string interp)
  ast.rs              -- Expr, Stmt, Pattern, Type enums
  parser.rs           -- recursive descent + Pratt precedence
  parser_expr.rs      -- expression parsing (split from parser.rs for 300-line limit)
  parser_pattern.rs   -- pattern parsing
  checker.rs          -- bidirectional type inference
  checker_types.rs    -- TypeInfo, unification, structural subtyping
  value.rs            -- Value enum, to_str, equality
  env.rs              -- Env scope chain, variable lookup
  interpreter.rs      -- tree-walking eval
  interpreter_shell.rs -- shell command execution
  interpreter_conc.rs -- par/sel/pmap implementation
  builtins.rs         -- built-in functions (map, filter, fold, etc.)
  builtins_str.rs     -- string functions
  builtins_io.rs      -- io/fs/env functions
  stdlib.rs           -- module loader for std/fs, std/net/http, etc.
  error.rs            -- LxError type, diagnostic formatting
  fmt.rs              -- lx fmt (canonical formatter)
```

Each file ≤300 lines. The split points are natural — shell execution, concurrency, and string builtins are each independent concerns.

## Phased Build Plan

See [implementation-phases.md](implementation-phases.md) for the detailed phase breakdown.
