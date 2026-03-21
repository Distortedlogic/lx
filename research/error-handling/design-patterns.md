# Error Handling Design Patterns: Propagation, Context, Recovery, Trade-offs

Cross-cutting patterns that apply across languages, with specific focus on design decisions relevant to lx's `Result`/`Maybe` + `^`/`??` model.

---

## 1. Error Propagation Patterns

### Manual Checking (Go)

Every call site must explicitly check for errors:

```go
user, err := getUser(id)
if err != nil {
    return nil, fmt.Errorf("getUser(%d): %w", id, err)
}
profile, err := getProfile(user)
if err != nil {
    return nil, fmt.Errorf("getProfile: %w", err)
}
```

**Advantages**: Maximum visibility of error flow. No hidden control transfers. Every error path is explicit in the code.

**Disadvantages**: Extreme verbosity — CockroachDB has ~25,000 `if err != nil` blocks. Easy to accidentally ignore errors (`f, _ := ...`). The repetitive pattern obscures business logic.

### Automatic Propagation via Operator (Rust `?`, lx `^`)

A single operator unwraps success or returns the error:

```rust
// Rust
let user = get_user(id)?;
let profile = get_profile(&user)?;

// lx
user = get_user id ^
profile = get_profile user ^
```

**Mechanism**: On `Ok`/`Some`, extract the value and continue. On `Err`/`None`, return from the enclosing function with the error. In Rust, `?` additionally calls `From::from()` on the error, enabling automatic type conversion.

**lx's `^`** works identically to Rust's `?` but operates on dynamic types (`Result`/`Maybe` variants) rather than static types. When applied to an `Err`, it triggers `LxError::Propagate`, which carries the error value up the call stack. Functions that catch propagated errors (via `try`, pattern matching, or at call boundaries) convert the `Propagate` signal back into an `Err` value.

**Advantages**: Concise, explicit (the `^` is visible), composable. Error paths are clear without drowning the code.

**Disadvantages**: Easy to sprinkle `^` everywhere without adding context. Can create error chains that lack information about intermediate steps.

### Exception Bubbling (Python, Java, JavaScript)

Errors propagate automatically without any syntax at intermediate call sites:

```python
def get_dashboard(user_id):
    user = get_user(user_id)        # may raise UserNotFound
    profile = get_profile(user)     # may raise ProfileError
    return render(user, profile)    # errors bubble up invisibly
```

**Advantages**: Minimal syntax. Intermediate functions don't need to know about errors they can't handle.

**Disadvantages**: Invisible control flow. Functions have hidden exit points. Callers must read documentation (or source code) to know what exceptions can occur. Easy to catch too broadly (`except Exception`) or too narrowly.

### Monadic Binding (Haskell, OCaml)

```haskell
getProfile uid = do
    user    <- getUser uid       -- binds on Right, short-circuits on Left
    profile <- loadProfile user
    pure profile

-- Equivalent to:
getProfile uid = getUser uid >>= loadProfile
```

**Advantages**: Mathematically composable. Type system tracks error possibility. Clean syntax with `do` notation.

**Disadvantages**: Monad transformer stacks can become complex (`ExceptT AppError (StateT Config IO) a`). Learning curve is steep.

### Elixir `with` Expression

```elixir
with {:ok, user}    <- fetch_user(id),
     {:ok, profile} <- fetch_profile(user),
     {:ok, avatar}  <- fetch_avatar(profile) do
  {:ok, render(user, profile, avatar)}
else
  {:error, :not_found} -> {:error, "user not found"}
  {:error, reason}     -> {:error, reason}
end
```

**Advantages**: Clean left-to-right composition. The `else` block handles all error cases in one place. No nesting.

**Disadvantages**: All intermediate values must follow the `{:ok, _}/{:error, _}` convention. Can't easily add context at each step.

### Comparison Table

| Pattern | Verbosity | Visibility | Composability | Forgettability |
|---------|-----------|------------|---------------|----------------|
| Manual check (Go) | Very high | Maximum | Low | High (can ignore) |
| Operator (Rust `?`, lx `^`) | Low | High (operator visible) | High | Low (type system) |
| Exception bubbling | Minimal | None (invisible) | Low | High (forget to catch) |
| Monadic bind | Low | High (type signature) | Maximum | None (type-enforced) |
| `with` expression | Medium | High | Medium | Low (pattern-enforced) |

