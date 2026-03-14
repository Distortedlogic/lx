# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is an agentic workflow language you (Claude) are designing and building. You are both the language designer and the implementer. The central purpose is enabling agents to write executable programs for agentic workflows — agent spawning, inter-agent communication, tool invocation (MCP), context persistence, and workflow orchestration. The syntax is optimized for LLM token generation: left-to-right, zero lookahead, minimal surface area.

**Identity:** lx is not a general scripting language that happens to have agent features. It is an agentic workflow language where the core primitives are agent communication, tool invocation, and workflow composition. The LLM-optimized syntax is a property, not the purpose. The niche — "a language agents write in to orchestrate other agents" — is essentially empty.

## Continuity Protocol

1. Read `asl/DEVLOG.md` — your memory across sessions. Has implementation status, key design decisions, known tensions, session history, and what needs doing next.
2. Read `asl/README.md` — directory structure and file index.
3. The three folders are one system:
   - `asl/spec/` — what lx IS (language specification)
   - `asl/impl/` — how to BUILD it (Rust implementation design docs)
   - `asl/suite/` — PROOF they agree (.lx golden test files)
4. `crates/lx/` — the actual Rust implementation
5. `crates/lx-cli/` — the `lx` binary
6. `justfile` — build recipes (`just test`, `just diagnose`, `just fmt`, `just run <file>`)

## Your Authority

You own this language. You can freely:
- **Expand** the spec — add new constructs, nail down underspecified areas, write new spec files
- **Rethink** decisions — if something feels wrong after reading it fresh, change it
- **Fill gaps** — if two docs contradict, fix both; if an example doesn't work under the rules, fix the example or fix the rule
- **Add test files** — write .lx files that prove the spec and implementation agree
- **Refactor impl docs** — restructure, split, merge, rewrite implementation design docs
- **Write Rust code** — implement features in crates/lx/ and crates/lx-cli/

You do NOT need permission to make changes. The spec, impl docs, and suite are yours to evolve. The only constraint is internal consistency — the three folders must agree with each other and with the Rust implementation.

## Cross-Referencing

When you change something, update all places that reference it:
- Spec change → update impl doc that describes how it's built → update suite test that covers it → update Rust code if implemented
- Impl change → verify spec still matches → verify tests still pass
- Suite change → verify it matches the spec rules
- Rust code change → verify it matches impl design → verify suite tests pass

## Session Workflow

At the end of every session, update `asl/DEVLOG.md`:
- Add a session entry describing what you found and changed
- Update "What Needs Doing Next"
- Note any new tensions or open questions
- Trim anything no longer relevant

Then update this file (`NEXT_PROMPT.md`) with accurate current state.

## Current State

`just diagnose` is clean — zero warnings, zero clippy errors.
`just test`: **17/17 PASS** — all tests passing.

The core language is feature-complete through Phase 8. The agentic workflow loop is **closed**: agents spawn as subprocesses, communicate via `~>`/`~>?`, invoke MCP tools (stdio or HTTP), and persist context. 9 stdlib modules implemented. MCP HTTP streaming transport complete.

### What's implemented

- Arithmetic, bindings, strings, interpolation, collections, pattern matching
- Functions, closures, currying, default params, pipes, sections, composition `<>`
- Type annotations (parse-and-skip), regex literals, slicing, named args
- Type definitions with tagged values and pattern matching
- Iterator protocol (lazy `map`/`filter`/`take` on infinite sequences)
- Concurrency: `par`, `sel`, `pmap`, `pmap_n`, `timeout` (sequential impl)
- Shell integration: `$cmd`, `$$cmd`, `$^cmd`, `${...}` with interpolation
- Error handling: `^` propagation, `??` coalescing, `(?? default)` sections, implicit Err return
- Module system: `use ./path`, aliasing, selective imports, `+` exports
- Agent communication: `~>` (send), `~>?` (ask) — infix operators, subprocess-transparent
- Message contracts: `Protocol Name = {field: Type}` with runtime validation
- 9 stdlib modules: `std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent`, `std/mcp`
- MCP HTTP streaming transport (`reqwest` blocking, SSE parsing, session management)
- `lx agent` subcommand for subprocess agent mode

### Important syntax notes

- Tuple creation with variables needs semicolons: `(b; a)` not `(b a)` (Idents are callable)
- Generic return types need parens: `-> (Tree a)` not `-> Tree a`
- Records/maps use collection-mode: `{x: (f 42)}` for function calls in field values
- Shell `$` consumes full line; wrap in parens for expressions: `($cmd) ? { ... }`
- `~>?` composes with `^` and `|`: `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`
- `assert (expr) "msg"` — if `(expr)` is callable, parser consumes `"msg"` as arg. Use `assert (expr == true) "msg"`

## Critical Reading

**Read `asl/CURRENT_OPINION.md` for design context.** Priorities A–D DONE. Remaining: E (implicit context scope), F (resumable workflows).

## What To Work On Next

The agentic core is complete. The next work falls into three categories:

### 1. MCP HTTP Streaming Transport (HIGH PRIORITY)

