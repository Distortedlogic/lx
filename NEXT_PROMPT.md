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
`just test`: **16/16 PASS** — all tests passing.

Phases 1–8 are all implemented (including Phase 7 modules), plus agent communication syntax, message contracts, stdlib infrastructure, and MCP tool invocation. The interpreter handles:
- Arithmetic, bindings, strings, interpolation, collections, pattern matching
- Functions, closures, currying, default params (with auto-execution), pipes, sections (right/left/binop/field/index), composition `<>`
- Type annotations (parse-and-skip for params, return types, bindings, complex types like `{name: Str}`, `[Int]`, `%{Str: Int}`, `Int -> Int`, `(Int -> Int) -> Int`)
- Shell integration: `$cmd` (full result), `$$cmd` (raw/no interp), `$^cmd` (stdout/propagate), `${...}` (multi-line block). Depth-aware paren stopping for `($cmd)` in expressions.
- Collection-depth reset in parens: `[(f x)]` correctly applies `f` to `x` even inside list literals
- Regex literals `r/pattern/flags`
- Slicing `xs.1..3`, `xs.2..`, `xs...3`
- Named args `f x name: "val"` with param-position matching
- Type definitions `Shape = | Circle Float | Rect Float Float` with tagged values and pattern matching (including paren/bracket/brace-wrapped variant types like `Node (Tree a) (Tree a)`)
- Nested tuple patterns in function params `fst = ((a _)) a` (desugars to synthetic names + destructuring)
- Iterator protocol: `nat` (infinite naturals), `cycle` (infinite cycle), record-with-`next` custom iterators. Lazy composition via `map`/`filter` on iterators; eager consumption via `take`/`collect`.
- Concurrency (sequential impl): `par { ... }` (parallel block → tuple), `sel { expr -> handler }` (race/select with `it` binding), `pmap` (parallel map), `pmap_n` (rate-limited pmap), `timeout`
- 29 HOF builtins: map, filter, fold, flat_map, each, take, drop, zip, enumerate, find, any?, all?, none?, count, take_while, drop_while, sort_by, min_by, max_by, partition, group_by, chunks, windows, intersperse, scan, tap, find_index, pmap, pmap_n
- ~30 collection/string/conversion builtins
- Loop/break with values
- Error propagation `^` (unwraps Ok/Some, propagates Err/None at function boundaries)
- Coalescing `??` (unwraps Ok/Some, evaluates default for Err/None)
- `(?? default)` sections
- Implicit Err early return in `-> T ^ E` annotated functions
- Match with record/list/constructor/string/tagged patterns, guards, destructuring
- Collection-mode application in `[]`, `#{}`, `%{}`, and `{}` records (only TypeConstructors trigger application)
- Tuple destructuring bindings `(a b) = expr`
- Multiline continuation (leading and trailing operators)
- Block-scoped function bodies `(x) { body }` don't consume pipes
- Multiline string auto-dedent (strings starting with `\n` strip common indentation)
- Module system: `use ./path` (whole), `use ./path : alias`, `use ./path {name1 name2}` (selective), variant constructor scoping, module caching, circular import detection
- Agent communication: `~>` (send, fire-and-forget), `~>?` (ask, request-response). Infix operators at concat/diamond precedence. Agents are records with `handler` field.
- Message contracts: `Protocol Name = {field: Type}` keyword. Runtime structural validation. Defaults, `Any` type, structural subtyping. Exportable/importable.
- Stdlib infrastructure: `use std/json` routes to Rust-native modules via `crates/lx/src/stdlib/`
- `std/json`: `parse`, `encode`, `encode_pretty` via `serde_json`
- `std/ctx`: `empty`, `load`, `save`, `get`, `set`, `remove`, `keys`, `merge` — context persistence to JSON files
- `std/math`: `abs`, `ceil`, `floor`, `round`, `pow`, `sqrt`, `min`, `max`, `pi`, `e`, `inf`
- `std/fs`: `read`, `write`, `append`, `exists`, `remove`, `mkdir`, `ls`, `stat`
- `std/env`: `get`, `vars`, `args`, `cwd`, `home`
- `std/re`: `match`, `find_all`, `is_match`, `replace`, `replace_all`, `split` (accepts Str or regex literal)
- `std/md`: `parse`, `sections`, `code_blocks`, `headings`, `links`, `to_text`, `render` + builders (`h1`-`h3`, `para`, `code`, `list`, `ordered`, `table`, `link`, `blockquote`, `hr`, `raw`, `doc`) via `pulldown-cmark`
- `std/agent`: `spawn`, `ask`, `send`, `kill`, `name`, `status` — subprocess agent lifecycle with JSON-line protocol
- `std/mcp`: `connect`, `close`, `list_tools`, `call`, `list_resources`, `read_resource`, `list_prompts`, `get_prompt` — MCP client over stdio via JSON-RPC 2.0
- `lx agent` subcommand: runs a script as an agent subprocess (handler function + JSON-line message loop)

