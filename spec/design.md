# Design Decisions

Every non-obvious choice in lx, with rationale. These are decisions, not axioms — each one was chosen over alternatives and could theoretically be revisited. The axioms live in [README.md](../README.md).

## Agentic Identity

**lx is an agentic workflow language** — not a general scripting language with agent features bolted on. The core use case is: an agent writes an lx program that spawns subagents, invokes tools, manages context, and orchestrates multi-step workflows. The existing language primitives (pipes, pattern matching, closures, `par`/`sel`, `^`/`??`) are the instruction set for the agentic layer.

**Agent communication has language-level syntax** — `~>` (send) and `~>?` (ask) are infix operators recognized by the parser, just as `$` identifies shell commands. The AST has distinct nodes for agent messages. Agent lifecycle (`agent.spawn`, `agent.channel`) and tools (`mcp.call`, `ctx.load`) remain library functions — they don't need special syntax because they're one-shot setup operations, not the core communication loop.

**Agents are opaque values** — Like `Handle` and `Duration`, an `Agent` cannot be destructured. It's created by `agent.spawn`, communicated with via `~>` (send) / `~>?` (ask), and managed via `agent.channel`. This prevents agents from being accidentally serialized or compared — they're process handles, not data.

**Context is immutable** — `ctx.set` returns a new context, not mutating in place. This matches lx's immutable-by-default principle and makes context threading through pipelines clean: `ctx.load path ^ | ctx.set "k" v | (c) ctx.save path c ^`.

**MCP is the unified tool interface** — Shell (`$`), HTTP (`std/http`), and file I/O (`std/fs`) remain for direct use, but MCP (`std/mcp`) provides the generalized tool abstraction. An agent that needs to invoke arbitrary tools uses MCP. This follows the principle of specific tools for common cases, generic interface for extensibility.

## Syntax

**Braces over indentation** — `}` is one token, unambiguously closes scope. Indentation requires counting whitespace tokens; my tokenizer makes this unreliable. Every Python-style generation I do risks an invisible tab/space mismatch.

**Pipes over nesting** — `data | filter (> 0) | map (* 2) | sum` generates left-to-right. I commit to each transformation as I produce it. `sum(map(lambda x: x*2, filter(lambda x: x>0, data)))` requires knowing the full nesting structure before the first token. This is my single biggest generation bottleneck in other languages.

**No keywords for common ops** — Functions: `name = (params) body`. Match: `?`. Export: `+`. Every keyword saved is a token saved across millions of invocations. 9 total keywords in the entire language.

**Sections for inline lambdas** — `(* 2)` instead of `lambda x: x * 2`. One token for the operation, one for the operand. Maximum density. Sections also work for field access: `(.name)` extracts the `name` field.

**No commas** — `[1 2 3]` not `[1, 2, 3]`. Whitespace already separates. Commas are noise tokens that I generate reflexively from training data — removing them as valid syntax prevents that waste.

**`--` for comments** — `#` was reserved for sets (removed). `//` creates ambiguity in division contexts. `--` is always unambiguous and typically one token.

**Regex literals** — `r/\d+/` compiles at evaluation time to a first-class `Regex` value. Flags after the closing slash: `r/hello/i`. All `std/re` functions accept both regex literals and string patterns. The literal form eliminates double-escaping (`\\d+`) that is hostile to LLM generation.

**`?` suffix for predicates** — `empty?`, `sorted?`, `prime?`. Single trailing `?` in identifiers. No `is_` prefix overhead. Not ambiguous with `?` match operator because `ident?` (no space) is a name while `expr ?` (space + block) is a match.

## Data Flow

**No UFCS** — Pipes handle function chaining. `.field` handles data access. `x | to_string` not `x.to_string()`. Two mechanisms, zero overlap. Methods are just functions that take their "receiver" as the last argument.

**Data-last arguments** — `map f xs` not `map xs f`. This single convention makes `xs | map f` work naturally via pipes. Every stdlib function follows this: the primary data argument is always last.

**Auto-currying** — `add 3` returns `(y) 3 + y`. Only for all-positional functions (no defaults). Enables `map (add 3)` without wrapper lambdas. Functions with default parameters are called once all required params are filled — no currying past defaults.

**`+` for exports** — Single character at column 0. No `pub`/`export`/`module.exports` boilerplate.

## Shell

**`$` for shell** — Shell commands are the most common scripting operation. `$ls src` is unambiguous — `$` always means shell. Three variants cover all needs:
- `$cmd` — interpolated, returns Result
- `$^cmd` — propagates error on nonzero exit
- `${ ... }` — multi-line shell block

