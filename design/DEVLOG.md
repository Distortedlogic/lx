# lx Development Log

Session history + design decisions. For priorities and gap analysis, see `NEXT_PROMPT.md`. For self-critique, see `CURRENT_OPINION.md`.

## Implementation Status

**44/44 PASS.** 29 stdlib modules (12 base + 8 orchestration/intelligence + 6 standard agents + 2 infrastructure + 1 visualization). 14 flow programs. Type checker. Regex literals. `lx diagram` CLI. `refine` expression. `agent.reconcile` (6 strategies). `just diagnose` clean.

## Key Design Decisions

Non-obvious choices that cause confusion if forgotten:

- **Pipe HIGHER than comparison**. `data | sort | len > 5` = `((data | sort) | len) > 5`.
- **`^` and `??` LOWER than `|`**. `url | fetch ^` = `(url | fetch) ^`.
- **Function body extent** — `map (x) x * 2 | sum` gives map body `x * 2 | sum`. Use blocks or sections.
- **Division by zero is a panic**. Use `math.safe_div` for recoverable.
- **Tuple auto-spread**: N-param function + single N-tuple → auto-destructure.
- **`none?` is 2-arg only** (collection predicate). Use `!some?` for Maybe.
- **`$echo "hello {name}"`** — `"` are shell quotes, `{name}` is lx interpolation.
- **`+` at column 0** is export. Anywhere else is addition.
- **Record equality is order-independent**.
- **`log` is a record** with `.info`, `.warn`, `.err`, `.debug` fields.
- **Application requires callable left-side**. `[1 2 3]` is three elements, not application.
- **Collection-mode**: inside `[]`, only TypeConstructor triggers application.
- **`??` unwraps Ok/Some**. Non-Result/Maybe values pass through.
- **Default params reduce effective arity**. 1 required arg + defaults → executes immediately.
- **Tuple variables need commas**. `(b, a)` tuple. `(b a)` is application.
- **`{x: (f 42)}`** — parens for function calls in record field values.
- **is_func_def ambiguity**: `(a b c) (expr)` with all bare Ident params NOT a func def. Defaults/underscores/type annotations override.
- **`~>`/`~>?` at concat precedence**. `agent ~>? msg ^ | process` = `((agent ~>? msg) ^) | process`.
- **`yield` is callback-based**. No handler → runtime error. Will be subsumed by `RuntimeCtx.yield_` backend.
- **`RuntimeCtx` for backend pluggability**. All I/O builtins receive `&RuntimeCtx` parameter. Traits: `AiBackend`, `EmitBackend`, `HttpBackend`, `ShellBackend`, `YieldBackend`, `LogBackend`. Standard defaults: Claude Code CLI for AI, reqwest for HTTP, `std::process::Command` for shell, stdout for emit, stdin/stdout JSON-lines for yield, stderr for logging. These are the production defaults, not placeholders. Embedders swap them for testing/server/sandbox use cases. Spec: `spec/runtime-backends.md`.
- **`with` is scoped binding**. Lexical scope, not dynamic. Supports `:=` mutable.
- **Record field update via `<-`**. Requires `:=` binding. Adding new fields allowed.
- **`MCP` declarations are typed tool contracts**. Callable → record of wrapper functions.
- **Type annotations don't consume lowercase idents as type args**. `(x: Maybe a)` treats `a` as next param, not type var. Write `(x: (Maybe a))`. Avoids body-start ambiguity.
- **`{` after type tokens is body, not record type**. `-> Int { body }` — `{` starts body. Record return types need parens: `-> ({x: Int})`.
- **`lx check` is optional**. `lx run` ignores annotations. Checker uses bidirectional inference + unification.
- **`std/plan` treats plans as data**. `plan.run` with `on_step` callback. `PlanAction` tagged union controls flow.
- **`std/introspect` is separate from `std/agent`**. Cross-cutting runtime metadata. Bounded action log (1000 entries).
- **`std/knowledge` is file-backed JSON**. Shared via path. Provenance metadata. File-level locking.
- **`std/diag` uses existing lexer+parser**. Walks AST, does not execute. Graph IR is plain lx records.

## Design Decisions (specified but not yet implemented)