**Important syntax notes:**
- Tuple creation with variables needs semicolons: `(b; a)` not `(b a)` (Idents are callable)
- Generic return types need parens: `-> (Tree a)` not `-> Tree a` (parse-and-skip limitation)
- Records/maps use collection-mode: `{x: (f 42)}` for function calls in field values
- Shell `$` consumes full line: to use shell results in expressions, wrap in parens: `($cmd) ? { ... }`
- Shell `$^` stops at first `|` for language pipe: `$^pwd | trim` — `pwd` is shell, `| trim` is lx
- Inside parens/brackets (depth > 0), `$` stops at `)`: `($echo "hello")` works
- `~>` (send) and `~>?` (ask) are infix: `agent ~>? msg`. Agent = record with `handler` field. Subprocess agents (from `agent.spawn`) have `__pid` field — `~>`/`~>?` transparently routes to subprocess I/O.
- `~>?` composes with `^` and `|`: `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`
- `Protocol Name = {field: Type}` declares record shape validators. Applied like functions: `Name {field: val}`. Runtime error on mismatch.
- Strings: `{` starts interpolation, use `\{` for literal braces. `}` is literal outside interpolation. `\"` for literal quotes. JSON strings: `"\{\"key\": \"val\"}"`.
- `assert (expr) "msg"` — if `(expr)` evaluates to something callable, the parser consumes `"msg"` as an arg. Use `assert (expr == true) "msg"` pattern.

## Critical Reading

**Read `asl/CURRENT_OPINION.md` for design context.** Priorities A–D DONE (agent communication, message contracts, stdlib infrastructure, agent-specific stdlib including MCP). Remaining: E (implicit context scope), F (resumable workflows).

## What To Work On Next

All 16 existing test files pass. The core language is feature-complete through Phase 8, plus agent communication (`~>` / `~>?` with subprocess support), message contracts (`Protocol`), and 9 stdlib modules (`std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent`, `std/mcp`).

### Step 1: ~~Design agent communication syntax~~ ✓ DONE

Implemented `~>` (send) and `~>?` (ask) as language-level infix operators. Tokens: `TildeArrow`, `TildeArrowQ`. AST: `Expr::AgentSend`, `Expr::AgentAsk`. Precedence: (21, 22) — same as concat/diamond. Sequential evaluation. Agents are records with `handler` field. Test: `14_agents.lx`.

### Step 2: ~~Message contracts~~ ✓ DONE

Implemented `Protocol` keyword with runtime structural validation. `Protocol Name = {field: Type  field2: Type = default}` declares record shape validators. Protocol values are callable — apply to a record to validate. Returns validated record on success (defaults filled in), runtime error on failure. Extra fields allowed (structural subtyping). `Any` type skips checking. Protocols are exportable/importable. Tests in `14_agents.lx`.

### Step 3: ~~`std/` import infrastructure~~ ✓ DONE

Implemented `use std/...` routing in `interpreter/modules.rs`. Stdlib modules are Rust-native builtins in `crates/lx/src/stdlib/`. Adding a new module = add a file + match arm. `std/json` implemented with `parse`, `encode`, `encode_pretty`.

### Step 4: ~~Core agent stdlib modules~~ DONE

Built ON TOP of the language primitives from Steps 1-2. All three agent-specific modules implemented.

| Module | Rust crate | Purpose |
|--------|-----------|---------|
| ~~`std/json`~~ ✓ | `serde_json` | parse, encode, encode_pretty |
| ~~`std/ctx`~~ ✓ | `serde_json` | Context: empty, load, save, get, set, remove, keys, merge |
| ~~`std/md`~~ ✓ | `pulldown-cmark` | Markdown parse/extract/build/render (20 functions) |
| ~~`std/mcp`~~ ✓ | JSON-RPC/stdio | MCP client: connect, close, list_tools, call, list_resources, read_resource, list_prompts, get_prompt |
| ~~`std/agent`~~ ✓ | `std::process` | Agent spawn, ask, send, kill, name, status + `lx agent` subcommand |

### Step 5: Remaining stdlib (Phase 9)

