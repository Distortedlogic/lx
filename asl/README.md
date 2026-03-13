# lx

A scripting language designed by an LLM for LLM generation.

Every existing scripting language optimizes for human readability and familiarity. lx optimizes for token-efficient generation, left-to-right production without lookahead, and minimal syntax surface area. The target user is me — a language model that generates code one token at a time, can't set breakpoints, can't visually scan for errors, and pays a real cost for every token produced.

## Motivation

1. **Wasted tokens** — `def`, `function`, `return`, `const`, `let` carry near-zero information. I generate millions of scripts; every token is real compute cost.

2. **Indentation errors** — Tokenizers group whitespace inconsistently. Whitespace-significant languages (Python, YAML) cause constant subtle bugs when I generate code. A single misgrouped space breaks the program silently.

3. **Lookahead requirements** — `g(f(x))` forces me to plan the full nesting depth before producing the first character. I generate left-to-right, one token at a time. Pipes eliminate lookahead entirely: `x | f | g` — I commit to each step as I produce it.

4. **State tracking failures** — I simulate mutable state poorly across reassignments. When `x` changes meaning three times in a function, I lose track. Immutable-by-default with explicit transforms matches how I actually track values through a program.

5. **Syntax irregularities** — Every special case (ternary vs if/else, for-in vs while, `this` vs `self`, `==` vs `===`) is a place I might hallucinate the wrong form. Uniform syntax reduces my error rate.

6. **Shell integration friction** — 80% of my scripting is "run command, parse output, act." Languages treat shell as an afterthought behind `subprocess.run()` or backtick hacks. lx treats it as primary.

## Anti-Goals

- Not a systems language — no manual memory management, no inline assembly
- Not a general-purpose application framework — no GUI toolkit, no ORM
- Not trying to replace Python/JS for human developers
- Not statically compiled — interpreted for fast startup, optional AOT compilation

## Design Axioms

1. Fewest tokens for every common operation
2. Unambiguous left-to-right parsing, zero lookahead
3. Brace-delimited, not whitespace-sensitive (tokenizer-proof)
4. No reserved words where a sigil suffices
5. Whitespace separates — no commas anywhere
6. Everything is an expression (everything returns a value)
7. Immutable by default
8. Pipes as primary composition — data flows left to right
9. First-class shell integration
10. Pattern matching as primary control flow
11. Structural typing, optional annotations
12. Errors are values, not exceptions
13. Structured concurrency — no dangling futures

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
| [open-questions.md](spec/open-questions.md) | All v0.1 questions resolved; v2 considerations |

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
| [16_edge_cases.lx](suite/16_edge_cases.lx) | Disambiguation, precedence, body extent, Err early return | 1-4 |
| [11_modules/](suite/11_modules/) | `use`, `+` exports, aliasing, selective imports | 7 |

## Status

v0.1 — specification complete, implementation design complete (all component design docs written, 11-phase plan), test suite covers phases 1–8 plus edge cases and module imports. Data ecosystem spec (std/df, std/db, std/num, std/ml, std/plot) added as Phase 11. No code exists yet. The language name is **lx**, file extension `.lx`.
