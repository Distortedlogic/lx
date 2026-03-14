# Test Suite

`.lx` programs that serve as the executable contract between the spec and the implementation. Each file tests a specific language feature.

## Structure

```
suite/
  01_literals.lx        -- integers, floats, strings, bools, unit
  02_bindings.lx        -- =, :=, <-, shadowing
  03_arithmetic.lx      -- operators, precedence, bigint, float widening
  04_functions.lx       -- definitions, closures, currying, recursion, TCO
  05_pipes.lx           -- |, sections, <>, data-last threading
  06_collections.lx     -- lists, records, maps, sets, tuples, spread, slicing
  07_patterns.lx        -- ? (three modes), destructuring, guards, exhaustiveness
  08_iteration.lx       -- map/filter/fold, ranges, lazy sequences, loop/break
  09_errors.lx          -- Result/Maybe, ^, ??, require, implicit Ok
  10_shell.lx           -- $, $$, $^, ${}, interpolation, OS pipes
  11_modules/           -- multi-file import tests (main.lx, lib_math.lx, lib_types.lx)
  12_types.lx           -- annotations, structural subtyping, tagged unions
  13_concurrency.lx     -- par, sel, pmap, timeout, mutable capture
  14_stdlib/            -- per-module test files (TODO)
  15_diagnostics.lx     -- expected error output (TODO)
  16_edge_cases.lx      -- regression tests for disambiguation rules, precedence, body extent
  17_dataframes.lx      -- std/df: read, filter, group_by, agg, join (TODO, Phase 11)
  18_database.lx        -- std/db: SQLite/DuckDB CRUD, transactions (TODO, Phase 11)
  19_numerical.lx       -- std/num: vectorized ops, statistics (TODO, Phase 11)
  20_ml.lx              -- std/ml: embeddings, similarity, classify (TODO, Phase 11)
  21_plot.lx            -- std/plot: chart construction, render (TODO, Phase 11)
  22_agents.lx          -- std/agent: spawn, ask, channel, polling (TODO, Phase 12)
  23_mcp.lx             -- std/mcp: connect, list_tools, call (TODO, Phase 12)
  24_context.lx         -- std/ctx: load, save, get, set, merge (TODO, Phase 12)
  25_markdown.lx        -- std/md: parse, sections, code_blocks, render (TODO, Phase 12)
  26_cron.lx            -- std/cron: every, at, cancel (TODO, Phase 12)
```

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
| 14_stdlib | stdlib.md, stdlib-modules.md |
| 15_diagnostics | diagnostics.md |
| 16_edge_cases | design.md, grammar.md, runtime.md, errors.md |
| 17_dataframes | stdlib-data.md (std/df) |
| 18_database | stdlib-data.md (std/db) |
| 19_numerical | stdlib-data.md (std/num) |
| 20_ml | stdlib-data.md (std/ml) |
| 21_plot | stdlib-data.md (std/plot) |
| 22_agents | agents.md, stdlib-agents.md (std/agent) |
| 23_mcp | agents.md, stdlib-agents.md (std/mcp) |
| 24_context | agents.md, stdlib-agents.md (std/ctx) |
| 25_markdown | agents.md, stdlib-agents.md (std/md) |
| 26_cron | agents.md, stdlib-agents.md (std/cron) |

## When to Update

- Spec change → update or add tests that cover the changed behavior
- Implementation bug → add a regression test before fixing
- New feature → tests ship with the implementation phase