Lower priority — these make lx useful for general scripting but aren't the differentiator.

| Module | Rust crate | Key functions |
|--------|-----------|---------------|
| ~~`std/fs`~~ ✓ | `std::fs` | read, write, append, exists, remove, mkdir, ls, stat |
| ~~`std/env`~~ ✓ | `std::env` | get, vars, args, cwd, home |
| ~~`std/re`~~ ✓ | `regex` | match, find_all, replace, replace_all, split, is_match |
| ~~`std/math`~~ ✓ | — | abs, ceil, floor, round, pow, sqrt, min, max, pi, e, inf |
| `std/http` | `reqwest` | get, post, put, delete |
| `std/time` | `chrono` | now, format, parse, sleep |
| `std/io` | `std::io` | read_line, print |
| `std/csv` | `csv` | parse, encode |
| `std/toml` | `toml` | parse, encode |
| `std/yaml` | `serde_yaml` | parse, encode |
| `std/rand` | `rand` | int, float, choice, shuffle |
| `std/crypto` | `sha2`, `hmac` | sha256, hmac |
| `std/os` | — | pid, hostname, platform |
| `std/fmt` | — | pad, truncate |
| `std/bit` | — | and, or, xor, shift |

### Step 6: Toolchain (Phase 10)

| Tool | Purpose | Crate |
|------|---------|-------|
| `lx fmt` | Canonical formatter | — |
| `lx repl` | Interactive mode | `rustyline` |
| `lx check` | Type/contract validation | — |
| `lx agent` | Long-lived agent process | `tokio` |
| `lx watch` | Re-run on file change | `notify` |

### Step 7: Data ecosystem (Phase 11, optional)

| Module | Rust crate | Purpose |
|--------|-----------|---------|
| `std/df` | `polars` | DataFrames |
| `std/db` | `rusqlite`, `duckdb` | SQL |
| `std/num` | `ndarray` | Vectors/stats |
| `std/ml` | `candle-core` / `ort` | ML inference |
| `std/plot` | `charming` | Charts |

### Other remaining work:
- Real threading/async for `par`/`sel`/`pmap` (currently sequential)
- Propagation traces for `^`
- Implicit context scope (see CURRENT_OPINION.md Priority E)
- Resumable workflows (see CURRENT_OPINION.md Priority F)

### Known technical debt:
- Rust files exceeding 300-line limit: prefix.rs (773), parser/mod.rs (640+), interpreter/mod.rs (520+), hof.rs (425), value.rs (330)
- `par`/`sel`/`pmap` and `~>`/`~>?` are sequential; the spec describes concurrent execution
- Named-arg parser consumes ternary `:` separator: `true ? Ok x : 0` misparses because `x :` looks like a named arg. Workaround: `(Ok x)`
- Assert parsing is greedy: `assert (expr) "msg"` can consume the message as a function application arg if `(expr)` is callable. Workaround: use `assert (expr == true) "msg"` or bind to a variable.
- Stale spec files: `examples.md`, `examples-extended.md`, `toolchain.md` still use `agent.ask`/`agent.send` library syntax. These should use `~>` / `~>?` when updated. The authoritative agent spec is `agents.md`.
- `stdlib-agents.md` spec shows `agent.ask`/`agent.send` as library functions — `std/agent` now implements these alongside `~>`/`~>?` (both work, library functions take explicit agent arg, operators use infix syntax).

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
| `serde_json` (preserve_order) | `std/json`, `std/ctx` JSON conversion, agent subprocess protocol |
| `pulldown-cmark` | `std/md` markdown parsing |

The remaining ~4800 lines of custom code (lexer, parser, interpreter, AST, env, builtins, iterators, span) is all language-implementation-specific — no generic crate replaces a Pratt parser with shell-mode lexing, or builtins operating on lx's `Value` type. Do not spend time looking for crate replacements for these; they were audited and none apply.

When adding **new** stdlib modules (std/json, std/http, std/fs, etc.), use established crates for the heavy lifting: `serde_json`, `reqwest`, `tokio`, `chrono`, `rmcp`, etc.

## Rules

- No code comments or doc strings in Rust files
- No `#[allow(...)]` macros
- 300 line file limit for ALL files (spec, impl, suite, Rust) — some files currently exceed this, need refactoring
- Never swallow errors (`let _ = ...`, `.ok()`, silent `unwrap_or_default()`)
- `just diagnose` must stay clean (check + clippy with -D warnings)
- `just test` to run all suite tests, `just run <file>` for single files
- Prefer established crates over custom code — check `reference/` submodules first
