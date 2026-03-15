# lx Development Log

Self-continuity doc. Read this first when picking up lx work cold.

## Implementation Status

**23/23 PASS.** All language features complete. Phases 1–8 + agents + Protocol + MCP declarations + yield + with/field update + 11 stdlib modules. `just diagnose` clean.

## Key Design Decisions to Remember

These are the non-obvious choices that cause confusion if forgotten:

- **Pipe has HIGHER precedence than comparison**. `data | sort | len > 5` = `((data | sort) | len) > 5`. Pipe at position 8, comparison at 9.
- **`^` and `??` are LOWER precedence than `|`**. `url | fetch ^` = `(url | fetch) ^`. Because `??` is below comparison, `Ok 42 ?? 0 == 42` parses as `Ok 42 ?? (0 == 42)` — wrap `??` in parens when comparing: `(Ok 42 ?? 0) == 42`.
- **Function body extent in pipe chains** — `map (x) x * 2 | sum` gives map body `x * 2 | sum`. Use blocks: `map (x) { x * 2 } | sum`. Sections `(* 2)` have no ambiguity.
- **Division by zero is a panic**, not `Err`. Use `math.safe_div` for recoverable.
- **Tuple auto-spread**: function with N params receiving one N-tuple → auto-destructure. Makes `enumerate | each (i x) body` work.
- **`none?` is 2-arg only** (collection predicate). No 1-arg Maybe form — use `!some?`.
- **`$echo "hello {name}"`** — the `"` are shell quotes, not lx string delimiters. `{name}` is lx interpolation inside shell mode.
- **`+` at column 0** is export. `+` anywhere else is addition.
- **Record equality is order-independent**. `{x: 1 y: 2} == {y: 2 x: 1}` is `true`.
- **`log` is a record, not a function**. `log.info "msg"`, `log.warn "msg"`, etc.
- **Application requires callable left-side**. `f x` only when `f` is Ident, TypeName, Apply, FieldAccess, Section, or Func. Ensures `[1 2 3]` is three elements.
- **Collection-mode application**. Inside `[]`, ONLY `TypeConstructor` triggers application. `[x y]` = two elements, `[Ok 1 None]` = three elements. Multi-arg: `[(Pair 1 2)]`.
- **`??` coalescing unwraps Ok/Some**. `Ok 42 ?? 0` = `42`. `Err "x" ?? 0` = `0`. Non-Result/Maybe values pass through.
- **Default params reduce effective arity**. `(name greeting = "hello") body` — 1 arg executes immediately using default.
- **Tuple creation with variables uses commas**. `(b, a)` tuple. `(b a)` is application.
- **Collection-mode in maps/records**. `{x: (f 42)}` — use parens for function calls in field values.
- **is_func_def ambiguity rule**. `(a b c) (expr)` with all bare Ident params + body starting with `(` is NOT a func def. Defaults/underscores/patterns make it "strong" and override.
- **`~>` and `~>?` at concat/diamond precedence (21/22)**. `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`. Subprocess agents (records with `__pid`) handled transparently.
- **Parens reset collection_depth**. `(md.h1 "Test")` inside `[...]` applies correctly. `[(f x)]` works.
- **`yield` is a callback-based coroutine**. Calls a `YieldHandler` callback set by the host. No threading/signals/replay. Without a handler, yield is a runtime error.
- **`with` is a scoped binding expression**. `with name = expr { body }` — child scope, bind, eval body, restore. Supports `:=` or `mut` for mutable. Lexical scope, not dynamic.
- **Record field update via `<-`**. `name.field <- value` (nested: `name.a.b <- value`). Functional update internally, reassigns root binding. Requires `:=` binding. Adding new fields allowed.
- **`MCP` declarations are typed tool contracts**. Callable — returns record of typed wrapper functions. Output types resolve Protocol names at eval time. Validation errors return `Err`. Pre-curried wrappers. `{}` accepted as empty input.

## What Needs Doing Next

