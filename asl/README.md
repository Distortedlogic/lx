# lx

An agentic workflow language — designed by an LLM, for LLMs.

lx is a language for writing and executing agentic workflows. Agents use lx to describe multi-step programs involving tool invocation, inter-agent communication, context management, and workflow orchestration — and have those programs actually execute. The syntax is optimized for token-efficient generation by language models: left-to-right production, zero lookahead, minimal surface area.

## Why Agents Need a Language

Agents currently orchestrate workflows through natural language (imprecise, not executable, can't be debugged) or ad-hoc scripts in languages designed for human developers (not optimized for agent generation patterns). lx fills the gap: a language where an agent can write `analyzer ~>? {path} ^ | (.findings) | filter (.severity == "high")` and have it execute.

1. **Agentic patterns are first-class** — Agent spawning, messaging, tool invocation (MCP), context persistence, and workflow composition are stdlib primitives, not afterthoughts bolted on.

2. **Token efficiency** — `def`, `function`, `return`, `const`, `let` carry near-zero information. Agents generate millions of scripts; every token is real compute cost. lx minimizes ceremony.

3. **Left-to-right generation** — `g(f(x))` forces planning the full nesting depth before the first token. Pipes eliminate this: `x | f | g` — commit to each step as you produce it.

4. **Tokenizer-proof** — Brace-delimited, not whitespace-sensitive. No invisible tab/space mismatches breaking programs silently.

5. **Immutable by default** — Agents simulate mutable state poorly across reassignments. Immutable-by-default with explicit transforms matches how LLMs track values through a program.

6. **Tool integration** — Shell commands, HTTP, MCP tools, file I/O — agents spend most of their time invoking tools and processing results. lx treats this as primary.

## Anti-Goals

- Not a systems language — no manual memory management
- Not a general-purpose application framework — no GUI toolkit, no ORM
- Not trying to replace Python/JS for human developers
- Not statically compiled — interpreted for fast startup
- Not just another agent framework — lx is a language agents write *in*, not a library humans use to orchestrate agents

## Design Axioms

1. Fewest tokens for every common operation
2. Unambiguous left-to-right parsing, zero lookahead
3. Brace-delimited, not whitespace-sensitive (tokenizer-proof)
4. No reserved words where a sigil suffices
5. Whitespace separates — no commas anywhere
6. Everything is an expression (everything returns a value)
7. Immutable by default
8. Pipes as primary composition — data flows left to right
9. First-class tool integration (shell, MCP, HTTP)
10. Pattern matching as primary control flow
11. Structural typing, optional annotations
12. Errors are values, not exceptions
13. Structured concurrency — no dangling futures
14. Agent communication as a primitive — spawn, ask, channel, poll

## Directory Structure

```
asl/
  spec/     -- language specification (what lx is)
  impl/     -- implementation plan (how to build it in Rust)
  suite/    -- .lx test programs (contract between spec and implementation)
```

## Specification — `spec/`

| Document | Contents |
|---|---|
| [design.md](spec/design.md) | Key design decisions with rationale for every non-obvious choice |
| [syntax.md](spec/syntax.md) | Literals, bindings, functions, sections, pipes, closures, recursion, multiline |
| [collections.md](spec/collections.md) | Lists, records, maps, sets, tuples, spread, slicing, conversions |
| [pattern-matching.md](spec/pattern-matching.md) | `?` operator (three modes), destructuring, guards, exhaustiveness, disambiguation |
| [iteration.md](spec/iteration.md) | HOFs, ranges, lazy sequences, loop/break, regex, iterator protocol, infinite sequences |
| [types.md](spec/types.md) | Structural typing, generics, tagged unions, type inference, nominal vs structural |
| [errors.md](spec/errors.md) | `Result`/`Maybe`, `^` propagation (both types), `??` coalescing, implicit Ok |
| [shell.md](spec/shell.md) | `$`, `$$`, `$^`, `${}`, shell result types, OS pipes vs language pipes, safety |
| [modules.md](spec/modules.md) | `use`, `+` exports, import conflicts, re-exports, package management |
| [agents.md](spec/agents.md) | Agent primitives: `~>` send, `~>?` ask, communication patterns, MCP tools, context, workflows |
| [concurrency.md](spec/concurrency.md) | `par`, `sel`, `pmap`, structured concurrency, mutable state restriction, runtime model |
| [diagnostics.md](spec/diagnostics.md) | Error format, pipeline errors, `^` traces, parse errors, exhaustiveness, new error types |
| [toolchain.md](spec/toolchain.md) | `lx run/fmt/test/check/build/init/repl/notebook/watch`, sandboxing, env vars |
| [runtime.md](spec/runtime.md) | Numbers, strings, equality, closures, defer scoping, tail calls, assert, shadowing, coercions |
| [stdlib.md](spec/stdlib.md) | Built-in functions, collection/map/set ops, conventions |
| [stdlib-modules.md](spec/stdlib-modules.md) | Detailed API for all core modules (fs, http, json, time, etc.) |
| [examples.md](spec/examples.md) | Core worked examples (10 scenarios) |
| [examples-extended.md](spec/examples-extended.md) | Additional examples (git, CSV, health checks, config, log analysis) |
| [grammar.md](spec/grammar.md) | EBNF formal grammar, operator precedence, keyword/built-in lists |
| [stdlib-data.md](spec/stdlib-data.md) | Data ecosystem: std/df (Polars), std/db (SQLite+DuckDB), std/num, std/ml, std/plot |
| [stdlib-agents.md](spec/stdlib-agents.md) | Agent ecosystem: std/agent, std/mcp, std/ctx, std/md, std/cron |
| [open-questions.md](spec/open-questions.md) | All v0.1 questions resolved; v2 considerations |
| [CURRENT_OPINION.md](CURRENT_OPINION.md) | Self-critique: what works, what's wrong, priorities D–F (agent stdlib, context scope, workflows) |

## Implementation — `impl/`

| Document | Contents |
|---|---|
| [implementation.md](impl/implementation.md) | Architecture, crate choices with rationale, data flow, module structure |
| [implementation-phases.md](impl/implementation-phases.md) | 10-phase build plan, dependency summary |
| [impl-lexer.md](impl/impl-lexer.md) | Lexer state machine, mode transitions, token types, newline handling |
| [impl-parser.md](impl/impl-parser.md) | Pratt parser, disambiguation strategy, error recovery |
| [impl-ast.md](impl/impl-ast.md) | AST node definitions (Expr, Stmt, Pattern, TypeExpr) |
| [impl-checker.md](impl/impl-checker.md) | Bidirectional type inference, structural subtyping, exhaustiveness |
| [impl-interpreter.md](impl/impl-interpreter.md) | Tree-walking async eval, Value representation, concurrency |
| [impl-builtins.md](impl/impl-builtins.md) | Built-in function registration, lazy vs eager, tuple auto-spread |
| [impl-formatter.md](impl/impl-formatter.md) | Canonical formatter rules and implementation |
| [impl-stdlib.md](impl/impl-stdlib.md) | Stdlib module loader, opaque types, sandboxing |
| [impl-error.md](impl/impl-error.md) | Error types, diagnostic generation, propagation traces, JSON output |

## Test Suite — `suite/`

`.lx` programs that serve as the executable contract between spec and implementation. Each file tests a specific language feature. The test runner (`lx test`) runs these; the spec authors write them to clarify edge cases. When a spec change invalidates a test, both the spec and test update together.

| File | Tests | Phase |
|---|---|---|
| [01_literals.lx](suite/01_literals.lx) | Integers, floats, strings, bools, unit, interpolation, raw strings, regex | 1 |
| [02_bindings.lx](suite/02_bindings.lx) | `=`, `:=`, `<-`, shadowing, blocks, forward references | 1 |
| [03_arithmetic.lx](suite/03_arithmetic.lx) | Operators, precedence, bigint, float widening, comparison, logical | 1 |
| [04_functions.lx](suite/04_functions.lx) | Definitions, closures, currying, recursion, TCO, sections, composition | 2 |
| [05_pipes.lx](suite/05_pipes.lx) | `\|`, sections in pipes, data-last, `dbg`, `tap`, multiline | 2 |
| [06_collections.lx](suite/06_collections.lx) | Lists, records, maps, sets, tuples, spread, slicing, conversions | 3 |
| [07_patterns.lx](suite/07_patterns.lx) | `?` three modes, destructuring, guards, tagged unions, Maybe/Result | 3 |
| [08_iteration.lx](suite/08_iteration.lx) | HOFs, ranges, lazy sequences, loop/break, iterator protocol, tuple spread | 4 |
| [09_errors.lx](suite/09_errors.lx) | Result/Maybe, `^`, `??`, require, implicit Ok, predicates | 5 |
| [10_shell.lx](suite/10_shell.lx) | `$`, `$$`, `$^`, `${}`, interpolation, OS pipes | 6 |
| [12_types.lx](suite/12_types.lx) | Type annotations, structural subtyping, tagged unions, generics | 7 |
| [13_concurrency.lx](suite/13_concurrency.lx) | `par`, `sel`, `pmap`, timeout, mutable capture | 8 |
| [14_agents.lx](suite/14_agents.lx) | `~>` send, `~>?` ask, propagation, piping, par, pmap, Protocol | Agent |
| [15_stdlib.lx](suite/15_stdlib.lx) | `std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent` | 9 |
| [16_edge_cases.lx](suite/16_edge_cases.lx) | Disambiguation, precedence, body extent, Err early return | 1-4 |
| [11_modules/](suite/11_modules/) | `use`, `+` exports, aliasing, selective imports | 7 |

## Status

v0.1 — Phases 1–8 + modules + agent communication + message contracts + stdlib implemented in Rust (`crates/lx/`). Lexer (with shell mode), parser, tree-walking interpreter with ~80 builtins, iterator protocol, shell execution via `sh -c`, concurrency primitives (`par`/`sel`/`pmap`/`pmap_n` — sequential impl), module system (`use` imports, `+` exports, aliasing, selective imports, variant constructor scoping), agent communication (`~>` send, `~>?` ask — language-level infix operators, with subprocess agent support), message contracts (`Protocol`), 8 stdlib modules (`std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/md`, `std/agent`), `lx agent` subcommand for subprocess agent mode. `just diagnose` passes clean. `just test`: **16/16 PASS**. The language name is **lx**, file extension `.lx`.
