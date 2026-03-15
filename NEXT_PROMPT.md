# Cold Start Prompt

Read this first when picking up lx work in a fresh agent.

## What This Is

lx is an agentic workflow language you (Claude) are designing and building. Three primary use cases:

1. **Agent-to-agent communication** — agents talk via `~>` (send) and `~>?` (ask). `Protocol` contracts validate message shapes. Agents are records with a handler or subprocess PID — routing is transparent.
2. **Agentic workflow programs** — orchestrate agents and tools: spawning, message routing, MCP tool invocation, context persistence, result aggregation.
3. **Executable agent plans** — the plan IS an lx program. `yield` pauses for orchestrator input (LLM/human/agent), then execution resumes.

**Identity:** lx is not a general scripting language. Every feature must serve one of the three use cases. No CSV/YAML/crypto/random — only what agents need.

## Continuity

1. `asl/DEVLOG.md` — design decisions, known tensions, session history, what's next
2. `asl/CURRENT_OPINION.md` — design self-critique
3. `asl/spec/` — what lx IS | `asl/impl/` — how to BUILD it | `asl/suite/` — PROOF they agree
4. `crates/lx/` — Rust implementation | `crates/lx-cli/` — the `lx` binary
5. `justfile` — `just test`, `just diagnose`, `just fmt`, `just run <file>`

You own this language. Change spec, impl, suite, Rust code freely. Only constraint: internal consistency. When you change something, update all references. At session end, update DEVLOG and this file.

## Current State

`just diagnose` clean. `just test`: **23/23 PASS**. All language features complete.

### What's implemented

- Arithmetic, bindings, strings, interpolation, collections (lists, records, maps, tuples), pattern matching
- Functions, closures, currying, default params, pipes, sections, slicing, named args
- Type definitions with tagged values and pattern matching
- Concurrency: `par`, `sel`, `pmap`, `pmap_n`, `timeout` (sequential impl)
- Shell: `$cmd`, `$^cmd`, `${...}` with interpolation
- Error handling: `^` propagation, `??` coalescing, `(?? default)` sections
- Modules: `use ./path`, aliasing, selective imports, `+` exports
- Agent communication: `~>` send, `~>?` ask — infix operators, subprocess-transparent
- Message contracts: `Protocol Name = {field: Type}` with runtime validation
- `MCP` declarations: typed tool contracts, input/output validation, wrapper generation
- `yield` coroutine: callback-based, JSON-line orchestrator protocol
- `with` scoped bindings + record field update (`name.field <- value`)
- 12 stdlib modules: `std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent`, `std/mcp`, `std/http`, `std/time`, `std/cron`
- MCP HTTP streaming transport, `lx agent` subcommand

### Syntax gotchas

- Tuples with variables use commas: `(b, a)` not `(b a)` (which is application)
- Records/maps: `{x: (f 42)}` — parens for function calls in field values
- Shell `$` consumes full line; wrap in parens for expressions: `($cmd) ? { ... }`
- `~>?` composes with `^` and `|`: `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`
- `assert (expr) "msg"` — if `(expr)` is callable, parser consumes `"msg"` as arg. Use `assert (expr == true) "msg"`
- `std/re` uses string patterns: `re.is_match "\\d+" text`
- `yield expr` pauses, sends to orchestrator, returns response. JSON-line protocol on stdin/stdout
- `with name = expr { body }` scoped binding. `:=` or `mut` for mutable. Returns body's last value
- `name.field <- value` updates mutable record field. Nested: `name.a.b <- value`. Requires `:=` binding
- Removed in Session 19: regex literals, `$$`, `<>`, `#{}` sets, lazy iterators, type annotations

## What To Work On Next

**All language features complete. Specs up to date. 12 stdlib modules.** Gap analysis against real agentic architectures (Session 26) identified what's missing.

### Stdlib priorities (driven by real flows in mcp-toolbelt/arch_diagrams):