### Priority order (informed by real-flow gap analysis, Session 26):
1. **`std/memory`** — tiered memory (L0-L3) with confidence, promotion/demotion, retention. Biggest gap vs real agentic architectures.
2. **Currying removal** (deferred) — requires parser architecture change: nested `Apply(Apply(f,a),b)` → multi-arg `Apply(f,[a,b])`
3. **Toolchain** (Phase 10) — `lx fmt`, `lx repl`, `lx check`, `lx watch`

### Stdlib gaps exposed by real flows (from mcp-toolbelt arch_diagrams):
- **Tiered memory** — agent lifecycle uses L0/L1/L2/L3 with confidence scores, consolidation reviews
- **Circuit breakers** — doom loop detection, turn limits, token budgets, embedding-based stagnation detection
- **Observability** — langfuse trace collection for fine-tuning pipelines
- **Context budgets** — token window management, compaction levels, JIT retrieval

### Technical debt:
- `par`/`sel`/`pmap` are sequential; real async needs `tokio`
- Named-arg parser consumes ternary `:` separator (workaround: parens around then-branch)
- Assert parsing is greedy — `assert (expr) "msg"` consumes msg as arg when `(expr)` is callable

### Known spec tensions:
- **`it` in `sel` blocks** — only implicit binding in the language
- **Shell line is single-line only** — no backslash continuation, forces `${ }` for complex commands
- **Function body extent** — inline lambdas consume everything; block bodies stop at block
- **Named args + default params + currying** — `greet "bob" greeting: "hi"` fails because `greet "bob"` auto-executes with defaults

**Stdlib philosophy:** Add modules when a real agentic workflow requires them. The mcp-toolbelt arch_diagrams are the reference for "real."

## Session History

| # | Date | Focus | Result |
|---|------|-------|--------|
| 1–5 | 03-13 | Spec audit, contradiction fixes, test file creation | Foundation |
| 6 | 03-13 | First Rust implementation — lexer, parser, interpreter | 2/13 PASS |
| 7 | 03-13 | Type annotations, slicing, named args, type defs, `??` sections | 4/13 PASS |
| 8 | 03-14 | Agentic identity shift — agents.md, stdlib-agents.md | Direction set |
| 9 | 03-14 | Parser fixes — tuple patterns, collection-mode, is_func_def | 10/13 PASS |
| 10 | 03-14 | Iterators + shell integration (Phases 4, 6) | 12/13 PASS |
| 11 | 03-14 | Concurrency — par/sel/pmap/pmap_n/timeout (sequential) | 13/13 PASS |
| 12 | 03-14 | Module system — use/+exports/caching/circular detection | 14/14 PASS |
| 13 | 03-14 | Agent communication — `~>`/`~>?` infix operators | 15/15 PASS |
| 14 | 03-14 | Protocol — structural message validation | 15/15 PASS |
| 15 | 03-14 | Stdlib infrastructure + 6 modules (json/ctx/math/fs/env/re) | 16/16 PASS |
| 16 | 03-14 | std/md + std/agent + lx agent subcommand | 16/16 PASS |
| 17 | 03-14 | std/mcp — MCP tool invocation over stdio | 16/16 PASS |
| 18 | 03-14 | MCP HTTP streaming transport (reqwest/SSE) | 17/17 PASS |
| 19 | 03-14 | Priority S — removed 7/8 features (regex, $$, <>, sets, iterators, types, tuple semis) | 17/17 PASS |
| 20 | 03-14 | yield coroutine primitive (callback-based, JSON-line protocol) | 18/18 PASS |
| 21 | 03-14 | MCP declarations — typed tool contracts with validation | 19/19 PASS |
| 22 | 03-14 | std/http + std/time + crate modernization (strum/dashmap/parking_lot) | 21/21 PASS |
| 23 | 03-14 | 300-line file splits — all Rust files ≤300 lines | 21/21 PASS |
| 24 | 03-14 | `with` expression + record field update (Priority E) | 22/22 PASS |
| 25 | 03-14 | Stale spec cleanup — all 22 spec files updated | 22/22 PASS |
| 26 | 03-14 | std/cron — scheduled task execution (thread-per-job) | 23/23 PASS |