The original design used `!` for shell, but `!cmd` vs `!expr` (logical not) vs `sort!` (identifier) vs `-> Str ! Err` (error annotation) was four-way ambiguity — exactly the overloading that causes generation errors.

## Error Handling

**`^` for errors** — `read path ^` propagates errors (like Rust's `?`). Postfix `^` unwraps `Ok`/`Some` or returns `Err` to the caller. I chose `^` over `?` because `?` is already the match operator.

**`??` for coalesce** — `read path ?? "fallback"`. Unwrap-or-default in two characters. Composes beautifully with sections: `map (?? default)` coalesces each element in a pipeline.

## Concurrency

**Structured concurrency only** — `par { ... }` runs expressions concurrently, returns results as tuple. `pmap f xs` for parallel map. `sel { ... }` for racing. No unstructured `spawn`/`go` with manual `await` — those create dangling futures, which are exactly the state-tracking bugs I'm worst at. If a `par` block errors, all siblings are cancelled.

**`<-` is exclusively reassignment** — `x := 5` creates mutable binding, `x <- 10` reassigns. No overloading with await or channel operations. Agent communication uses `~>` (send) and `~>?` (ask) — distinct operators that avoid `<-` ambiguity. Concurrency uses `par`/`sel` blocks.

## Types and Mutability

**Optional type annotations** — Parameters, return types, and bindings can be annotated: `(x: Int y: Str) -> Result Int Str { ... }`. `lx check` validates annotations via bidirectional inference + unification. `lx run` ignores them — the language stays dynamic at runtime.

**No formal traits in v1** — Structural subtyping means any record with matching fields satisfies a type constraint. No explicit trait/interface definitions needed for scripting.

**`'` suffix for mutating variants** — `sort` returns new sorted list, `sort'` sorts in place. Mutation is the exception, not the rule — the `'` is a visual flag that something unusual is happening. Rare; only for performance-critical inner loops.

**Eager evaluation** — Ranges produce lists. Pipeline stages (`map`, `filter`, `take`) operate eagerly on lists.

## Debugging

**`dbg` is pipeline-transparent** — `dbg expr` prints `[file:line] expr = value` and returns value unchanged. Drop it anywhere in a pipeline: `data | dbg | filter pred | dbg | map f`. I can't attach a debugger or set breakpoints. Self-describing trace output IS the debugger.

**`tap f` for side effects** — `tap` applies a function for its side effects and returns the original value. `data | tap (d) log.debug "count: {d | len}" | process`. Distinct from `dbg`: `tap` runs arbitrary code, `dbg` is specifically trace output.

## Resolved Questions

Decisions made during the v0.1 specification pass. Each resolves an open question from early design.

**No iterator protocol** — Removed. Pipelines operate on lists and ranges. Custom iteration uses `loop`/`break`.

**Duration values are stdlib functions** — `time.sec 5`, `time.ms 100`, `time.min 2`. No literal suffixes (`5s`, `100ms`). Suffixes complicate the lexer for marginal token savings. Functions compose naturally: `time.sec 5 + time.ms 500`.

**`defer` is a built-in function** — `defer () cleanup`. Takes a zero-argument function (closure), registers it for execution when the enclosing scope exits (normal return, error propagation, break, or signal). Multiple defers run LIFO. Not a keyword — it's a function that receives a closure. The closure captures any needed state.

**Map keys are always expressions** — `%{expr: val}` always evaluates `expr`. Write `%{"foo": val}` for string keys. `%{foo: val}` evaluates `foo` as a variable. No ambiguity, no identifier-as-string shorthand.

**`^` works on Maybe values** — `expr ^` where `expr` is `Maybe a` converts `None` to an `Err` with source location info and propagates. For descriptive error messages, use `require`: `env.get "PATH" | require "PATH not set" ^`.

**No truthiness** — `?` ternary and single-arm forms require `Bool`. `0 ? "yes" : "no"` is a type error. `"" ? "yes" : "no"` is a type error. Use explicit comparisons: `x > 0 ?`, `s != "" ?`. This prevents bugs where non-boolean values are accidentally used as conditions.

**Bitwise operations are not in v1** — `|`, `&`, and `^` are used for pipes, guards, and error propagation respectively. Bitwise ops are rare in agentic scripting.

**Add `fs.read_lossy`** — Three functions for the three use cases: `fs.read` for UTF-8 (errors on invalid), `fs.read_bytes` for binary, `fs.read_lossy` for messy files (replaces invalid bytes with U+FFFD).

**Flat error codes in v1** — `error[type]`, `error[parse]`, not `error[type.mismatch]`. The error message itself carries specifics. Dotted subcodes add indexing complexity for marginal benefit in a scripting context.

**Ranges are ascending only** — `10..1` is an empty sequence. Use `1..=10 | rev` for descending. This eliminates ambiguity about whether `10..1` means "empty" or "countdown."

**`(expr)` is grouping, not a one-element tuple** — `(1 + 2)` evaluates to `3`. `(1 2)` is a 2-tuple. `()` is unit. There is no one-element tuple; use `[x]` if a single-element container is needed.

**No or-patterns** — `1 | 2 -> ...` in match arms conflicts with pipe syntax. Use guards: `n & (n == 1 || n == 2) -> ...`.

**No string patterns** — Match arms cannot destructure strings via interpolation. Use regex or string functions with guards.

**No `continue` keyword** — Use pattern matching inside `loop` to skip iterations, or prefer `filter` pipelines which express "skip" naturally.

**No format-string mini-language** — No `"{x:.2f}"`. Use `to_str` and string interpolation. Zero new syntax.

**No comprehensions** — `map`/`filter`/`fold` with sections and pipes cover every comprehension use case with fewer concepts. `[1..10] | filter even? | map (* 2)` instead of `[x * 2 for x in 1..10 if x % 2 == 0]`.

**`^` and `??` are lower precedence than `|`** — Error propagation and coalescing apply to pipeline results, not to individual functions. `url | fetch ^` parses as `(url | fetch) ^`. This means `^` unwraps the result of the whole pipe stage. To apply `^` before piping, use parens: `(fs.read path ^) | process`. The alternative (high-precedence `^`) would make `url | fetch ^` try to unwrap the function `fetch` itself — a type error.

**`$^cmd` returns `Str`, not the full shell record** — `$cmd` returns `Result {out err code} ShellErr` for full access to stdout, stderr, and exit code. `$^cmd` returns `Str ^ ShellErr` — extracts stdout on exit 0, propagates error on nonzero exit. This makes `$^cmd | trim` work without field access boilerplate.

**`? {` always starts a match block** — No ambiguity between records-as-ternary-values and match blocks. `cond ? {x: 1}` enters match mode. For record literals in ternary position, wrap in parens: `cond ? ({x: 1}) : ({x: 0})`. This eliminates a lookahead requirement.

**`assert` panics, not recoverable** — `assert` is for invariant checking and tests, not for error handling. A failed assertion prints the expression, source location, and optional message, then aborts. In `lx test` mode, the test runner catches panics. This is distinct from `Result`/`^`/`??` which handle expected, recoverable failures.

**No mutable captures in concurrent code** — Capturing a mutable binding inside `par`/`sel`/`pmap` bodies is a compile error. This prevents data races without locks or atomics. Concurrent code operates on immutable data and returns results; aggregation happens sequentially.

**Implicit Err early return removed** — The original design had `-> T ^ E` annotated functions automatically return on bare `Err` values. This was removed along with type annotations. Error handling uses `^` for propagation from called functions and explicit conditionals for locally-constructed errors.

**`defer` is per-block-scope** — `defer () cleanup` runs when the immediately enclosing `{}` block exits (normal return, `^` error propagation, `break`). Inside a loop, a `defer` in the loop body runs every iteration. Put defers next to the resource they clean up, at the scope level where cleanup should happen.

**Top-level bindings support forward references** — Top-level function bindings can reference each other regardless of definition order, enabling mutual recursion. Within blocks, bindings are sequential.

**Variant constructors are module-unique** — Two tagged unions in the same module cannot share a variant name. This ensures unambiguous pattern matching without requiring qualified names in match arms.

**Division by zero is a runtime panic** — `10 / 0` panics with a clear diagnostic, like `assert`. It does not return `Err DivZero`. Making `/` return `Result` would force `^` after every division, violating axiom #1 (fewest tokens). Division by zero is a programmer bug (you should validate your divisor), not an expected failure. For data processing where zero divisors are expected, use `math.safe_div a b` which returns `Result`. Modulo and integer division (`%`, `//`) follow the same rule. Implemented in: [impl-interpreter.md](../design/impl-interpreter.md).

**Tuple auto-spread in function application** — When a function with N parameters is called with a single tuple of arity N as the sole argument, the tuple is automatically spread into the parameters. This enables `enumerate | each (i item) body` and `entries | map (k v) k ++ v` to work without explicit destructuring syntax. Without this, users would need `each ((i item)) body` with double parens, which is noisy. This is consistent with data-last + pipe composition. Implemented in: [impl-interpreter.md](../design/impl-interpreter.md).

**`+main` must be a function** — If a `+main` binding exists and is not a function value, the interpreter reports `error[type]: +main must be a function`. Top-level bindings are evaluated before `main` is called, so `+main = 42` would type-error at invocation time.

**Import shadows built-in with warning** — `use std/fs {read}` makes `read` refer to `fs.read` in the current scope, shadowing any built-in named `read`. The compiler emits `warning: import 'read' shadows built-in`. This matches shadowing-with-`=` semantics — explicit imports are intentional.

**`pmap_n` for rate-limited concurrency** — `pmap_n limit f xs` runs at most `limit` concurrent tasks at a time. Rate-limited APIs are too common to defer to v2. The implementation uses a tokio semaphore. `pmap` remains unlimited for CPU-bound work.

**`none?` is exclusively a collection predicate** — `none? pred xs` checks that no element satisfies `pred`. There is no 1-arg `none?` for `Maybe` values. Use `!some? m` or pattern matching to check for `None`. This avoids ambiguity with auto-currying: `none? even?` unambiguously means "curried predicate: check that no element is even." `some?` remains the 1-arg `Maybe` predicate.

**`emit` for agent-to-human output** — `emit expr` sends a value to whoever invoked the agent — human, orchestrator, or parent. Fire-and-forget: returns `()`, does not block. Every flow uses `$echo` for user-facing output, but `$echo` is a shell command — it spawns `/bin/sh`, produces unstructured text, and is indistinguishable from any other shell execution. An orchestrator cannot tell `$echo "step 2 done"` from `$curl https://...`. `emit` is to human communication what `~>` is to agent communication: a dedicated primitive with its own AST node, runtime semantics, and interception point. In standalone mode, strings go to stdout directly; structured values are JSON-encoded. In orchestrated mode, the host's `EmitHandler` callback intercepts the value. Works with `Protocol` validation for typed output contracts. `emit` replaces `$echo` for all user-facing output in flows.

**`yield` for coroutine execution** — `yield expr` pauses execution, sends value to an orchestrator callback, and returns the orchestrator's response. Callback-based, no threading. Without a handler, `yield` is a runtime error. This gives external orchestrators (Python, shell, other lx programs) control over execution flow without requiring concurrency primitives inside lx itself.

**`with` for scoped bindings** — `with name = expr { body }` creates a child scope, binds name, evaluates body, returns body's last value. Supports `:=` for mutable bindings within the block. Not dynamic scope — lexical. Useful for temporary bindings that should not leak into the surrounding scope.

**Record field update via `<-`** — `name.field <- value` updates a field on a mutable record binding. Functional update internally — produces a new record with the field changed and rebinds. Requires `:=` binding on the record. Nested paths supported: `name.a.b <- value`. Consistent with `<-` as the reassignment operator.

**`Protocol` for message contracts** — `Protocol Name = {field: Type}` validates record shape at runtime. Callable — `Name {field: value}` returns a validated record with defaults filled in. Protocols enforce structural contracts between agents without a compile-time type system, catching shape mismatches at the boundary where data enters a function or agent.

**`MCP` for typed tool contracts** — `MCP Name = { tool_name { field: Type } -> OutputType }` declares MCP tool interfaces with input/output validation. Each tool block specifies its parameter schema and return type. Calling through an MCP declaration validates arguments before sending and validates the response before returning, turning protocol errors into structured `Err` values.

**`~>>?` for streaming agent responses** — `agent ~>>? msg` returns a lazy sequence of partial results. The receiving agent `yield`s incremental chunks; the caller iterates them as they arrive. Without streaming, the caller blocks until the full response is ready — no early course-correction, no progress visibility. `~>>?` composes with pipes and iteration: `agent ~>>? {task: "review"} | each (chunk) display chunk`. At the same precedence level as `~>` and `~>?`.

**`checkpoint`/`rollback` for transactional execution** — `checkpoint "name" { body }` snapshots mutable state (context, bindings) before executing `body`. Inside the block, `rollback "name"` restores the snapshot and exits the block with `Err`. This makes agentic trial-and-error safe by default — try a refactor, run tests, roll back if tests fail. Without this, agents must manually snapshot and restore state, which is error-prone and verbose. `checkpoint` is a keyword; `rollback` is a built-in function that only works inside a `checkpoint` block.

**Capability attenuation on agent.spawn** — `agent.spawn` accepts a `capabilities` field that restricts what the subagent can do: `{tools: [Str]  fs: {read: [Str] write: [Str]}  network: Bool  budget: {tokens: Int}}`. Every subagent should run with least-privilege. Without capability attenuation, spawning a subagent gives it the parent's full permissions — the single biggest safety gap in multi-agent systems. Capabilities are enforced by the runtime, not by convention.

**Shared blackboard via std/blackboard** — `ctx` is single-owner. When multiple agents collaborate on one problem in `par`, they can't see each other's intermediate findings. `std/blackboard` provides a concurrent shared workspace: `blackboard.create`, `blackboard.read key`, `blackboard.write key val`, `blackboard.watch key callback`. Agents read and write to the same board during parallel execution. Conflict resolution is last-write-wins (simplest model that works for agentic workflows where semantic merging is the agent's job).

**Pub/sub via std/events** — All agent communication is point-to-point (`~>`, `~>?`). `std/events` adds topic-based broadcast: `events.create`, `events.publish bus topic msg`, `events.subscribe bus topic handler`. Multiple agents react to the same events without the publisher knowing who's listening. Essential for reactive multi-agent systems where agents respond to environmental changes, not just direct messages.

**Negotiation is a pattern, not a primitive** — Before committing to work, agents should be able to negotiate scope, constraints, and expected outputs. This uses existing features: `Protocol Offer`, `Protocol Accept`, `Protocol Reject` with a `counter_offer` field. The router and planner standard agents will adopt this pattern. No new syntax needed — Protocols and pattern matching handle it.

**`dialogue` for multi-turn agent conversation** — `~>?` is single request-response. Real agent collaboration is multi-turn. `agent.dialogue` is a library function (not a keyword) following the precedent of `agent.spawn` — it's a setup operation that creates a `Session` value, not a core communication loop. Dialogue sessions accumulate history on both sides via JSON-line protocol extension. Each `dialogue_turn` carries accumulated context without the caller manually threading history. This is the difference between HTTP requests and a WebSocket session.

**`intercept` for message middleware** — Cross-cutting concerns (tracing, rate-limiting, context injection) need to wrap agent communication without modifying every call site. `agent.intercept` is a library function that returns a new wrapped agent — the original is unchanged (immutable). Interceptors compose by wrapping: `intercept (intercept agent m1) m2`. The middleware function takes `(msg next)` where `next` is the continuation — not calling `next` short-circuits. This follows the middleware pattern from HTTP frameworks, which is well-understood and composable. Considered making it a keyword, but it's a one-shot wrapping operation like `agent.spawn`, not control flow.

**`handoff` for structured context transfer** — When agent A finishes and agent B takes over, the transfer loses metadata about what A tried, assumed, and was uncertain about. `agent.handoff` and `agent.as_context` are library functions in `std/agent`. A `Handoff` Protocol defines the standard shape: result, tried, assumptions, uncertainties, recommendations, files_read, tools_used. The agent constructs this explicitly — no auto-population (that's `std/introspect`'s job). `as_context` renders it as a prompt-friendly string for LLM-backed agents.

**`std/plan` for dynamic plan revision** — `yield` pauses for input; `checkpoint`/`rollback` undoes. Neither revises the remaining plan forward. `std/plan` treats plans as data (lists of step records with dependencies) and provides `plan.run` with an `on_step` callback that can return `plan.replan`, `plan.insert_after`, `plan.skip`, or `plan.abort`. This is a library module, not syntax, because plan execution is a higher-order pattern built on existing features. The `std/agents/planner` generates initial plans; `std/plan` executes them with revision.

**`std/knowledge` for shared discovery cache** — `std/blackboard` is in-memory, `par`-scoped, last-write-wins. `std/ctx` is per-agent, immutable. Neither prevents 3 agents from independently reading the same files. `std/knowledge` is file-backed (like `std/ctx` for consistency), shared via path, with provenance metadata (source, confidence, tags) and query support. Key convention: `"file:{path}"` for reads, `"tool:{name}:{hash}"` for tool calls, `"fact:{desc}"` for conclusions. This is session-scoped shared facts — distinct from `std/memory` which is long-term agent-internal learning.

**`std/introspect` for agent self-awareness** — `std/circuit` fires when limits are breached. `std/introspect` lets agents proactively reason about their own state: budget remaining, actions taken, whether they're stuck. Separate module (not an extension of `std/agent`) because introspection is cross-cutting — it reads runtime metadata that goes beyond agent lifecycle. The interpreter collects action history as a side effect of evaluation, bounded to last 1000 entries.
