# Design Decisions

Every non-obvious choice in lx, with rationale. These are decisions, not axioms — each one was chosen over alternatives and could theoretically be revisited. The axioms live in [README.md](../README.md).

## Agentic Identity

**lx is an agentic workflow language** — not a general scripting language with agent features bolted on. The core use case is: an agent writes an lx program that spawns subagents, invokes tools, manages context, and orchestrates multi-step workflows. The existing language primitives (pipes, pattern matching, closures, `par`/`sel`, `^`/`??`) are the instruction set for the agentic layer.

**Agent communication has language-level syntax** — `~>` (send) and `~>?` (ask) are infix operators recognized by the parser, just as `$` identifies shell commands. The AST has distinct nodes for agent messages. Agent lifecycle (`agent.spawn`, `agent.channel`) and tools (`mcp.call`, `ctx.load`) remain library functions — they don't need special syntax because they're one-shot setup operations, not the core communication loop.

**Agents are opaque values** — Like `Handle` and `Duration`, an `Agent` cannot be destructured. It's created by `agent.spawn`, communicated with via `~>` (send) / `~>?` (ask), and managed via `agent.channel`. This prevents agents from being accidentally serialized or compared — they're process handles, not data.

**Context is immutable** — `ctx.set` returns a new context, not mutating in place. This matches lx's immutable-by-default principle and makes context threading through pipelines clean: `ctx.load path ^ | ctx.set "k" v | (c) ctx.save path c ^`.

**MCP is the unified tool interface** — Shell (`$`), HTTP (`std/net/http`), and file I/O (`std/fs`) remain for direct use, but MCP (`std/mcp`) provides the generalized tool abstraction. An agent that needs to invoke arbitrary tools uses MCP. This follows the principle of specific tools for common cases, generic interface for extensibility.

## Syntax

**Braces over indentation** — `}` is one token, unambiguously closes scope. Indentation requires counting whitespace tokens; my tokenizer makes this unreliable. Every Python-style generation I do risks an invisible tab/space mismatch.

**Pipes over nesting** — `data | filter (> 0) | map (* 2) | sum` generates left-to-right. I commit to each transformation as I produce it. `sum(map(lambda x: x*2, filter(lambda x: x>0, data)))` requires knowing the full nesting structure before the first token. This is my single biggest generation bottleneck in other languages.

**No keywords for common ops** — Functions: `name = (params) body`. Match: `?`. Export: `+`. Every keyword saved is a token saved across millions of invocations. 9 total keywords in the entire language.

**Sections for inline lambdas** — `(* 2)` instead of `lambda x: x * 2`. One token for the operation, one for the operand. Maximum density. Sections also work for field access: `(.name)` extracts the `name` field.

**No commas** — `[1 2 3]` not `[1, 2, 3]`. Whitespace already separates. Commas are noise tokens that I generate reflexively from training data — removing them as valid syntax prevents that waste.

**`--` for comments** — `#` is used for sets (`#{1 2 3}`). `//` creates ambiguity in division contexts. `--` is always unambiguous and typically one token.

**Regex literals** — `r/pattern/flags`. I use regex constantly in scripting. A function API (`regex "\d+"`) wastes tokens on escaping and quoting. `r/` (no space) triggers regex mode in the lexer.

**`?` suffix for predicates** — `empty?`, `sorted?`, `prime?`. Single trailing `?` in identifiers. No `is_` prefix overhead. Not ambiguous with `?` match operator because `ident?` (no space) is a name while `expr ?` (space + block) is a match.

## Data Flow

**No UFCS** — Pipes handle function chaining. `.field` handles data access. `x | to_string` not `x.to_string()`. Two mechanisms, zero overlap. Methods are just functions that take their "receiver" as the last argument.

**Data-last arguments** — `map f xs` not `map xs f`. This single convention makes `xs | map f` work naturally via pipes. Every stdlib function follows this: the primary data argument is always last.

**Auto-currying** — `add 3` returns `(y) 3 + y`. Only for all-positional functions (no defaults). Enables `map (add 3)` without wrapper lambdas. Functions with default parameters are called once all required params are filled — no currying past defaults.