- **`~>>?` at concat precedence**. Returns lazy stream, not single value.
- **`checkpoint`/`rollback` scoping**. Shell/MCP not rolled back.
- **Capability attenuation is runtime-enforced**. `CapabilityDenied` error on violation.
- **`std/blackboard` is last-write-wins**. No CRDTs.
- **`std/events` handlers are synchronous**. Invoked in subscription order.
- **`agent.dialogue` is library, not keyword**. Session-based multi-turn.
- **`agent.intercept` returns new wrapped agent**. Middleware takes `(msg next)`.
- **`agent.handoff` is explicit, not auto-populated**. `as_context` renders for LLM.
- **`|>>` at same precedence as `|`**. Lazy until `collect`/`each`.
- **`with context` extends `with`**. `context` keyword = ambient scope.
- **`caller` is handler-scoped implicit binding**. Only in agent handler context.
- **`_priority` is stripped before handler delivery**. 4 levels. No preemption.
- **`agent.gate` is a library function**. Three runtime modes matching `yield`/`emit`.
- **`agent.supervise` strategies**: `:one_for_one`/`:one_for_all`/`:rest_for_one`.
- **`std/saga` compensations run in reverse**. Undo failures recorded, not fatal.

## Technical Debt

- `par`/`sel`/`pmap` are sequential; real async needs `tokio`
- Named-arg parser consumes ternary `:` separator (workaround: parens)
- Assert parsing greedy — `assert (expr) "msg"` consumes msg when `(expr)` is callable
- Currying removal deferred — requires parser architecture change
- `it` in `sel` blocks — only implicit binding
- Shell line is single-line only — forces `${ }` for complex commands
- Named args + default params + currying interaction
- Unicode chars in lexer cause panics (byte vs char indexing in comments)

## Session History

| # | Date | Focus |
|---|------|-------|
| 1-5 | 03-13 | Spec authoring, contradiction fixes, test files |
| 6-7 | 03-13 | First Rust impl — lexer, parser, interpreter, type defs |
| 8-14 | 03-14 | Parser fixes, iterators, shell, concurrency, modules, agents, Protocol |
| 15-18 | 03-14 | Stdlib (json/ctx/math/fs/env/re/md/agent/mcp), MCP HTTP transport |
| 19 | 03-14 | Removed 7 features (regex, $$, <>, sets, iterators, types, tuple semis) |
| 20-24 | 03-14 | yield, MCP decls, std/http, std/time, file splits, with/field update |
| 25 | 03-14 | Stale spec cleanup — all 22 spec files updated |
| 26 | 03-14 | std/cron, real-flow gap analysis vs mcp-toolbelt arch_diagrams |
| 27 | 03-15 | Repo reorg (asl/ -> spec/design/tests/flows), 14 flow programs + specs |
| 28 | 03-15 | Design review: types + regex back, full stdlib roadmap |
| 29 | 03-15 | Type annotations + checker: AST, parser, bidirectional inference, `lx check` |
| 30 | 03-15 | Regex literals: `r/\d+/flags`, Value::Regex, std/re accepts both, 25/25 tests |
| 31 | 03-15 | Agentic features: `~>>?` streaming, checkpoint/rollback, capabilities, blackboard, events, negotiation |
| 32 | 03-15 | Agentic layer completion: dialogue, interceptors, handoff, plan revision, introspection, knowledge cache |
| 33 | 03-15 | std/ai + std/tasks + std/audit + std/circuit + std/knowledge + std/plan + std/introspect. 19 stdlib modules, 32/32 tests |
| 34 | 03-15 | Agent self-assessment: 10 missing features identified. 8 new spec files + updates to 10 existing files |
| 35 | 03-15 | Standard agents (auditor/router/grader/planner/monitor/reviewer) + std/memory + std/trace. Deduplication audit: shared LLM parsing, shared eval records, shared keyword matching. 40/40 tests |
| 36 | 03-15 | std/diag: program visualization. AST walker extracts workflow graph, emits Mermaid. `lx diagram` CLI. Shared `read_and_parse` in CLI. 41/41 tests |
| 37 | 03-15 | RuntimeCtx design: spec/runtime-backends.md. Context parameter on builtins for swappable AI/HTTP/shell/emit/yield/log backends. Updated NEXT_PROMPT, CURRENT_OPINION, stdlib_roadmap, runtime.md, agents-advanced.md |
