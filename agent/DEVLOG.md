# lx Development Log

Design decisions, technical debt, and session history. Read `NEXT_PROMPT.md` first for orientation.

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
- **`yield` is callback-based**. No handler → runtime error. Subsumed by `RuntimeCtx.yield_` backend.
- **`RuntimeCtx` for backend pluggability**. All I/O builtins receive `&Arc<RuntimeCtx>`. Traits: `AiBackend`, `EmitBackend`, `HttpBackend`, `ShellBackend`, `YieldBackend`, `LogBackend`. Standard defaults in `crates/lx/src/backends/defaults.rs`.
- **`with` is scoped binding**. Lexical scope, not dynamic. Supports `:=` mutable.
- **`with ... as` uses `stop_ident` parser flag**. Expression parsing stops at `Ident("as")` to prevent it being consumed as application argument. Multi-resource separated by `,` (Semi token). Cleanup via `Closeable` convention (record with `.close` field).
- **Record field update via `<-`**. Requires `:=` binding. Adding new fields allowed.
- **`MCP` declarations are typed tool contracts**. Callable → record of wrapper functions.
- **Type annotations don't consume lowercase idents as type args**. `(x: Maybe a)` treats `a` as next param, not type var. Write `(x: (Maybe a))`.
- **`{` after type tokens is body, not record type**. `-> Int { body }` — `{` starts body. Record return types need parens: `-> ({x: Int})`.
- **`lx check` is optional**. `lx run` ignores annotations. Checker uses bidirectional inference + unification.
- **Protocol failures are runtime errors**, not lx-level `Err` values. Not catchable with `??`.
- **Protocol `where` constraints bind the field name**. `score: Float where score >= 0.0`.
- **Protocol union `_variant` injection**. First matching variant in declaration order injects `_variant` field.
- **Protocol spread overrides**: later fields override same-named spread fields.
- **`Trait` is a keyword**. Lexed like `Protocol`/`MCP` (uppercase → TypeName, but special-cased). Produces `Stmt::TraitDecl` and `Value::Trait`. Trait names stored in agent `__traits` list field.
- **`agent.implements` not `agent.implements?`**. The `?` suffix parses as ternary operator. Use bare name.
- **`std/pool` is sequential**. Like `par`/`pmap`, pools distribute work sequentially. Abstraction value is organizational — round-robin dispatch, status tracking.
- **`std/plan` treats plans as data**. `plan.run` with `on_step` callback. `PlanAction` tagged union controls flow.
- **`std/introspect` is separate from `std/agent`**. Cross-cutting runtime metadata. Bounded action log (1000 entries).
- **`std/knowledge` is file-backed JSON**. Shared via path. Provenance metadata. File-level locking.
- **`std/diag` uses existing lexer+parser**. Walks AST, does not execute. Graph IR is plain lx records.

## Technical Debt

- Builtins (str.rs, coll.rs, hof.rs) still use "X expects Y" without showing actual type — interpreter errors are fixed, stdlib next
- `par`/`sel`/`pmap`/`std/pool` are sequential; real async needs `tokio`
- Named-arg parser consumes ternary `:` separator (workaround: parens)
- Assert parsing greedy — `assert (expr) "msg"` consumes msg when `(expr)` is callable
- Currying removal deferred — requires parser architecture change
- `it` in `sel` blocks — only implicit binding
- Shell line is single-line only — forces `${ }` for complex commands
- Named args + default params + currying interaction
- Unicode chars in lexer cause panics (byte vs char indexing in comments)
- 19+ files over 300-line limit (parser: 3, interpreter: 4, builtins: 2, stdlib: 5+, lexer: 1, core: 1)

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
| 35 | 03-15 | Standard agents (auditor/router/grader/planner/monitor/reviewer) + std/memory + std/trace. 40/40 tests |
| 36 | 03-15 | std/diag: program visualization. AST walker, Mermaid output. `lx diagram` CLI. 41/41 tests |
| 37 | 03-15 | RuntimeCtx design + implementation. Backend traits. Refine expression. agent.reconcile. 44/44 tests |
| 38 | 03-15 | trace.improvement_rate + trace.should_stop: diminishing returns detection. 45/45 tests |
| 39 | 03-15 | Agent communication layer: dialogue, intercept, handoff, capabilities, gate, supervise, mock, dispatch, ai.prompt_structured. 54/54 tests |
| 40 | 03-15 | Protocol extensions: composition, unions, field constraints. 56/56 tests |
| 41 | 03-16 | `with ... as` scoped resources, `Trait` declarations + `agent.implements`, `std/pool`. 59/59 tests |
| 42 | 03-16 | Tier 1 stdlib (`std/budget`, `std/prompt`, `std/context`) + Tier 2 agent extensions (`agent.negotiate`, `agent.topic`/`subscribe`/`publish` pub/sub). 64/64 tests |
| 43 | 03-16 | `std/git`: structured git access — 36 functions (status, log, diff, blame, grep, add, commit, branch, stash, remote). 7 Rust files, unified diff parser. 65/65 tests |
| 44 | 03-16 | Gap analysis: 7 new specs for unplanned features (profile, interrupt, constraint propagation, provenance, workspace, teaching, pipeline checkpoint). Priority queue restructured. `std/blackboard`/`std/events` eliminated |
| 45 | 03-16 | `std/retry`: retry-with-backoff (2 functions, `fastrand` dep for jitter). Improved binding pattern-match error messages. 66/66 tests |
| 46 | 03-16 | Spec consolidation: 9 merges applied. Eliminated `std/strategy` (→ `std/profile`), `std/reputation` (→ `std/trace`), `checkpoint`/`on_interrupt` keywords (→ `user.check` + `:signal` lifecycle hook), `plan.run_incremental` (→ `std/pipeline`), `agent.teach` (→ dialogue convention), `workflow.peers` (→ topic convention), constraint propagation spec (→ `with context` ambient), provenance spec (→ `std/trace`), `Goal`/`Task` (→ docs). Agent/Trait declaration specs from Session 45b integrated. Net: 21 planned features (down from ~33) |
| 47 | 03-16 | Error message overhaul: cross-language keyword hints (30+ keywords), value/type in all mismatch errors, Pattern Display impl, Value::short_display(). `cargo install` to host. Quick syntax reference in NEXT_PROMPT for cold-start agents. 66/66 tests |
| 48 | 03-16 | Gap analysis: 7 unplanned features for dynamic multi-agent coordination. New specs: `agents-task-graph` (DAG execution), `agents-capability-routing` (declarative routing), `agents-deadline` (time propagation), `agents-introspect-live` (system observation), `agents-dialogue-branch` (fork/compare/merge), `agents-format-negotiate` (Protocol adapters), `agents-hot-reload` (handler swap). Updated ROADMAP (28 features), PRIORITIES (28 items), OPINION (8 new gaps). No code changes |