**`<>` for left-to-right composition** — `f <> g` = `(x) f x | g`. Follows pipe direction, not Haskell's right-to-left `.`. Useful for HOFs: `map (parse <> validate <> save)` instead of `map (x) x | parse | validate | save`.

**`+` for exports** — Single character at column 0. No `pub`/`export`/`module.exports` boilerplate.

## Shell

**`$` for shell** — Shell commands are the most common scripting operation. `$ls src` is unambiguous — `$` always means shell. Four variants cover all needs:
- `$cmd` — interpolated, returns Result
- `$$cmd` — raw (no `{expr}` interpolation, for commands with literal braces)
- `$^cmd` — propagates error on nonzero exit
- `${ ... }` — multi-line shell block

The original design used `!` for shell, but `!cmd` vs `!expr` (logical not) vs `sort!` (identifier) vs `-> Str ! Err` (error annotation) was four-way ambiguity — exactly the overloading that causes generation errors.

## Error Handling

**`^` for errors** — `read path ^` propagates errors (like Rust's `?`). `-> Str ^ IoErr` annotates fallibility in type signatures. Same symbol, same concept: "this can fail." Unambiguous because expression context (`expr ^`) and type context (`Type ^ ErrType`) are syntactically distinct. I chose `^` over `?` because `?` is already the match operator.

**`??` for coalesce** — `read path ?? "fallback"`. Unwrap-or-default in two characters. Composes beautifully with sections: `map (?? default)` coalesces each element in a pipeline.

## Concurrency

**Structured concurrency only** — `par { ... }` runs expressions concurrently, returns results as tuple. `pmap f xs` for parallel map. `sel { ... }` for racing. No unstructured `spawn`/`go` with manual `await` — those create dangling futures, which are exactly the state-tracking bugs I'm worst at. If a `par` block errors, all siblings are cancelled.

**`<-` is exclusively reassignment** — `x := 5` creates mutable binding, `x <- 10` reassigns. No overloading with await or channel operations. Agent communication uses `~>` (send) and `~>?` (ask) — distinct operators that avoid `<-` ambiguity. Concurrency uses `par`/`sel` blocks.

## Types and Mutability

**Types are always optional** — Full structural type inference. `add = (x y) x + y` and `add = (x:Int y:Int) -> Int x + y` are both valid. Annotations serve as documentation and disambiguation. Types never change runtime behavior.

**No formal traits in v1** — Structural subtyping means any record with matching fields satisfies a type constraint. No explicit trait/interface definitions needed for scripting.

**`'` suffix for mutating variants** — `sort` returns new sorted list, `sort'` sorts in place. Mutation is the exception, not the rule — the `'` is a visual flag that something unusual is happening. Rare; only for performance-critical inner loops.

**Lazy sequences** — Ranges (`1..100`), generators, and pipeline stages produce lazy sequences evaluated on demand. `nat | filter prime? | take 10` doesn't compute all primes — it pulls 10 through the pipeline and stops. Collecting operations (`sort`, `len`, `collect`) force evaluation.

## Debugging

**`dbg` is pipeline-transparent** — `dbg expr` prints `[file:line] expr = value` and returns value unchanged. Drop it anywhere in a pipeline: `data | dbg | filter pred | dbg | map f`. I can't attach a debugger or set breakpoints. Self-describing trace output IS the debugger.

**`tap f` for side effects** — `tap` applies a function for its side effects and returns the original value. `data | tap (d) log.debug "count: {d | len}" | process`. Distinct from `dbg`: `tap` runs arbitrary code, `dbg` is specifically trace output.

## Resolved Questions

Decisions made during the v0.1 specification pass. Each resolves an open question from early design.

**User-defined generators use the iterator protocol** — Any record with `next: () -> Maybe a` is iterable by pipelines. No `yield` keyword, no generator syntax. Structural typing handles it naturally. If a function returns a record with a `next` field, pipelines consume it lazily. Zero language additions, zero keyword cost.

**Duration values are stdlib functions** — `time.sec 5`, `time.ms 100`, `time.min 2`. No literal suffixes (`5s`, `100ms`). Suffixes complicate the lexer for marginal token savings. Functions compose naturally: `time.sec 5 + time.ms 500`.

**`defer` is a built-in function** — `defer () cleanup`. Takes a zero-argument function (closure), registers it for execution when the enclosing scope exits (normal return, error propagation, break, or signal). Multiple defers run LIFO. Not a keyword — it's a function that receives a closure. The closure captures any needed state.

**Map keys are always expressions** — `%{expr: val}` always evaluates `expr`. Write `%{"foo": val}` for string keys. `%{foo: val}` evaluates `foo` as a variable. No ambiguity, no identifier-as-string shorthand.

**`^` works on Maybe values** — `expr ^` where `expr` is `Maybe a` converts `None` to an `Err` with source location info and propagates. For descriptive error messages, use `require`: `env.get "PATH" | require "PATH not set" ^`.

**No truthiness** — `?` ternary and single-arm forms require `Bool`. `0 ? "yes" : "no"` is a type error. `"" ? "yes" : "no"` is a type error. Use explicit comparisons: `x > 0 ?`, `s != "" ?`. This prevents bugs where non-boolean values are accidentally used as conditions.

**Bitwise operations are stdlib functions** — `bit.and`, `bit.or`, `bit.xor`, `bit.not`, `bit.shl`, `bit.shr`. Bitwise ops are rare in scripting. The obvious operator symbols (`|`, `&`, `^`) are already used for pipes, pattern guards, and error propagation. The stdlib approach avoids all overloading.

**Add `fs.read_lossy`** — Three functions for the three use cases: `fs.read` for UTF-8 (errors on invalid), `fs.read_bytes` for binary, `fs.read_lossy` for messy files (replaces invalid bytes with U+FFFD).

**Flat error codes in v1** — `error[type]`, `error[parse]`, not `error[type.mismatch]`. The error message itself carries specifics. Dotted subcodes add indexing complexity for marginal benefit in a scripting context.

**Ranges are ascending only** — `10..1` is an empty sequence. Use `1..=10 | rev` for descending. This eliminates ambiguity about whether `10..1` means "empty" or "countdown."

**`(expr)` is grouping, not a one-element tuple** — `(1 + 2)` evaluates to `3`. `(1 2)` is a 2-tuple. `()` is unit. There is no one-element tuple; use `[x]` if a single-element container is needed.

**No or-patterns** — `1 | 2 -> ...` in match arms conflicts with pipe syntax. Use guards: `n & (n == 1 || n == 2) -> ...`.

**No string patterns** — Match arms cannot destructure strings via interpolation. Use regex or string functions with guards.

**No `continue` keyword** — Use pattern matching inside `loop` to skip iterations, or prefer `filter` pipelines which express "skip" naturally.

**No format-string mini-language** — No `"{x:.2f}"`. Use stdlib functions inside interpolation: `"{x | fmt.fixed 2}"`. Pipes inside `{expr}` compose with formatters. Zero new syntax.

**No comprehensions** — `map`/`filter`/`fold` with sections and pipes cover every comprehension use case with fewer concepts. `[1..10] | filter even? | map (* 2)` instead of `[x * 2 for x in 1..10 if x % 2 == 0]`.

**`^` and `??` are lower precedence than `|`** — Error propagation and coalescing apply to pipeline results, not to individual functions. `url | fetch ^` parses as `(url | fetch) ^`. This means `^` unwraps the result of the whole pipe stage. To apply `^` before piping, use parens: `(fs.read path ^) | process`. The alternative (high-precedence `^`) would make `url | fetch ^` try to unwrap the function `fetch` itself — a type error.

**`$^cmd` returns `Str`, not the full shell record** — `$cmd` returns `Result {out err code} ShellErr` for full access to stdout, stderr, and exit code. `$^cmd` returns `Str ^ ShellErr` — extracts stdout on exit 0, propagates error on nonzero exit. This makes `$^cmd | trim` work without field access boilerplate.

**`? {` always starts a match block** — No ambiguity between records-as-ternary-values and match blocks. `cond ? {x: 1}` enters match mode. For record literals in ternary position, wrap in parens: `cond ? ({x: 1}) : ({x: 0})`. This eliminates a lookahead requirement.

**`assert` panics, not recoverable** — `assert` is for invariant checking and tests, not for error handling. A failed assertion prints the expression, source location, and optional message, then aborts. In `lx test` mode, the test runner catches panics. This is distinct from `Result`/`^`/`??` which handle expected, recoverable failures.

**No mutable captures in concurrent code** — Capturing a mutable binding inside `par`/`sel`/`pmap` bodies is a compile error. This prevents data races without locks or atomics. Concurrent code operates on immutable data and returns results; aggregation happens sequentially.

**Implicit Err early return in Result-annotated functions** — In a function with `-> T ^ E`, any bare expression statement that evaluates to `Err e` immediately returns `Err e`. This enables the natural validation pattern: `age < 0 ? Err "too young"` followed by more checks, without nested ternary pyramids. The rule applies only to annotated functions — unannotated functions treat `Err` as an ordinary value. This resolves the "no early return" tension: `^` propagates errors from called functions, while implicit Err return handles locally-constructed errors. Together they cover every error-return pattern with zero keywords.

**`defer` is per-block-scope** — `defer () cleanup` runs when the immediately enclosing `{}` block exits (normal return, `^` error propagation, `break`). Inside a loop, a `defer` in the loop body runs every iteration. Put defers next to the resource they clean up, at the scope level where cleanup should happen.

**Top-level bindings support forward references** — Top-level function bindings can reference each other regardless of definition order, enabling mutual recursion. Within blocks, bindings are sequential.

**Bidirectional type inference** — Types propagate downward (from annotations) and synthesize upward (from literals and usage). Each function is checked independently. No global constraint solving. Error messages are localized to the ambiguous site.

**Variant constructors are module-unique** — Two tagged unions in the same module cannot share a variant name. This ensures unambiguous pattern matching without requiring qualified names in match arms.

**Division by zero is a runtime panic** — `10 / 0` panics with a clear diagnostic, like `assert`. It does not return `Err DivZero`. Making `/` return `Result` would force `^` after every division, violating axiom #1 (fewest tokens). Division by zero is a programmer bug (you should validate your divisor), not an expected failure. For data processing where zero divisors are expected, use `math.safe_div a b` which returns `Result`. Modulo and integer division (`%`, `//`) follow the same rule. Implemented in: [impl-interpreter.md](../impl/impl-interpreter.md).

**Tuple auto-spread in function application** — When a function with N parameters is called with a single tuple of arity N as the sole argument, the tuple is automatically spread into the parameters. This enables `enumerate | each (i item) body` and `entries | map (k v) k ++ v` to work without explicit destructuring syntax. Without this, users would need `each ((i item)) body` with double parens, which is noisy. This is consistent with data-last + pipe composition. Implemented in: [impl-interpreter.md](../impl/impl-interpreter.md).

**`+main` must be a function** — If a `+main` binding exists and is not a function value, the interpreter reports `error[type]: +main must be a function`. Top-level bindings are evaluated before `main` is called, so `+main = 42` would type-error at invocation time.

**Import shadows built-in with warning** — `use std/fs {read}` makes `read` refer to `fs.read` in the current scope, shadowing any built-in named `read`. The compiler emits `warning: import 'read' shadows built-in`. This matches shadowing-with-`=` semantics — explicit imports are intentional.

**`pmap_n` for rate-limited concurrency** — `pmap_n limit f xs` runs at most `limit` concurrent tasks at a time. Rate-limited APIs are too common to defer to v2. The implementation uses a tokio semaphore. `pmap` remains unlimited for CPU-bound work.

**`none?` is exclusively a collection predicate** — `none? pred xs` checks that no element satisfies `pred`. There is no 1-arg `none?` for `Maybe` values. Use `!some? m` or pattern matching to check for `None`. This avoids ambiguity with auto-currying: `none? even?` unambiguously means "curried predicate: check that no element is even." `some?` remains the 1-arg `Maybe` predicate.