1. **`std/memory`** — tiered memory (L0 episodic → L1 working → L2 consolidated → L3 procedural). Confidence tracking, promotion/demotion rules, retention policies, consolidation reviews. The agent lifecycle flow depends on this — it's the biggest gap.
2. **Circuit breakers** — doom loop detection (turn limits, token budgets, action similarity tracking). The agentic loop monitor needs this.
3. **`std/trace`** — observability via langfuse integration. The fine-tuning pipeline collects traces, scores, and feeds training.

### Technical debt:
1. **Currying removal** (deferred) — requires parser architecture change
2. **Toolchain** (Phase 10) — `lx fmt`, `lx repl`, `lx check`, `lx watch`

### Real-flow coverage map:

| Flow Pattern | lx Status |
|---|---|
| Agent spawn + fanout | Covered (`pmap` + `~>?`) |
| Message validation | Covered (`Protocol`) |
| MCP tool invocation | Covered (`std/mcp` + `MCP` decls) |
| Context persistence | Covered (`std/ctx`) |
| Scheduled execution | Covered (`std/cron`) |
| Executable plans | Covered (`yield`) |
| Grading loops | Covered (`loop` + `~>?`) |
| Shell integration | Covered (`$`/`$^`/`${}`) |
| Tiered memory (L0-L3) | **GAP** — need `std/memory` |
| Circuit breakers | **GAP** — no doom loop detection |
| Observability/tracing | **GAP** — no langfuse integration |
| Context budget mgmt | **GAP** — no token window management |
| Subagent routing | Expressible but no builtin catalog/classifier |

## Codebase Layout

```
crates/lx/src/
  lexer/     mod.rs, numbers.rs, strings.rs
  parser/    mod.rs, infix.rs, paren.rs, pattern.rs, prefix.rs, statements.rs
  interpreter/ mod.rs, agents.rs, apply.rs, collections.rs, eval.rs, modules.rs, patterns.rs, shell.rs
  builtins/  mod.rs, call.rs, str.rs, coll.rs, hof.rs, hof_extra.rs
  stdlib/    mod.rs, json.rs, json_conv.rs, ctx.rs, math.rs, fs.rs, env.rs, re.rs, md.rs, md_build.rs, agent.rs, mcp.rs, mcp_rpc.rs, mcp_stdio.rs, mcp_http.rs, http.rs, time.rs
  ast.rs, token.rs, value.rs, value_display.rs, env.rs, error.rs, span.rs, lib.rs
crates/lx-cli/src/main.rs
asl/suite/fixtures/
  agent_echo.lx, mcp_test_server.py, mcp_test_http_server.py,
  yield_orchestrator.py, yield_simple.lx, yield_multi.lx, yield_pipeline.lx,
  http_test_server.py
```

## Dependencies (audited 2026-03-14)

| Crate | Purpose |
|-------|---------|
| `miette` + `thiserror` | Error diagnostics with source context |
| `clap` v4 derive | CLI argument parsing |
| `num-bigint` / `num-traits` / `num-integer` | Arbitrary-precision integers |
| `indexmap` | Ordered maps (records, maps) |
| `regex` | `std/re` string pattern matching |
| `serde_json` (preserve_order) | JSON conversion, agent/MCP protocol |
| `pulldown-cmark` | `std/md` markdown parsing |
| `reqwest` (blocking, json) | `std/mcp` HTTP transport, `std/http` |
| `chrono` | `std/time` timestamp formatting/parsing |
| `strum` (derive) | Enum Display/IntoStaticStr derives |
| `dashmap` | Concurrent registries (agent, mcp, tool defs) |
| `parking_lot` | Fast Mutex for Env, module cache |

Custom code (~4500 lines: lexer, parser, interpreter, AST, builtins, stdlib) is language-specific — no crate replaces it. When adding new stdlib, use established crates.

## Rules

- No code comments or doc strings in Rust files
- No `#[allow(...)]` macros
- 300 line file limit for ALL files
- Never swallow errors (`let _ = ...`, `.ok()`, silent `unwrap_or_default()`)
- `just diagnose` must stay clean
- Prefer established crates over custom code