`std/mcp` currently uses stdio transport only. The real-world transport is **HTTP streaming (SSE)**. This requires adding `reqwest` + `tokio` dependencies and implementing Streamable HTTP transport in `mcp_rpc.rs`. The stdio transport stays as a fallback for local servers.

### 2. Remaining stdlib modules (Phase 9)

These make lx useful for general scripting beyond agentic workflows.

| Module | Rust crate | Key functions |
|--------|-----------|---------------|
| `std/http` | `reqwest` | get, post, put, delete |
| `std/time` | `chrono` | now, format, parse, sleep |
| `std/rand` | `rand` | int, float, choice, shuffle |
| `std/io` | `std::io` | read_line, print |
| `std/csv` | `csv` | parse, encode |
| `std/toml` | `toml` | parse, encode |
| `std/yaml` | `serde_yaml` | parse, encode |
| `std/crypto` | `sha2`, `hmac` | sha256, hmac |
| `std/os` | — | pid, hostname, platform |
| `std/fmt` | — | pad, truncate |
| `std/bit` | — | and, or, xor, shift |

### 3. Language design work (Priorities E–F)

- **Implicit context scope (Priority E)** — eliminate manual state threading. `with` block or implicit parameter so agent functions don't manually pass state around.
- **Resumable workflows (Priority F)** — workflows as inspectable, checkpointable values. If step 3 of 5 fails, resume from step 3.

### 4. Technical debt

- **300-line limit violations**: prefix.rs (773), parser/mod.rs (640+), interpreter/mod.rs (520+), hof.rs (425), value.rs (330). These are the core files — splitting them improves readability and context-friendliness.
- **Fake concurrency**: `par`/`sel`/`pmap` are sequential. Real threading/async requires `tokio`.
- **Parser fragility**: named-arg/ternary conflict, assert greedy parsing, `is_func_def` heuristics.
- **Stale spec files**: `examples.md`, `examples-extended.md`, `toolchain.md` still use `agent.ask`/`agent.send` library syntax instead of `~>`/`~>?`.

### 5. Toolchain (Phase 10)

| Tool | Purpose | Crate |
|------|---------|-------|
| `lx fmt` | Canonical formatter | — |
| `lx repl` | Interactive mode | `rustyline` |
| `lx check` | Type/contract validation | — |
| `lx watch` | Re-run on file change | `notify` |

### 6. Data ecosystem (Phase 11, optional)

| Module | Rust crate | Purpose |
|--------|-----------|---------|
| `std/df` | `polars` | DataFrames |
| `std/db` | `rusqlite`, `duckdb` | SQL |
| `std/num` | `ndarray` | Vectors/stats |
| `std/ml` | `candle-core` / `ort` | ML inference |
| `std/plot` | `charming` | Charts |

## Codebase Layout

```
crates/lx/src/
  lexer/     mod.rs, numbers.rs, strings.rs
  parser/    mod.rs, prefix.rs, pattern.rs
  interpreter/ mod.rs, apply.rs, collections.rs, modules.rs, patterns.rs, shell.rs
  builtins/  mod.rs, str.rs, coll.rs, hof.rs
  stdlib/    mod.rs, json.rs, json_conv.rs, ctx.rs, math.rs, fs.rs, env.rs, re.rs, md.rs, md_build.rs, agent.rs, mcp.rs, mcp_rpc.rs
  ast.rs, token.rs, value.rs, env.rs, error.rs, span.rs, iterator.rs, lib.rs
crates/lx-cli/src/main.rs
asl/suite/fixtures/
  agent_echo.lx         -- echo handler for std/agent tests
  mcp_test_server.py    -- minimal MCP server for std/mcp tests
```

## Dependencies (audited 2026-03-14)

External crates already cover every area where an established solution exists:

| Crate | Purpose |
|-------|---------|
| `miette` + `thiserror` | Error diagnostics with source context |
| `clap` v4 derive | CLI argument parsing |
| `num-bigint` / `num-traits` / `num-integer` | Arbitrary-precision integers |
| `indexmap` | Ordered maps/sets (records, maps, sets) |
| `regex` | Regex literals, string builtins, `std/re` |
| `serde_json` (preserve_order) | `std/json`, `std/ctx` JSON conversion, agent/MCP subprocess protocol |
| `pulldown-cmark` | `std/md` markdown parsing |

The remaining ~5000 lines of custom code (lexer, parser, interpreter, AST, env, builtins, iterators, span, stdlib) is all language-implementation-specific — no generic crate replaces a Pratt parser with shell-mode lexing, or builtins operating on lx's `Value` type. Do not spend time looking for crate replacements for these; they were audited and none apply.

When adding **new** stdlib modules, use established crates for the heavy lifting: `reqwest`, `tokio`, `chrono`, etc.

## Rules

- No code comments or doc strings in Rust files
- No `#[allow(...)]` macros
- 300 line file limit for ALL files (spec, impl, suite, Rust) — some files currently exceed this, need refactoring
- Never swallow errors (`let _ = ...`, `.ok()`, silent `unwrap_or_default()`)
- `just diagnose` must stay clean (check + clippy with -D warnings)
- `just test` to run all suite tests, `just run <file>` for single files
- Prefer established crates over custom code — check `reference/` submodules first
