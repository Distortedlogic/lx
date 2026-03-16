# Design Decisions — Reference

Resolved questions as compact FAQ. Each entry is a decision, not an axiom.

| Question | Decision |
|---|---|
| Braces vs indentation | Braces — `}` is one unambiguous token |
| Pipes vs nesting | Pipes — left-to-right generation, no lookahead |
| Keyword minimalism | 9 keywords total. Functions: `name = (params) body`. Match: `?`. Export: `+`. |
| Inline lambdas | Sections: `(* 2)`, `(.name)` |
| Commas | None — whitespace separates |
| Comment syntax | `--` (always unambiguous) |
| Regex literals | `r/\d+/` with flags after closing slash: `r/hello/i` |
| Predicate suffix | `empty?`, `sorted?` — trailing `?` in identifiers |
| UFCS | None — pipes for chaining, `.field` for access |
| Data-last arguments | `map f xs` so `xs \| map f` works via pipes |
| Auto-currying | All-positional functions only (no defaults). `add 3` returns `(y) 3 + y` |
| Exports | `+` at column 0 |
| Shell syntax | `$cmd` (interpolated Result), `$^cmd` (propagate, returns Str), `${ }` (multi-line) |
| Error propagation | `^` postfix (like Rust's `?`). `??` for coalesce. |
| Concurrency model | Structured only: `par`, `sel`, `pmap`. No unstructured spawn/await. |
| `<-` meaning | Exclusively reassignment. `x := 5` creates mutable, `x <- 10` reassigns. |
| Type annotations | Optional. `lx check` validates, `lx run` ignores. |
| Mutating variants | `sort'` suffix — visual flag for in-place mutation |
| Evaluation | Eager — ranges produce lists, pipeline stages operate eagerly |
| `dbg` | Pipeline-transparent: prints `[file:line] expr = value`, returns value |
| `tap f` | Side effects, returns original value |
| Iterator protocol | Removed — use `loop`/`break` |
| Duration values | Stdlib: `time.sec 5`, `time.ms 100` |
| `defer` | Built-in function, zero-arg closure, LIFO on scope exit, per-block-scope |
| Map keys | Always expressions: `%{expr: val}` evaluates `expr` |
| `^` on Maybe | Converts `None` to `Err` with source location. Use `require` for descriptive messages. |
| Truthiness | None — `?` requires `Bool`. `0 ? "yes" : "no"` is a type error. |
| Bitwise ops | Not in v1 — `\|`, `&`, `^` used for pipes/guards/errors |
| `fs.read` variants | `fs.read` (UTF-8), `fs.read_bytes` (binary), `fs.read_lossy` (replaces invalid) |
| Error codes | Flat: `error[type]`, `error[parse]` |
| Ranges | Ascending only: `10..1` is empty. Use `1..=10 \| rev`. |
| `(expr)` | Grouping, not 1-tuple. `(1 2)` is 2-tuple. `()` is unit. |
| Or-patterns | None — `1 \| 2` conflicts with pipe. Use guards. |
| String patterns | None in match arms — use regex or string functions |
| `continue` | None — use pattern matching in `loop` or `filter` pipelines |
| Format strings | None — use `to_str` + interpolation |
| Comprehensions | None — `map`/`filter`/`fold` with sections and pipes |
| `^`/`??` precedence | Lower than `\|` — apply to pipeline results |
| `$^cmd` return | `Str ^ ShellErr` (stdout extracted on exit 0) |
| `? {` | Always starts a match block |
| `assert` | Panics (not recoverable). Test runner catches. |
| Mutable captures | Compile error in `par`/`sel`/`pmap` |
| Forward references | Top-level: yes. Within blocks: sequential. |
| Variant constructors | Must be unique within a module |
| Division by zero | Runtime panic. Use `math.safe_div` for recoverable. |
| Tuple auto-spread | N-param function + single N-tuple = auto-spread |
| `+main` | Must be a function |
| Import shadowing | Shadows built-in with warning |
| `pmap_n` | `pmap_n limit f xs` for rate-limited concurrency |
| `none?` | Exclusively 2-arg collection predicate. Use `!some?` for Maybe. |
| `emit` | Agent-to-human output. Fire-and-forget, returns `()`. Strings to stdout, structured to JSON. |
| `yield` | Coroutine pause. Sends value to orchestrator callback, returns response. |
| `with` | Scoped binding: `with name = expr { body }`. Lexical, not dynamic. |
| Record field update | `name.field <- value` on `:=` bindings. Functional update internally. |
| `Protocol` | `Protocol Name = {field: Type}` — runtime record shape validation |
| `MCP` declarations | `MCP Name = { tool { field: Type } -> Out }` — typed tool contracts |
| `~>>?` streaming | Returns lazy stream. Same precedence as `~>`/`~>?`. |
| `checkpoint`/`rollback` | Snapshot mutable state, restore on rollback |
| Capability attenuation | `agent.spawn` `capabilities` field restricts tools/fs/network/budget |
| Shared blackboard | `std/blackboard` — concurrent workspace, last-write-wins |
| Pub/sub events | `std/events` — topic-based broadcast |
| Negotiation | Pattern via Protocol (Offer/Accept/Reject), not a primitive |
| Multi-turn dialogue | `agent.dialogue`/`dialogue_turn`/`dialogue_end` — library functions |
| Message middleware | `agent.intercept agent middleware` — composable wrapping |
| Structured handoff | `agent.handoff` + `Handoff` Protocol + `agent.as_context` |
| Dynamic plans | `std/plan` — plan-as-data with `on_step` callback revision |
| Discovery cache | `std/knowledge` — file-backed, provenance metadata, query support |
| Introspection | `std/introspect` — budget, actions, stuck detection |
| Reactive dataflow | `\|>>` streaming pipe, same precedence as `\|`, lazy until consumed |
| Supervision | `agent.supervise` — Erlang strategies (one_for_one/all/rest_for_one) |
| Ambient context | `with context key: val { }` — auto-propagates to agent ops |
| Inline clarification | `caller` implicit binding in handlers |
| Human-in-the-loop | `agent.gate` — structured approval with timeout policies |
| Multi-agent transactions | `std/saga` — compensating actions in reverse |
| Message priority | `_priority` field (`:critical`/`:high`/`:normal`/`:low`) |
| Capability discovery | `Capabilities` Protocol + `agent.capabilities` helper |
