# Test Suite

`.lx` programs that serve as the executable contract between the spec and the implementation. Each file tests a specific language feature.

## Structure

```
suite/
  01_literals.lx        -- integers, floats, strings, bools, unit, interpolation, raw strings, regex
  02_bindings.lx        -- =, :=, <-, shadowing, blocks, forward references
  03_arithmetic.lx      -- operators, precedence, bigint, float widening, comparison, logical
  04_functions.lx       -- definitions, closures, currying, recursion, TCO, sections, composition
  05_pipes.lx           -- |, sections, <>, data-last threading, dbg, tap, multiline
  06_collections.lx     -- lists, records, maps, sets, tuples, spread, slicing, conversions
  07_patterns.lx        -- ? (three modes), destructuring, guards, tagged unions, Maybe/Result
  08_iteration.lx       -- map/filter/fold, ranges, lazy sequences, loop/break, iterator protocol
  09_errors.lx          -- Result/Maybe, ^, ??, require, implicit Ok, predicates
  10_shell.lx           -- $, $$, $^, ${}, interpolation, OS pipes
  11_modules/           -- multi-file import tests (main.lx, lib_math.lx, lib_types.lx)
  12_types.lx           -- annotations, structural subtyping, tagged unions, generics
  13_concurrency.lx     -- par, sel, pmap, timeout, mutable capture
  14_agents.lx          -- ~> send, ~>? ask, propagation, piping, par, pmap, Protocol
  15_stdlib.lx          -- std/json, std/math, std/ctx, std/fs, std/env, std/re, std/md, std/mcp, std/agent
  16_edge_cases.lx      -- regression tests for disambiguation rules, precedence, body extent
  17_mcp_http.lx        -- MCP HTTP streaming transport tests
  fixtures/
    agent_echo.lx             -- echo agent handler for std/agent tests
    mcp_test_server.py        -- minimal MCP stdio server for std/mcp tests
    mcp_test_http_server.py   -- minimal MCP HTTP server for HTTP transport tests
```

**17/17 PASS** — all tests passing.

## Convention

Each test file uses `assert` statements. A passing suite means every assert passes:

```
assert (42 == 42)
assert (3.14 == 3.14)
assert ("a {1 + 2} b" == "a 3 b")
```

Each file begins with a comment header noting:
- Which spec files it tests
- Which implementation phase it covers

## Spec Cross-References

| Suite File | Spec Files |
|---|---|
| 01_literals | syntax.md (Literals, Lexical Grammar) |
| 02_bindings | syntax.md (Bindings), runtime.md (Shadowing, Forward References) |
| 03_arithmetic | syntax.md (operators), runtime.md (Numbers, Equality, Coercions) |
| 04_functions | syntax.md (Functions, Closures, Recursion, Sections, Composition) |
| 05_pipes | syntax.md (Pipes, Sections, Multiline) |
| 06_collections | collections.md |
| 07_patterns | pattern-matching.md |
| 08_iteration | iteration.md |
| 09_errors | errors.md |
| 10_shell | shell.md |
| 11_modules | modules.md |
| 12_types | types.md |
| 13_concurrency | concurrency.md |
| 14_agents | agents.md |
| 15_stdlib | stdlib.md, stdlib-modules.md, stdlib-agents.md |
| 16_edge_cases | design.md, grammar.md, runtime.md, errors.md |
| 17_mcp_http | agents.md, stdlib-agents.md |

## Planned Test Files (not yet implemented)

| File | Module | Phase |
|---|---|---|
| 18_dataframes.lx | std/df | Phase 11 (Data Ecosystem) |
| 19_database.lx | std/db | Phase 11 |
| 20_numerical.lx | std/num | Phase 11 |
| 21_ml.lx | std/ml | Phase 11 |
| 22_plot.lx | std/plot | Phase 11 |

## When to Update

- Spec change → update or add tests that cover the changed behavior
- Implementation bug → add a regression test before fixing
- New feature → tests ship with the implementation phase