Sources:
- [Rust ? operator](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
- [Go error handling patterns](https://go.dev/blog/go1.13-errors)
- [Elixir with expression](https://hexdocs.pm/elixir/try-catch-and-rescue.html)
- [Haskell ExceptT](https://hackage.haskell.org/package/mtl/docs/Control-Monad-Except.html)

---

## 2. Error Context and Wrapping

Adding context to errors as they propagate through layers is critical for debuggability.

### Rust — anyhow `.context()`

```rust
let config = fs::read_to_string("config.toml")
    .context("reading config file")?;

let parsed: Config = toml::from_str(&config)
    .context("parsing config file")?;
```

Each `.context()` call wraps the error with an additional message, creating a chain: `"parsing config file: invalid key at line 3: expected string"`. The chain is displayed top-down (outermost context first).

**anyhow vs thiserror**: anyhow erases the concrete error type — you get a displayable chain but can't match on variants. thiserror preserves types via `#[from]` — you can match but must define explicit variants for every error source.

### Go — `fmt.Errorf` with `%w`

```go
return fmt.Errorf("loading user %d: %w", id, err)
```

Creates a chain navigable via `errors.Is`/`errors.As`/`errors.Unwrap`. The `%w` verb is critical — `%v` creates a new error with the message but breaks the chain.

**Wrapping discipline**: Only wrap errors you're willing to make part of your API. Using `%v` instead of `%w` deliberately hides implementation details from callers.

### Python — Exception Chaining

```python
try:
    data = json.loads(raw)
except json.JSONDecodeError as e:
    raise ConfigError("invalid config format") from e
```

**Two chaining modes**:
- **Explicit** (`raise X from Y`): Sets `__cause__`, suppresses `__context__` display
- **Implicit** (exception during handling): Sets `__context__`, displayed as "During handling of the above exception, another exception occurred"
- **Suppression** (`raise X from None`): Hides the original exception from display

### Java — Exception Wrapping

```java
try { connectToDb(); }
catch (SQLException e) {
    throw new ServiceException("database unavailable", e);
}
// Later: exception.getCause() returns the original SQLException
```

The `getCause()` chain can be arbitrarily deep. `printStackTrace()` shows the full chain with "Caused by:" prefixes.

### lx — Current State

lx currently propagates errors without structured context. When `^` triggers a `Propagate`, the error value and span are preserved, but no intermediate context is added.

```
-- lx today
result = some_operation () ^   -- if Err, propagates the raw error value
```

**Potential improvement**: A context-adding operator or method:

```
-- Hypothetical
result = some_operation () ^ "loading user config"
-- or
result = (some_operation ()) .context "loading user config" ^
```

### Context Wrapping Patterns

| Pattern | Mechanism | Chain inspection | Type preservation |
|---------|-----------|-----------------|-------------------|
| Rust anyhow | `.context()` method | `chain()` iterator | No (erased) |
| Rust thiserror | `#[from]` attribute | Match on variants | Yes |
| Go | `fmt.Errorf("%w")` | `errors.Is/As/Unwrap` | Partial |
| Python | `raise X from Y` | `__cause__/__context__` | Yes |
| Java | Constructor with cause | `getCause()` chain | Yes |
| lx | (none currently) | N/A | N/A |

Sources:
- [anyhow context](https://docs.rs/anyhow/latest/anyhow/)
- [Go error wrapping](https://go.dev/blog/go1.13-errors)
- [PEP 3134 - Exception Chaining](https://peps.python.org/pep-3134/)

---

## 3. Error Recovery Strategies

### Default/Fallback Values

The simplest recovery: substitute a value when an operation fails.

```rust
// Rust
let port = env::var("PORT").unwrap_or("8080".to_string());
let port: u16 = port.parse().unwrap_or(8080);
```

```
-- lx
port = (env.get "PORT") ?? "8080"
config = (load_config path) ?? default_config
```

lx's `??` coalescing operator handles this cleanly — it unwraps `Ok`/`Some` values and evaluates the fallback for `Err`/`None`. This is equivalent to Rust's `.unwrap_or_else()`, C#'s `??`, JavaScript's `??`, and Kotlin's `?:`.

**Comparison of coalescing operators**:

| Language | Operator | Applies to | Evaluates RHS |
|----------|----------|-----------|---------------|
| lx | `??` | Result/Maybe | Lazily (on Err/None) |
| C# | `??` | Nullable types | Lazily |
| JavaScript | `??` | null/undefined | Lazily |
| Kotlin | `?:` (Elvis) | Nullable types | Lazily |
| Swift | `??` | Optionals | Lazily |
| Rust | `.unwrap_or_else(\|\| ...)` | Result/Option | Lazily (closure) |
| Rust | `.unwrap_or(...)` | Result/Option | Eagerly |

**Key difference**: lx's `??` works on both `Result` and `Maybe`, while most other languages only coalesce nullability. This is more powerful — it covers both "absent" and "failed" cases with one operator.

### Retry

Re-attempt a failed operation, typically with backoff.

```
-- lx retry pattern (from pkg/core/retry.lx)
retry {max: 3  backoff: "exponential"} {
    fetch_data url ^
}
```

**When to retry**: Transient failures — network timeouts, rate limits, temporary unavailability. lx's structured agent errors (`Timeout`, `RateLimited`, `Upstream` with code >= 500) are designed for exactly this discrimination:

```
-- lx retry decision
retry_decision = (e) e ? {
    Timeout _      -> true
    RateLimited _  -> true
    Upstream info  -> info.code >= 500
    _              -> false
}
```

**When NOT to retry**: Permanent failures — invalid input, permission denied, budget exhausted. Retrying these wastes resources and can amplify damage.

### Circuit Breaker

Stop attempting operations after repeated failures to prevent cascade failures:

1. **Closed**: Normal operation, errors counted
2. **Open**: All calls fail immediately without attempting the operation
3. **Half-open**: Periodically allow one call through to test recovery

Relevant to lx's agent model: an agent calling an external service should circuit-break after repeated `Upstream` errors rather than consuming budget on doomed requests.

### Compensation (Sagas)

For multi-step operations, undo previously completed steps when a later step fails:

```
-- lx saga pattern (from pkg/core/saga.lx)
saga {
    step "create_user"    {do: create_user    undo: delete_user}
    step "setup_billing"  {do: setup_billing  undo: cancel_billing}
    step "send_welcome"   {do: send_email     undo: noop}
}
```

Each step defines a compensating action. If step N fails, steps N-1 through 1 are compensated in reverse order. This is the distributed systems equivalent of a database transaction rollback.

### Pattern Matching Recovery (lx/Rust/Elixir)

Match on specific error variants to apply targeted recovery:

```
-- lx
result = fetch_data url
recovered = result ? {
    Ok data         -> data
    Err e -> e ? {
        Timeout info    -> fetch_from_cache url
        RateLimited info -> {
            sleep info.retry_after_ms
            fetch_data url
        }
        _ -> Err e
    }
}
```

This is lx's primary recovery mechanism and it maps naturally to the structured agent error variants.

### Common Lisp Restart-Based Recovery

The most flexible recovery model — low-level code provides multiple recovery strategies, high-level code chooses:

```lisp
;; Low-level: "here's how I CAN recover"
(restart-case (parse-entry text)
  (skip-entry () nil)
  (use-value (v) v)
  (retry () (parse-entry (fix-encoding text))))

;; High-level: "here's how you SHOULD recover"
(handler-bind ((parse-error
                (lambda (c) (invoke-restart 'skip-entry))))
  (process-all-entries))
```

**Key insight**: This separates mechanism (how to recover) from policy (when to recover). Traditional try/catch conflates both at the catch site.

**Relevance to lx**: Agent orchestration is a natural fit for restart-like patterns. An agent encountering an error could offer restarts: "I can retry with different parameters," "I can skip this subtask," "I can use cached results." The orchestrator chooses the strategy based on budget, deadline, and priority.

Sources:
- [Beyond Exception Handling: Conditions and Restarts](https://gigamonkeys.com/book/beyond-exception-handling-conditions-and-restarts)
- [Elixir error handling](https://hexdocs.pm/error_message/error_handling_in_elixir.html)

---

## 4. Stack Traces and Diagnostics

### Capture Cost

Stack traces are expensive to capture. Languages differ significantly in when and how they capture them:

| Language | When captured | Overhead |
|----------|--------------|----------|
| Java | At exception construction (`new Exception()`) | High — walks entire stack |
| Python | At raise time | High — builds traceback chain |
| Rust (backtrace) | On demand (`std::backtrace::Backtrace`) | High — disabled by default |
| Rust (span) | At compile time (source location) | Zero — embedded in binary |
| Go | Never (by convention) | Zero |
| OCaml | At raise time (skippable with `raise_notrace`) | 55 vs 25 cycles |
| Zig | At compile time (error return traces in debug) | Zero in release |

### Design Tension

Full stack traces aid debugging but cost performance. Approaches:

**Eager capture** (Java, Python): Always available for debugging, but expensive even when not needed. Java's `fillInStackTrace()` is a measurable performance cost on hot exception paths.

**Lazy/on-demand capture** (Rust): `std::backtrace::Backtrace::capture()` only captures when `RUST_BACKTRACE=1`. Zero cost when disabled. anyhow captures backtraces automatically when enabled.

**Structured context instead of traces** (Go, lx): Instead of a stack trace, each layer adds context as a string. The resulting error message reads top-down: `"loading dashboard: fetching user 42: connection refused"`. Cheaper than stack traces and often more useful — shows the semantic path, not the mechanical call stack.

**Source spans** (Rust, lx): Embed source location at compile time. lx's `LxError` variants carry `SourceSpan` which points to the exact source position. This is free at runtime and provides precise error locations.

### Recommendation for lx

lx already uses source spans (`SourceSpan` in `LxError`). This is the right base strategy for a scripting language. Stack traces are less useful when the "call stack" includes interpreter frames that are meaningless to the lx programmer.

Consider adding: structured error context chains (like anyhow) that record the semantic path through lx code, not the Rust interpreter stack.

---

## 5. Panic vs. Error — The Recoverable/Unrecoverable Divide

Every language with Result-like types must draw a line between "errors the caller handles" and "bugs that crash the program."

### Rust's Guidelines

| Use `Result` when... | Use `panic!` when... |
|---------------------|---------------------|
| Failure is expected (file not found, network timeout) | A contract/invariant is violated |
| The caller can reasonably recover | Continuing would be unsafe or meaningless |
| The failure is part of normal operation | It's a programmer bug (index out of bounds) |
| You're writing a library | You're in test code (unwrap/expect is fine) |

**Special case**: `unwrap()`/`expect()` in prototyping and tests is acceptable — they convert `Result` to `panic!` and are intended for cases where failure indicates a bug.

### Go's Guidelines

`panic()` is for truly unrecoverable situations (nil pointer, index out of range). `error` is for everything else. `recover()` can catch panics but is rarely used outside of frameworks.

Go's convention is stricter than Rust's: almost everything should be an `error`. Panics indicate programming bugs that should be fixed, not caught.

### lx's Position

lx currently has:
- `Result`/`Maybe` for recoverable errors
- `assert` for invariant violations (produces `LxError::Assert`)
- No explicit panic mechanism

For a scripting language targeting agent workflows, the division should be:
- **Result/Maybe**: Expected failures (API errors, validation failures, missing data)
- **Assert**: Programming bugs, invariant violations — these should crash the current agent
- **Agent restart**: The supervisor equivalent — failed agents are restarted by the orchestrator

Sources:
- [To panic! or Not to panic!](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)
- [No-Panic Rust](https://blog.reverberate.org/2025/02/03/no-panic-rust.html)

---

## 6. Error Handling in Async Contexts

### The Problem

Async/concurrent execution creates unique error handling challenges:
1. **Where does the error go?** A failed task's error can't propagate to the spawning function if it's already moved on.
2. **Multiple concurrent errors**: When running tasks in parallel, multiple can fail simultaneously.
3. **Cancellation**: Is cancellation an error? How does it interact with cleanup?

### Language Solutions

**Rust (tokio/async-std)**: `Future<Output = Result<T, E>>` — futures carry their errors in the return type. `JoinHandle::await` returns `Result<Result<T, E>, JoinError>` — two layers: join failure (panic in task) and task-level error. `try_join!` runs futures concurrently, returning the first error.

**JavaScript**: `Promise.allSettled()` collects all results regardless of failure. `Promise.all()` fails fast on first rejection. `AggregateError` wraps multiple errors.

**Elixir/Erlang**: Process isolation means async errors are naturally contained. A spawned process's crash is observed via monitors/links, not via return values. The supervisor handles restart.

**Scala**: `Future` has `recover`, `recoverWith`, `fallbackTo` for handling async errors. `Future.sequence` converts `List[Future[T]]` to `Future[List[T]]`, failing on first error.

### Patterns for Concurrent Error Collection

**Fail-fast**: Stop on first error, cancel remaining work. Best for dependent operations where a single failure invalidates everything.

**Collect-all**: Run all operations, collect successes and failures separately. Best for independent operations (batch processing, fan-out).

**Partial success**: Accept results from operations that succeeded, report failures separately. Best for user-facing operations where partial results are better than nothing.

### lx's Async Error Model

lx agents are isolated processes (like Erlang). An agent that fails returns an `Err` result to its caller. When multiple agents run concurrently (`fan_out`), the orchestrator collects all results:

```
-- lx concurrent error handling
results = fan_out tasks
successes = results |> filter ok?
failures = results |> filter err?
```

This is the collect-all pattern. The orchestrator decides the policy — fail if any failed, proceed with partial results, retry failures, etc.

Sources:
- [Scala Futures](https://docs.scala-lang.org/overviews/core/futures.html)
- [Mastering Asynchronous JavaScript](https://dev.to/kelvinguchu/mastering-asynchronous-javascript-promises-asyncawait-error-handling-and-more-41ph)

---

## 7. Null/None Safety and Coalescing

### The Problem

Tony Hoare called null references his "billion-dollar mistake." Languages differ in how they address this:

| Approach | Languages | Mechanism |
|----------|-----------|-----------|
| Nullable by default | C, Java, Python, JS | All references can be null |
| Optional types | Rust, Haskell, Swift, lx | Explicit wrapper, must unwrap |
| Nullable annotations | Kotlin, TypeScript | Type-level null tracking |
| No null at all | Haskell (pure) | Bottom/undefined only |

### Coalescing Operators

**lx's `??`** operates on `Result` and `Maybe`:
```
-- On Maybe
name = (get_name user) ?? "anonymous"

-- On Result
config = (load_config path) ?? default_config

-- On non-Result/Maybe values: passes through unchanged
x = 42 ?? 0   -- x = 42 (42 is not Err or None, so passes through)
```

This is more general than most null coalescing operators, which only handle null/nil/undefined. lx's `??` handles both "absent" (None) and "failed" (Err) cases.

**Comparison**:
- **C# `??`**: `value ?? default` — only for nullable types
- **JavaScript `??`**: `value ?? default` — only for null/undefined (not falsy values, unlike `||`)
- **Kotlin `?:`**: `value ?: default` — for nullable types, but can also throw (`value ?: throw ...`)
- **Swift `??`**: `optional ?? default` — for optionals
- **Rust**: No operator — uses `.unwrap_or(default)` or `.unwrap_or_else(|| default)`

### Optional Chaining

Many languages pair coalescing with optional chaining (safe navigation):

| Language | Syntax | Behavior |
|----------|--------|----------|
| JavaScript | `obj?.prop?.method()` | Short-circuit to undefined |
| Kotlin | `obj?.prop?.method()` | Short-circuit to null |
| Swift | `obj?.prop?.method()` | Short-circuit to nil |
| C# | `obj?.Prop?.Method()` | Short-circuit to null |
| Rust | `option.as_ref()?.field` | Short-circuit via `?` on Option |

**lx consideration**: lx currently uses `??` for coalescing but doesn't have optional chaining syntax. Field access on `None`/`Err` is a runtime error. Optional chaining (`user?.name ?? "anon"`) could reduce boilerplate in agent workflows where many values are optional.

Sources:
- [Null coalescing operator - Wikipedia](https://en.wikipedia.org/wiki/Null_coalescing_operator)
- [C# ?? and ??= operators](https://learn.microsoft.com/en-us/dotnet/csharp/language-reference/operators/null-coalescing-operator)
- [Elvis operator - Wikipedia](https://en.wikipedia.org/wiki/Elvis_operator)

---

## 8. Error Types and Hierarchies

### Enum-Based (Rust, lx)

```rust
enum AppError {
    Io(std::io::Error),
    Parse(ParseError),
    NotFound { resource: String },
}
```

**Advantages**: Exhaustive matching. Compiler verifies all variants are handled. Clear, finite set of possible errors.

**Disadvantages**: Adding a variant is a breaking change. Can lead to "god enums" with dozens of variants.

**lx's approach**: Agent errors are variant types (`Timeout`, `RateLimited`, `BudgetExhausted`, etc.) that can be pattern-matched:

```
handle_err = (e) e ? {
    Timeout info        -> "timeout: {info.elapsed_ms}ms"
    RateLimited info    -> "rate limited: wait {info.retry_after_ms}ms"
    BudgetExhausted info -> "budget: {info.resource}"
    _                   -> "other error"
}
```

### Class-Based (Python, Java)

```python
class AppError(Exception): pass
class NotFoundError(AppError): pass
class ValidationError(AppError):
    def __init__(self, field, message):
        self.field = field
        super().__init__(f"{field}: {message}")
```

**Advantages**: Inheritance allows catching broad categories (`except AppError`). Rich objects with methods and data.

**Disadvantages**: Deep hierarchies become unwieldy. Multiple inheritance creates diamond problems. Catching base classes can over-catch.

### String-Based (Go)

```go
var ErrNotFound = errors.New("not found")

// Or with context:
return fmt.Errorf("user %d: %w", id, ErrNotFound)
```

**Advantages**: Simple. No type hierarchy to maintain.

**Disadvantages**: Matching on string content is fragile. No structured data. Sentinel errors are just string constants.

Go mitigates this with custom error types (`struct` implementing `error` interface) and `errors.As` for type-safe matching.

### Error Type Design Guidelines

1. **Granularity**: One variant per distinct recovery action. If two errors require the same recovery, they can be the same variant. If they require different recovery, they must be different.

2. **Context data**: Include data the caller needs to recover. `Timeout{elapsed_ms, deadline_ms}` lets the caller decide whether to retry with a longer deadline. Just `"timeout"` forces the caller to guess.

3. **Hierarchy depth**: Flat is almost always better. Rust and Go both favor flat error enums/types. Deep hierarchies (Java's exception tree) create classification ambiguity.

4. **Open vs. closed**: Enum-based errors are closed sets (exhaustive matching). Class-based errors are open (new subclasses can be added). For libraries, closed sets are safer (compiler catches missing cases). For application code, open sets are more flexible.

5. **Machine-readable vs. human-readable**: Error types should carry machine-readable data (codes, durations, resource names) AND produce human-readable messages. Separate the two concerns — the type carries data, a `Display`/`to_string` implementation produces the message.

Sources:
- [Dave Cheney - Don't just check errors](https://dave.cheney.net/2016/04/27/dont-just-check-errors-handle-them-gracefully)
- [Composable Error Handling in OCaml](https://keleshev.com/composable-error-handling-in-ocaml)

---

## 9. Design Trade-offs for Scripting Languages

### The Core Tension

Scripting languages prioritize **developer ergonomics** — fast iteration, minimal boilerplate, forgiving semantics. Error-as-values prioritize **safety** — explicit handling, no forgotten errors, compiler enforcement. These goals conflict.

### How Scripting Languages Handle Errors Today

| Language | Primary model | Safety level | Ergonomics |
|----------|--------------|-------------|------------|
| Python | Exceptions | Low (can ignore) | High (minimal syntax) |
| Ruby | Exceptions | Low | High |
| Lua | pcall/xpcall (protected calls) | Low | Medium |
| JavaScript | Exceptions + Promises | Low | Medium |
| Elixir | Tagged tuples | Medium | High (with/pattern match) |
| **lx** | **Result/Maybe + ^/??** | **Medium** | **High** |

### lx's Unique Position

lx is unusual: a scripting language with typed error values. This combination is almost nonexistent. Most scripting languages chose exceptions for their lower ceremony. lx chose error values because:

1. **Agent isolation**: Each agent is a separate execution context. Exceptions don't cross agent boundaries naturally — but error values do (they're just data).
2. **Pattern matching**: lx has rich pattern matching on variants. This makes error handling expressive without try/catch ceremony.
3. **Composability**: Concurrent agent results need to be collected and filtered. Error values compose; exceptions don't.

### The `^` vs. `try/catch` Ergonomics Comparison

```
-- lx: error propagation
result = fetch_user id ^           -- propagate Err
profile = fetch_profile user ^     -- propagate Err
```

```python
# Python: exception propagation
try:
    result = fetch_user(id)        # raises on error
    profile = fetch_profile(user)  # raises on error
except UserError as e:
    handle(e)
```

The `^` approach requires one character per fallible call. The try/catch approach requires zero characters per call but needs a surrounding block. For chains of 2-3 calls, they're equivalent. For 10+ calls, try/catch is less noisy. For mixed success/error handling, `^` is more flexible because you can handle individual errors inline.

### The `??` vs. `.unwrap_or_default()` Ergonomics

```
-- lx
name = (get_name user) ?? "anonymous"

-- Rust (no operator, must use method)
let name = get_name(&user).unwrap_or("anonymous".to_string());

-- JavaScript
const name = getName(user) ?? "anonymous";  // but only for null/undefined

-- Python (no equivalent — must use try/except or conditional)
name = get_name(user) if get_name(user) is not None else "anonymous"
```

lx's `??` is more concise than Rust and more powerful than JavaScript (handles both Result and Maybe, not just null).

### Checked vs. Unchecked in Scripting Context

Java's checked exceptions lesson: forced handling creates verbosity that drives developers to write `catch (Exception e) {}` — worse than no checking at all.

**lx's approach**: Error handling is unchecked at the language level (no compiler enforcement) but the conventions strongly encourage it:
- Functions returning `Result` are visually distinct
- `^` makes propagation explicit and visible
- `??` makes fallback handling concise
- Pattern matching on error variants is idiomatic

This is the scripting-language sweet spot: conventional, not enforced. Like Elixir's `{:ok, _}/{:error, _}` pattern — universally followed but not compiler-checked.

### Zig's `errdefer` for Scripting

Zig's `errdefer` (cleanup only on error return) is compelling for scripting:

```zig
fn init() !*Resource {
    const r = try allocate();
    errdefer deallocate(r);      // only if we return an error
    try r.configure();
    return r;
}
```

**Potential lx equivalent**:
```
-- Hypothetical
init = () {
    r = allocate () ^
    errdefer { deallocate r }    -- only runs if this function returns Err
    configure r ^
    r
}
```

This would be valuable for agent workflows that acquire resources (connections, file handles, API sessions) and need cleanup only on failure.

Sources:
- [Zig error handling](https://zig.guide/language-basics/errors/)
- [Lua error handling](https://www.lua.org/pil/8.4.html)
- [Errors Are Not Exceptions](https://dev.to/swyx/errors-are-not-exceptional-1g0b)
- [Away from Exceptions: Errors as Values](https://news.ycombinator.com/item?id=28337112)

---

## 10. Summary: What lx Should Take From Each

| Source | Pattern | Applicability to lx |
|--------|---------|---------------------|
| **Rust** | `?` operator, From trait, thiserror/anyhow split | Already adopted (`^`). Consider error conversion traits and context wrapping. |
| **Go** | `%w` wrapping, `errors.Is`/`errors.As` | Error chain inspection could complement pattern matching. |
| **Haskell** | Monadic bind for error sequencing | lx's `with`-like patterns could be formalized. |
| **Elixir** | `with` expression, let-it-crash | Agent supervision is lx's equivalent. `with`-like multi-step matching could reduce nesting. |
| **Common Lisp** | Condition/restart separation | Agent restarts: let agents define recovery strategies, let orchestrators choose. This is the most promising unexplored pattern. |
| **Algebraic effects** | Resumable effects, effect handlers | lx agent tool calls are already effects. Making this explicit enables composition. |
| **Zig** | `errdefer`, error set coercion | `errdefer` for resource cleanup on error. Error set concepts for agent error categorization. |
| **Swift** | `try?` (convert to Optional) | Consider `^?` or similar: convert Result to Maybe, discarding the error details. |
| **OCaml** | Dual system (exceptions + Result) | lx already does this implicitly (assert for bugs, Result for expected errors). |

### Open Design Questions for lx

1. **Error context**: Should `^` accept an optional context string? (`operation () ^ "while loading config"`)
2. **Error conversion**: Should lx have a `From`-like protocol for automatic error type conversion?
3. **Condition/restart for agents**: Can agent error handling be restructured as condition/restart? Agents define restarts (retry, skip, fallback), orchestrators bind handlers.
4. **errdefer**: Is cleanup-on-error-only a common enough pattern in lx workflows to warrant syntax?
5. **Optional chaining**: Should `user?.name` short-circuit to `None` instead of erroring?
6. **Concurrent error collection**: Should `fan_out` have built-in strategies (fail-fast, collect-all, partial-success)?
