# Implementation Phases

Each phase produces a working, testable increment. No phase depends on a later phase. Each phase ends with `just test` passing.

## Phase 1: Lexer + Literal Expressions

**Goal:** Lex and parse literal expressions, bindings, and arithmetic. Run `lx run` on trivial scripts.

**Deliverables:**
- `crates/lx/` with Cargo.toml (deps: `miette`, `num-bigint`, `num-traits`, `thiserror`)
- `crates/lx-cli/` with Cargo.toml (deps: `lx`, `tokio`, `clap` or just arg parsing)
- Lexer: integers, floats, strings (with interpolation), booleans, unit, operators, identifiers, types, comments (`--`), newlines, `;`
- Parser: literals, binary ops (+, -, *, /, %, //), unary (-, !), grouping `(expr)`, bindings (`=`, `:=`, `<-`), blocks `{ stmts }`
- Interpreter: evaluate arithmetic, bindings, print last expression
- `lx run file.lx` works for: `x = 5; y = x + 3; y * 2` → prints `16`
- Diagnostics: parse errors with source spans via miette

**Test cases:** arithmetic, precedence, integer overflow (bigint), float widening, division by zero panics, mutable binding + reassignment.

## Phase 2: Functions, Pipes, Sections

**Goal:** First-class functions, pipe operator, sections, auto-currying.

**Deliverables:**
- Lexer: `|`, `->`, function params `(x y)`
- Parser: function definitions `name = (params) body`, application by juxtaposition, pipe `|`, sections `(* 2)` `(.field)`
- Interpreter: closures (Env capture), function application, pipe threading (data-last), sections as anonymous functions, currying for all-positional functions
- `[1 2 3] | map (* 2) | sum` works

**Test cases:** closures capture scope, currying, pipe left-to-right, section for each operator, data-last threading.

## Phase 3: Collections + Pattern Matching

**Goal:** Lists, records, maps, tuples. The `?` operator in all three modes.

**Deliverables:**
- Lexer: `[`, `]`, `{`, `}`, `%{`, `..`, `..=`, `_`
- Parser: list/record/map/tuple literals, spread `..`, field access `.`, slicing, destructuring patterns, `?` (multi-arm, ternary, single-arm), guards `&`
- Interpreter: collection values, structural equality, `get`/`contains?`/`len`/`empty?`, pattern matching with destructuring
- Value: implement `PartialEq` for structural equality

**Test cases:** each collection type, spread merge, slicing, nested destructuring, guard conditions, no truthiness (non-Bool in ternary is type error).

## Phase 4: Iteration + Lazy Sequences

**Goal:** `map`, `filter`, `fold`, ranges, `loop`/`break`.

**Deliverables:**
- Built-in HOFs: `map`, `filter`, `fold`, `flat_map`, `each`, `sort`, `sort_by`, `rev`, `take`, `drop`, `zip`, `enumerate`, `partition`, `group_by`, `chunks`, `windows`, `find`, `any?`, `all?`, `count`, `sum`, `product`, `uniq`, `flatten`, `intersperse`, `scan`, `take_while`, `drop_while`, `min`, `max`, `min_by`, `max_by`
- Ranges: `1..10`, `1..=10` (eager — produce lists)
- `loop`/`break` with optional value
- `nat`, `cycle` built-ins

**Test cases:** each HOF, range materialization, loop with break value.

## Phase 5: Error Handling

**Goal:** `Result`/`Maybe`, `^` propagation, `??` coalescing, implicit Ok wrapping.

**Deliverables:**
- `Ok`, `Err`, `Some`, `None` as tagged union constructors
- `^` postfix: unwrap Ok/Some, propagate Err/None-as-Err
- `??` binary: coalesce Err/None to default
- `require` built-in: Maybe → Result
- Implicit Ok wrapping on final expression of Result-returning functions
- Propagation trace: each `^` site recorded for diagnostics
- `assert` keyword: panic on false, test runner catches

**Test cases:** `^` on Result, `^` on Maybe, `??` on both, propagation chain, pipeline error patterns (`map (x) f x ^`), implicit Ok, assert panics.

## Phase 6: Shell Integration

**Goal:** `$`, `$^`, `${ }` — the core scripting use case.

**Deliverables:**
- Lexer: shell mode after `$`/`$^`/`${`, `{expr}` interpolation re-entry, shell mode until newline (or `}` for blocks)
- Parser: shell expressions as AST nodes with interpolation holes
- Interpreter: execute via `std::process::Command` through `/bin/sh -c`, capture stdout/stderr/exit code
- `$cmd` returns `Result {out err code} ShellErr`
- `$^cmd` returns `Str ^ ShellErr` (extract stdout on exit 0)
- `${ }` — multi-line block
- OS pipe vs language pipe disambiguation (parens to exit shell mode)

**Test cases:** simple commands, interpolation, `$^` with pipe to `trim`, exit code handling, multi-line block, spawn failure returns Err.

## Phase 7: Modules + Type Checker

**Goal:** `use` imports, `+` exports, structural type checking.

**Status:** DONE.

**Implemented:**
- Module system: file = module, `use ./...`, `use ../...`, `use std/...`, aliasing `: name`, selective `{name1 name2}`
- Export: `+` prefix at column 0 (both lowercase and uppercase bindings/types)
- Circular import detection, module caching
- Variant constructor scoping (tagged union constructors imported as bare names)
- Bidirectional type checker: annotation propagation, type synthesis, unification
- `lx check` subcommand
- Type annotations on params, return types, bindings: `(x: Int y: Str) -> Result Int Str { ... }`
- `^` in type signatures: `-> Str ^ IoErr`

**Test cases:** import resolution, circular import error, type annotations, `lx check` validation.

## Phase 8: Concurrency

**Goal:** `par`, `sel`, `pmap` with structured concurrency.

**Status:** Syntax implemented, execution is **sequential**. Real async (tokio) planned.

**Implemented:**
- `par { stmts }` — evaluates sequentially, collects into tuple
- `sel { expr -> handler }` — evaluates first arm only
- `pmap f xs` / `pmap_n limit f xs` — sequential map
- `timeout n` — sequential (just evaluates)

**Not yet implemented:** actual concurrent execution, cancellation, mutable capture restriction.

## Phase 9: Standard Library

**Goal:** Core stdlib modules.

**Status:** DONE — 12 modules implemented.

**Implemented:**
- `std/json` — serde_json (parse, encode, encode_pretty)
- `std/ctx` — immutable key-value context (load, save, get, set, merge)
- `std/math` — numeric functions (abs, sqrt, pow, log, trig, clamp, safe_div)
- `std/fs` — filesystem (read, write, exists, mkdir, rm, copy, move, glob, walk, read_lines)
- `std/env` — environment (args, get, set, cwd)
- `std/re` — regex (is_match, match, find_all, replace, replace_all, split)
- `std/md` — markdown parsing (parse, extract sections/code/links, build, render)
- `std/agent` — agent spawning/messaging (spawn, channel, list, stop)
- `std/mcp` — MCP client (connect, list_tools, call, stdio/HTTP transports)
- `std/http` — HTTP client (get, post, put, delete)
- `std/time` — time (now, format, parse, sleep, sec, ms, min)
- `std/cron` — scheduled execution (every, at, cancel)

## Phase 10: Toolchain Polish

**Status:** PARTIALLY DONE.

**Implemented:**
- `lx test` — run tests/*.lx, collect assert failures, report counts
- `lx check` — type checker subcommand
- `lx agent` — agent mode (long-lived process for cron/channels)

**Not yet implemented:**
- `lx fmt` — see [impl-formatter.md](impl-formatter.md) for design
- `lx repl` — interactive loop
- `lx watch` — file watcher

## Phase 11: `emit` Primitive

**Goal:** Dedicated agent-to-human output primitive replacing `$echo` for user-facing output.

**Status:** PLANNED.

**Deliverables:**
- Lexer: `emit` keyword → `TokenKind::Emit`
- Parser: `emit expr` as prefix expression (same pattern as `yield`)
- AST: `Emit { value: Box<SExpr> }` variant
- Interpreter: `EmitHandler` callback (non-blocking, unlike `YieldHandler`). Default: `println!` for strings, JSON for structured values. Returns `()`.
- Subprocess protocol: emits `{"type":"emit","value":...}` JSON-line to stdout
- Works with `Protocol` validation: `emit StatusUpdate {type: "status" msg: "done"}`

**Test cases:** emit string, emit record, emit with Protocol, emit in loop, emit default (no handler), emit in subprocess agent.

## Phase 12: Dialogue, Interceptors, Handoff

**Goal:** Multi-turn agent dialogue, message middleware, structured handoff — extensions to `std/agent`.

**Status:** PLANNED.

**Deliverables:**
- `agent.dialogue` / `agent.dialogue_turn` / `agent.dialogue_history` / `agent.dialogue_end` — session management with accumulated history
- `agent.intercept` — middleware wrapping for `~>` and `~>?`, composable by nesting
- `agent.handoff` / `agent.as_context` — structured context transfer between agents
- `Handoff` Protocol — standard shape for context transfer records
- JSON-line protocol extension: `dialogue_turn` / `dialogue_response` message types
- All functions added to `stdlib/agent.rs`

**Test cases:** dialogue open/turn/end, dialogue history accumulation, interceptor chain ordering, interceptor short-circuit, handoff with Protocol validation, as_context rendering.

## Phase 13: Plan Revision, Introspection, Knowledge

**Goal:** Dynamic plan execution, agent self-awareness, shared discovery cache — three new stdlib modules.

**Status:** PLANNED.

**Deliverables:**
- `std/plan` — `plan.run`, `plan.replan`, `plan.continue`, `plan.abort`, `plan.skip`, `plan.insert_after`. Topological step ordering, `on_step` callback, `PlanAction` tagged union.
- `std/introspect` — `introspect.self`, `budget`, `elapsed`, `actions`, `is_stuck`, `strategy_shift`, `mark`. Interpreter-level action logging (bounded buffer).
- `std/knowledge` — `knowledge.create`, `store`, `get`, `query`, `keys`, `remove`, `merge`, `expire`. File-backed JSON with provenance metadata and file-level locking.

**Test cases:** plan execution with continue, plan replan mid-execution, plan insert_after, plan abort. Introspect actions list, budget tracking, is_stuck detection, strategy_shift reset. Knowledge create/store/get/query, provenance metadata, expire.

## Future Phases

For the stdlib buildout roadmap (std/ai, std/tasks, std/audit, standard agents, etc.), see [stdlib_roadmap.md](stdlib_roadmap.md) and `NEXT_PROMPT.md`.
