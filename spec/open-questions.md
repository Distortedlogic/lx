# Open Questions

All v0.1 design questions have been resolved. Decisions and rationale are in [design.md](design.md). This file tracks considerations for future versions.

## Resolved in v0.1 (Summary)

| Question | Decision |
|---|---|
| User-defined generators | Removed — use `loop`/`break` for custom iteration |
| Duration literals | Stdlib functions: `time.sec 5`, `time.ms 100` |
| Retry combinator | Stdlib: `retry n f` with optional backoff config |
| Interactive/notebook mode | `lx notebook` — shared environment, `---` separated blocks |
| Shell string escapes | `$` interpolates `{expr}` |
| Error code taxonomy | Flat codes in v1: `error[type]`, `error[parse]` |
| Bytes/Str boundary | Three functions: `fs.read`, `fs.read_bytes`, `fs.read_lossy` |
| Signal handling | `defer` built-in: `defer () cleanup`. LIFO on scope exit |
| Map literal key types | Expression keys: `%{expr: val}` always evaluates `expr` |
| `^`/`??` vs `\|` precedence | `^` and `??` are lower than `\|` — apply to pipeline results |
| `$^` return type | Returns `Str ^ ShellErr` (stdout extracted), not full record |
| `? {` disambiguation | `?` followed by `{` always starts a match block |
| `assert` semantics | Hard panic, not recoverable via `^`/`??`. Test runner catches. |
| Mutable captures in concurrency | Compile error — prevents data races |
| `defer` scope | Per-block-scope, not per-function |
| Forward references | Top-level bindings visible to each other. Within blocks, sequential. |
| Type inference | Bidirectional with local inference. Per-function checking. |
| Variant uniqueness | Variant constructors must be unique within a module |

## Resolved Post-v0.1

| Question | Decision |
|---|---|
| Division by zero | Runtime panic, not `Result`. Use `math.safe_div` for recoverable. |
| Tuple auto-spread | Function with N params + single N-tuple arg = auto-spread |
| `+main` type | Must be a function, compile error otherwise |
| Import shadowing | Selective imports shadow built-ins with a warning |
| `pmap_n` | Added to v1: `pmap_n limit f xs` for rate-limited concurrency |
| `none?` ambiguity | Exclusively 2-arg collection predicate. Use `!some?` for Maybe. |

## Resolved Post-v0.1 (Session 4)

| Question | Decision |
|---|---|
| Data processing at scale | Deferred — no stdlib modules for dataframes, databases, or ML in v1 |
| MCP transport | Both stdio and HTTP+SSE are implemented |
| Message serialization | JSON (serde_json) for inter-agent messages |

## Agentic Design Questions (v1)

These are open questions for the agentic layer — to be resolved during implementation:

| Question | Considerations |
|---|---|
| Agent process model | Are agents subprocesses (CLI invocations), API calls, or both? Should `agent.spawn` support both local and remote agents? |
| Agent discovery | How do agents find each other? Registry, well-known names, URIs? |
| Channel backpressure | What happens when a channel sender outpaces the receiver? Buffer, drop, block? |
| Context conflict resolution | When two agents update the same context key, who wins? Last-write-wins, CRDTs, error? |
| Agent lifecycle | What happens to subagents when the parent dies? Orphan cleanup? |
| Workflow resumability | How to serialize workflow state for checkpoint/resume across process restarts? |
| Agent permissions | Can an agent restrict what tools its subagents can use? Capability model? |

## Considerations for v2

These are not blockers for v1 implementation but worth revisiting after real-world usage:

**Dotted error codes** — `error[type.mismatch]` vs `error[type]`. Flat is simpler for v1. If programmatic error filtering becomes a real need, add subcodes.

**Or-patterns in match arms** — `1 | 2 -> ...` conflicts with pipe. Could use `1 , 2 -> ...` or `[1 2] -> ...` as set-of-values syntax. Guards work but are verbose for large literal sets. Revisit if pattern matching on sets of values proves common.

**String interpolation patterns** — matching `"http://{rest}"` in pattern arms. Powerful but complex to implement. Regex handles the same cases. Revisit if regex patterns in match arms prove too verbose.

**~~Concurrency limits on `pmap`~~** — Resolved: `pmap_n limit f xs` added in v1. Rate-limited APIs are too common to defer. See [concurrency.md](concurrency.md).

**Streaming/channel primitives** — `par`/`sel`/`pmap` cover request-response concurrency. Long-running producers/consumers (event streams, queue workers) may need channels. Defer until real use cases emerge.

**CLI argument parser** — `std/args` or `std/cli` for declarative argument parsing (flags, options, subcommands). v1 uses `env.args` with pattern matching, which handles simple cases. A structured parser would help for complex CLIs.

**Plugin/extension system** — Loading native (Rust/C) functions as lx modules for performance-critical operations. The FFI boundary would need careful design around error handling and type mapping.

**WASM target** — `lx build --target wasm` for running lx scripts in browsers or edge runtimes. The runtime model (async I/O, work-stealing) needs adaptation.

**Pattern matching on regex** — Using regex directly in match arms instead of guards with `std/re`. Currently requires guards: `s & (re.is_match r/\d+/ s)`.

**`where` clauses for type constraints** — Currently there's no way to express "this generic type must support equality" or "must be sortable." Structural typing handles fields, but behavioral constraints (like "has a `<` operator") are implicit. Revisit if the lack of constraints causes confusing error messages.
