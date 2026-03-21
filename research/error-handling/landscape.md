# Error Handling Across Programming Languages

A comprehensive survey of error handling models, covering eight major paradigms with technical depth sufficient to inform lx language design decisions. lx uses `Result`/`Maybe` types with `^` propagation and `??` coalescing — a Rust-inspired model adapted for scripting.

---

## 1. C — The Baseline

C provides no built-in error handling. Every strategy is ad hoc and convention-based.

**Return codes**: Functions return an integer status (0 = success, nonzero = error). Callers must check every return value manually. Nothing enforces this — unchecked errors silently propagate corruption.

```c
int fd = open("file.txt", O_RDONLY);
if (fd < 0) {
    perror("open");        // consult errno
    return -1;
}
```

**errno**: A thread-local global integer set by libc functions on failure. Must be read immediately — the next successful call may overwrite it. Fragile and easy to misuse.

**setjmp/longjmp**: C's only non-local transfer of control. `setjmp` saves a stack context; `longjmp` restores it, effectively implementing a primitive throw/catch. No destructors run, no resources are freed — guaranteed leak source. Used almost exclusively in legacy codebases and some embedded interpreters.

**Key lesson for lx**: C demonstrates why implicit error handling (unchecked returns, global state) leads to bugs. The absence of language support is itself a design choice with severe consequences.

Sources:
- [Error handling across different languages](https://blog.frankel.ch/error-handling/)

---

## 2. Exceptions (try/catch/finally)

### Core Semantics

Exceptions separate the error path from the happy path via stack unwinding. When a function throws, the runtime walks up the call stack until it finds a matching catch handler, destroying stack frames as it goes.

**Three components**:
1. **Throw/raise**: Creates an exception object and initiates stack unwinding
2. **Catch/rescue/except**: Pattern-matches on exception types to handle specific failures
3. **Finally/ensure/after**: Cleanup code that runs regardless of success or failure

### Language Variants

**Java — Checked vs. Unchecked Exceptions**

Java's most controversial design decision: checked exceptions appear in method signatures and the compiler verifies they are caught or declared.

```java
// Checked — must declare or handle
public void readFile(String path) throws IOException {
    FileInputStream f = new FileInputStream(path);  // throws IOException
}

// Unchecked — RuntimeException subclasses bypass checking
public int divide(int a, int b) {
    return a / b;  // ArithmeticException — not checked
}
```

The exception hierarchy: `Throwable` -> `Error` (unrecoverable: `OutOfMemoryError`) and `Exception` (recoverable). `RuntimeException` subclasses are unchecked; everything else under `Exception` is checked.

**The verbosity problem**: Checked exceptions lead to "catch-and-rethrow chains" where intermediate code wraps exceptions just to satisfy the compiler, obscuring the original error. This is widely considered a design failure — Kotlin, Scala, C#, and every major language since has rejected checked exceptions.

**Exception chaining**: `getCause()` returns the wrapped original exception, enabling `throw new ServiceException("failed", originalException)` patterns.

**Python — Exception Chaining**

Python provides two chaining mechanisms:
- **Implicit chaining** (`__context__`): When a new exception is raised during handling of another, the original is automatically preserved
- **Explicit chaining** (`__cause__` via `raise X from Y`): Developer explicitly links exceptions. Setting `__cause__` also sets `__suppress_context__ = True`

```python
try:
    connect_to_db()
except ConnectionError as e:
    raise ServiceError("database unavailable") from e  # __cause__ = e
```

Display behavior: `__cause__` is always shown when present. `__context__` is shown only when `__cause__` is None and `__suppress_context__` is False.

**JavaScript** — Untyped catch (catches everything), `finally` for cleanup, `Promise.catch()` for async errors. No exception hierarchy in the language spec; convention-based `Error` subtypes.

**C++** — Zero-cost on the happy path (table-based unwinding), but throwing is expensive (stack unwinding + RTTI). `noexcept` functions abort on throw. No `finally` — uses RAII instead.

### Performance Characteristics

The performance story is counterintuitive:

- **Happy path**: Exceptions have near-zero overhead (table-based unwinding means no branch checks)
- **Error path**: Expensive — stack unwinding, RTTI matching, memory allocation for exception objects
- **Error values**: Constant small overhead on every call (branch check), no overhead difference on error vs. success

CedarDB benchmarks show exceptions at 7.7ms vs. error returns at 37ms for 10,000 iterations of Fibonacci(15) — exceptions were ~5x faster because the happy path dominates. However, this depends on error frequency: at high error rates, exception overhead dominates.

OCaml benchmarks: raising an exception costs ~55 cycles; `raise_notrace` (skip backtrace) costs ~25 cycles; unused try/catch blocks cost ~6 cycles.

### Fundamental Trade-offs

| Dimension | Exceptions | Error Values |
|-----------|-----------|--------------|
| Happy path cost | Near-zero | Branch check per call |
| Error path cost | Expensive (unwinding) | Same as happy path |
| Forgettability | Easy to forget to catch | Easy to forget to check |
| Composability | Poor (single current exception) | Excellent (values compose) |
| Async compatibility | Problematic (cross-thread) | Natural |
| Control flow visibility | Implicit (hidden jumps) | Explicit |

Sources:
- [Exceptions vs Error Values](https://programmingduck.com/articles/exceptions-error-values)
- [Why I Prefer Exceptions to Error Values](https://cedardb.com/blog/exceptions_vs_errors/)
- [Java Exception Hierarchy](https://dev.to/noel_kamphoa_e688aece0725/exception-hierarchy-in-java-checked-unchecked-and-errors-1f45)
- [PEP 3134 - Exception Chaining](https://peps.python.org/pep-3134/)
- [Java Unchecked Exceptions Controversy](https://docs.oracle.com/javase/tutorial/essential/exceptions/runtime.html)

---

## 3. Result/Option Types

### Rust — The Gold Standard

Rust's error handling is the most mature implementation of typed error values in a systems language.

**Core types**:
```rust
enum Result<T, E> {
    Ok(T),
    Err(E),
}

enum Option<T> {
    Some(T),
    None,
}
```

**The `?` operator**: Syntactic sugar for early-return propagation. On `Ok`/`Some`, unwraps the value. On `Err`/`None`, returns from the enclosing function immediately.

```rust
// ? expands to:
let file = match File::open("x.txt") {
    Ok(f) => f,
    Err(e) => return Err(e.into()),  // note: calls From::from()
};

// Concise form:
let file = File::open("x.txt")?;
```

**`From` trait for error conversion**: The `?` operator calls `From::from()` on the error, enabling automatic conversion between error types. This is what makes `?` work across different error types within the same function.

```rust
impl From<io::Error> for MyError {
    fn from(e: io::Error) -> Self { MyError::Io(e) }
}
// Now ? on io::Result automatically converts to MyError
```

**Ecosystem crates**:
- **thiserror** (libraries): Derive macro for implementing `Display`, `Error`, and `From` on error enums. Generates boilerplate for structured error types.
- **anyhow** (applications): Dynamic error type (`anyhow::Error`) that erases the concrete type. Provides `.context("msg")` for wrapping errors with additional information. Best for application-level code where callers don't match on error variants.
- **snafu**: Alternative to thiserror with context selectors and backtrace support.

**Design pattern**: Libraries define typed errors with thiserror (callers can match on variants). Applications use anyhow to aggregate errors (callers only display/log them).

**Combinators**: `map`, `map_err`, `and_then`, `unwrap_or_else`, `unwrap_or_default` — functional composition of fallible operations.

**panic! vs. Result**: `panic!` is for unrecoverable errors (violated invariants, programmer bugs). Result is for expected failures. The rule: if a caller can reasonably recover, use Result. If it's a bug, panic.

### Haskell — Monadic Error Handling

Haskell uses the same core types but embeds them in monadic composition:

```haskell
-- Maybe a = Nothing | Just a
-- Either e a = Left e | Right a

safeDivide :: Int -> Int -> Either String Int
safeDivide _ 0 = Left "division by zero"
safeDivide x y = Right (x `div` y)
```

**Monadic binding** (`>>=`): Sequences operations that may fail, short-circuiting on the first `Left`/`Nothing`:

```haskell
lookupUser :: UserId -> Either Error User
lookupUser uid =
    getRecord uid >>= validateAge >>= checkPermissions
```

**ExceptT monad transformer**: Adds exception-like behavior to any monad stack. `throwError` and `catchError` from `MonadError` class provide throw/catch semantics within the type system:

```haskell
type App a = ExceptT AppError IO a
-- Combines IO effects with typed error handling
```

**Key insight**: Haskell proves that error handling can be entirely within the type system, with no runtime mechanism needed. The `do` notation makes monadic error handling read like imperative code.

### Swift — Hybrid Approach

Swift combines exceptions with value types:

```swift
// Exception-style (untyped until Swift 6)
func load() throws -> Data { ... }
let data = try load()          // propagate
let data = try? load()         // convert to Optional
let data = try! load()         // force-unwrap (crash on error)

// Swift 6: Typed throws
func fetch() throws(NetworkError) -> Data { ... }
// Compiler knows exactly which errors are possible
```

**Swift 6 typed throws**: Functions declare specific error types. In `do-catch` blocks with typed throws, the `error` variable has the concrete type rather than `any Error`. This reduces `as?` casting boilerplate.

**Result type**: `Result<Success, Failure>` exists in the standard library for cases where exceptions don't fit (async callbacks, stored results).

**Design philosophy**: Untyped throws is recommended for most code. Typed throws targets performance-critical and embedded contexts where existential boxing overhead matters.

### OCaml — Dual System

OCaml uniquely supports both exceptions and Result types as first-class error handling:

```ocaml
(* Exception style *)
let parse s =
  if String.length s = 0 then raise Empty_input
  else ...

(* Result style with let* binding operator *)
let (let*) = Result.bind
let process input =
  let* parsed = parse input in
  let* validated = validate parsed in
  Ok (transform validated)
```

**Recommendation**: Use Result/Option for foreseeable, ordinary errors in production code. Use exceptions for rare, truly exceptional conditions. Bridge between them with `Option.try_with` and `Or_error.try_with`.

**Performance**: Exception handlers cost ~6 cycles when unused. Raising costs ~55 cycles (25 with `raise_notrace`). Result matching is essentially free.

Sources:
- [Rust Error Handling - The Rust Programming Language](https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html)
- [To panic! or Not to panic!](https://doc.rust-lang.org/book/ch09-03-to-panic-or-not-to-panic.html)
- [Rust Error Handling Compared: anyhow vs thiserror vs snafu](https://dev.to/leapcell/rust-error-handling-compared-anyhow-vs-thiserror-vs-snafu-2003)
- [Error Handling - Real World OCaml](https://dev.realworldocaml.org/error-handling.html)
- [Swift 6 Typed Throws](https://www.hackingwithswift.com/swift/6.0/typed-throws)
- [Haskell MonadError](https://hackage.haskell.org/package/mtl/docs/Control-Monad-Error-Class.html)
- [Swift typed throws proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0413-typed-throws.md)

---

## 4. Multiple Return Values — Go

Go chose the simplest possible error model: functions return `(value, error)` tuples.

```go
f, err := os.Open("file.txt")
if err != nil {
    return fmt.Errorf("opening config: %w", err)
}
```

### Core Design

- **`error` interface**: Any type implementing `Error() string` is an error. No hierarchy, no special base type.
- **No automatic propagation**: Every error must be explicitly checked. This is by design — Go's creators believe implicit propagation hides control flow.
- **Convention-enforced**: The language doesn't force you to check errors. `f, _ := os.Open(...)` silently discards the error.

### Error Wrapping (Go 1.13+)

**`fmt.Errorf` with `%w`**: Wraps an error while preserving the chain for later inspection.

```go
return fmt.Errorf("reading config for %s: %w", name, err)
```

**`errors.Is(err, target)`**: Walks the error chain checking for a specific sentinel value. Replaces `err == ErrNotFound` comparisons.

**`errors.As(err, &target)`**: Walks the chain checking for a specific type. Replaces type assertions.

**`errors.Unwrap(err)`**: Returns the next error in the chain (the wrapped error).

**Custom `Is()` and `As()` methods**: Error types can implement custom matching logic for flexible comparisons.

### Error Categories

| Category | Mechanism | Use Case |
|----------|-----------|----------|
| Sentinel errors | `var ErrNotFound = errors.New(...)` | "What happened?" |
| Custom error types | `type QueryError struct { ... }` | "What are the details?" |
| Opaque errors | `fmt.Errorf("...: %v", err)` with `%v` | Hide implementation details |

### Wrapping Guidelines

**Wrap when**: The underlying error is part of your API contract, or the caller provided the resource that failed.

**Don't wrap when**: The underlying error is an implementation detail you don't want to commit to supporting.

### Criticism

Go's error handling produces extreme verbosity — CockroachDB contains ~25,000 error-handling paths. The `if err != nil { return err }` pattern accounts for a significant fraction of Go code. Proposals for a `check` or `try` keyword have been rejected repeatedly.

Sentinel errors also have performance implications — `errors.Is` traverses the entire chain, which DoltHub benchmarked as up to 500% slower than direct comparison for deep chains.

Sources:
- [Working with Errors in Go 1.13](https://go.dev/blog/go1.13-errors)
- [Don't just check errors, handle them gracefully](https://dave.cheney.net/2016/04/27/dont-just-check-errors-handle-them-gracefully)
- [Sentinel errors benchmark](https://www.dolthub.com/blog/2024-05-31-benchmarking-go-error-handling/)
- [Go error inspection draft design](https://go.googlesource.com/proposal/+/master/design/go2draft-error-inspection.md)

---

## 5. Erlang/Elixir — Let It Crash

The BEAM VM's process isolation model enables a fundamentally different approach to error handling.

### Tagged Tuples for Expected Errors

```elixir
case File.read("hello") do
  {:ok, body}      -> process(body)
  {:error, reason} -> log_error(reason)
end
```

**Convention**: Functions return `{:ok, value}` or `{:error, reason}`. Bang variants (`File.read!`) raise exceptions on failure — used when the caller cannot recover.

### Three Error Mechanisms

| Mechanism | Purpose | Syntax |
|-----------|---------|--------|
| `raise`/`rescue` | Unexpected exceptions | `try do ... rescue e -> ... end` |
| `throw`/`catch` | Non-local returns (rare) | `try do ... catch :throw, val -> ... end` |
| `exit`/`catch` | Process termination | `try do ... catch :exit, reason -> ... end` |

### The `with` Expression

Composes multiple pattern-matched operations, short-circuiting on first mismatch:

```elixir
with {:ok, user}    <- fetch_user(id),
     {:ok, profile} <- fetch_profile(user),
     {:ok, avatar}  <- fetch_avatar(profile) do
  render(user, profile, avatar)
else
  {:error, :not_found} -> render_404()
  {:error, reason}     -> render_error(reason)
end
```

### Let It Crash Philosophy

**Core principle**: Don't write defensive code for unexpected failures. Let the process crash. A supervisor will restart it in a known-good state.

**Why this works**:
1. **Process isolation**: An unhandled exception in one process never corrupts another process's state
2. **Supervision trees**: Hierarchical supervisors monitor child processes and apply restart strategies (one-for-one, one-for-all, rest-for-one)
3. **Cheap processes**: BEAM processes cost ~2KB, so restarting is nearly free
4. **Preemptive scheduling**: A crashing process doesn't starve others

**When NOT to let it crash**: Expected, handleable errors (invalid user input, missing records) should use tuple returns. Let-it-crash is for unexpected failures where recovery logic would be speculative.

**Key insight for lx**: The BEAM model shows that error handling strategy is inseparable from execution model. Isolated, supervised processes make crash-recovery a viable primary strategy. lx's agent model has natural parallels — agents are isolated, can be supervised, and can be restarted.

Sources:
- [Elixir try/catch/rescue](https://hexdocs.pm/elixir/try-catch-and-rescue.html)
- [Let it crash - Joe Armstrong](https://dev.to/adolfont/the-let-it-crash-error-handling-strategy-of-erlang-by-joe-armstrong-25hf)
- [Error Handling in Elixir Libraries](https://michal.muskala.eu/post/error-handling-in-elixir-libraries/)
- [Errors and Exceptions - Learn You Some Erlang](https://learnyousomeerlang.com/errors-and-exceptions)

---

## 6. Common Lisp — Condition/Restart System

The most powerful error handling model ever designed. Separates error handling into three independently composable layers.

### Three-Layer Architecture

| Layer | Responsibility | Macro |
|-------|---------------|-------|
| **Signaling** | Detect problem, create condition object | `signal`, `error`, `warn` |
| **Handling** | Decide *whether* and *how* to respond | `handler-bind`, `handler-case` |
| **Restarting** | Provide concrete recovery strategies | `restart-case`, `invoke-restart` |

### Core Mechanisms

**Signaling**: Low-level code detects a problem and signals a condition. `signal` searches for handlers but returns normally if none match. `error` enters the debugger if unhandled. `warn` prints a warning if unhandled.

**handler-case** (stack-unwinding — like try/catch):
```lisp
(handler-case (parse-log-entry text)
  (malformed-log-entry-error () nil))  ; returns nil on error
```

**handler-bind** (non-unwinding — preserves stack):
```lisp
(handler-bind ((malformed-log-entry-error
                #'(lambda (c) (invoke-restart 'skip-log-entry))))
  (analyze-log log))
```

**restart-case** (establishes recovery strategies):
```lisp
(restart-case (error 'malformed-log-entry-error :text text)
  (skip-log-entry ()      ; restart 1: skip this entry
    nil)
  (use-value (value)      ; restart 2: substitute a value
    value))
```

### Separation of Policy from Mechanism

The defining feature: **restarts define HOW to recover** (mechanism), **handlers decide WHETHER to recover** (policy).

Low-level parsing code establishes restarts ("I can skip this entry, or use a substitute value"). High-level orchestration code establishes handlers ("when encountering malformed entries, skip them"). Neither needs knowledge of the other's implementation.

This separation is impossible in traditional exception handling, where the catch site must both decide to handle AND implement the recovery, forcing either:
- Recovery code in low-level functions (committed to a strategy), or
- All context lost by the time the high-level catch runs (stack already unwound)

### Stack Preservation

With `handler-bind`, the handler executes with the full call stack intact. The handler can inspect the condition, invoke any available restart, or decline to handle (let other handlers try). The stack unwinds only when/if a restart transfers control.

This means recovery code can resume execution at the point of failure, not at the catch site — something no exception system can do.

### Interactive Debugging

If no handler matches, the debugger presents available restarts to the user. The developer can choose a recovery strategy interactively, fix the problem, and continue execution — no restart needed.

**Key insight for lx**: The condition/restart model maps naturally to agent supervision. An agent encountering an error could signal a condition; the orchestrator (handler) could choose from restarts defined by the agent's runtime: retry, skip, substitute, escalate. This is more flexible than simple Result propagation.

Sources:
- [Beyond Exception Handling: Conditions and Restarts](https://gigamonkeys.com/book/beyond-exception-handling-conditions-and-restarts)
- [Common Lisp Condition System - Wikibooks](https://en.wikibooks.org/wiki/Common_Lisp/Advanced_topics/Condition_System)
- [Conditions and Restarts Tutorial](https://lisper.in/restarts)
- [Beyond Try-Catch: Common Lisp's Restart System](https://www.rangakrish.com/index.php/2026/03/06/beyond-try-catch-common-lisps-restart-system/)

---

## 7. Algebraic Effects

Algebraic effects are **"exceptions you can resume."** They generalize exceptions, generators, async/await, and coroutines into a single mechanism.

### Core Model

```
effect Ask : string -> int    // declare an effect

let program () =
  let age = perform (Ask "how old?") in    // "throw" an effect
  if age < 18 then "minor" else "adult"

// Handle the effect — can resume or not
let result = handle (program ()) with
  | effect (Ask prompt) k ->
      let answer = read_input prompt in
      continue k answer          // RESUME with a value
```

The critical distinction from exceptions: the handler receives a **continuation** `k` and can choose to:
- Resume execution with a value (`continue k answer`)
- Not resume (like a traditional exception)
- Resume multiple times (backtracking, probabilistic programming)
- Resume later (async/await)

### Language Implementations

**Koka**: Full effect system with static tracking. Every function's type includes its effects. Uses evidence passing to compile to C without runtime overhead. Effects are inferred.

```koka
effect ask
  ctl ask(prompt : string) : int

fun program() : ask int
  val age = ask("how old?")
  age
```

**OCaml 5**: Retrofitted effect handlers for concurrency. Deliberately limited to one-shot continuations (each continuation can be resumed at most once). Dynamic checks raise an exception on double-resume. Primarily designed for concurrent programming with `Eio`.

**Eff**: Research language by Matija Pretnar and Andrej Bauer. Supports multi-shot continuations (resume multiple times). Enables backtracking, probabilistic branching.

### Effects as Generalization

| Feature | As an effect |
|---------|-------------|
| Exceptions | Effect handler that doesn't resume |
| Generators/yield | Effect handler that collects values and resumes |
| Async/await | Effect handler that suspends, resumes later |
| State | Effect handler that threads state through resumes |
| Coroutines | Two effect handlers resuming each other |

This unification solves the **"what color is your function"** problem — functions don't need to be "async" or "sync", they just perform effects, and the handler decides the execution strategy.

**Key insight for lx**: Algebraic effects are the theoretical ideal for agent orchestration. An agent "performs" effects (tool calls, AI requests, spawning subagents), and the runtime handles them. lx's current architecture already resembles this — agent tool calls are effects handled by the runtime. Making this explicit could enable powerful composition patterns.

Sources:
- [Why Algebraic Effects?](https://antelang.org/blog/why_effects/)
- [OCaml effects tutorial](https://github.com/ocaml-multicore/ocaml-effects-tutorial)
- [Algebraic Effects in Modern Languages](https://tuttlem.github.io/2025/06/27/algebraic-effects-in-modern-languages.html)
- [Jane Street - Effective Programming](https://www.janestreet.com/tech-talks/effective-programming/)
- [Algebraic Handler Lookup in Koka, Eff, OCaml, and Unison](https://interjectedfuture.com/algebraic-handler-lookup-in-koka-eff-ocaml-and-unison/)

---

## 8. Zig — Explicit Error Sets

Zig takes Rust's approach and makes it even more explicit by giving errors their own type system.

### Error Sets

Named collections of error values, functioning as enumerations:

```zig
const FileOpenError = error{ AccessDenied, OutOfMemory, FileNotFound };
const AllocationError = error{ OutOfMemory };
```

Error sets automatically **coerce to their supersets** — a function returning `AllocationError` can be assigned to a variable expecting `FileOpenError` because `AllocationError` is a subset.

### Error Unions

The `!` operator creates a union of an error set with a payload type:

```zig
fn parse(input: []const u8) ParseError!AST { ... }
// Returns either a ParseError or an AST
```

**`anyerror`**: The global error set containing all errors. Equivalent to `Box<dyn Error>` in Rust. Discouraged in library APIs because it prevents compiler optimization.

### Propagation and Handling

**`try`**: Shorthand for `x catch |err| return err`. Equivalent to Rust's `?`.

**`catch`**: Evaluate a fallback on error, optionally capturing the error value:

```zig
const value = risky_function() catch |err| blk: {
    log.err("failed: {}", .{err});
    break :blk default_value;
};
```

**`errdefer`**: Like `defer`, but only executes when the function returns an error. Critical for cleanup in functions that allocate before potentially failing:

```zig
fn init() !*Resource {
    const r = try allocate();
    errdefer deallocate(r);      // only runs if we return an error
    try r.configure();           // if this fails, r is deallocated
    return r;
}
```

### Comptime Error Checking

Zig infers error sets at compile time. If a function's error set is not explicitly declared, the compiler computes the union of all possible errors. This enables:
- Dead code elimination for impossible error branches
- Compile-time verification that all errors are handled
- Error set merging with `||` operator

**Key insight for lx**: Zig's `errdefer` is a compelling pattern for resource cleanup in scripting contexts. lx could benefit from a similar construct — cleanup that only runs on error propagation, not on normal returns.

Sources:
- [Zig error handling guide](https://zig.guide/language-basics/errors/)
- [Advanced Guide to Return Values and Error Unions in Zig](https://gencmurat.com/en/posts/advanced-guide-to-return-values-and-error-unions-in-zig/)
- [Introduction to Zig - Error Handling](https://pedropark99.github.io/zig-book/Chapters/09-error-handling.html)

---

## Comparative Matrix

| Language | Model | Propagation | Type Safety | Recovery | Async compat |
|----------|-------|-------------|-------------|----------|-------------|
| C | Return codes + errno | Manual | None | Ad hoc | N/A |
| Java | Checked + unchecked exceptions | Automatic (bubbling) | Partial (checked only) | catch blocks | Poor |
| Python | Unchecked exceptions | Automatic (bubbling) | None | except blocks | try/except in async |
| Go | (value, error) tuples | Manual | None (can ignore) | if err != nil | Natural |
| Rust | Result/Option + ? | Semi-automatic (?) | Full (compiler-enforced) | match/combinators | Natural (Future<Result>) |
| Haskell | Either/Maybe + monads | Monadic bind (>>=) | Full | Monadic composition | Natural (monad stacks) |
| Swift | throws + Result | try/try?/try! | Partial (typed in Swift 6) | do-catch | async throws |
| OCaml | Exceptions + Result | Both available | Partial | match + handlers | Effect handlers |
| Erlang/Elixir | {:ok}/{:error} tuples | Manual (with) | Convention-based | Pattern matching | Process isolation |
| Common Lisp | Conditions/restarts | Signal (non-unwinding) | Dynamic | Restart invocation | N/A |
| Koka/Eff | Algebraic effects | Effect performance | Full (effect types) | Resume/don't resume | Effects ARE async |
| Zig | Error sets + unions | try/catch | Full (comptime) | catch + errdefer | N/A |
| **lx** | **Result/Maybe + ^/?? ** | **Semi-automatic (^)** | **Convention-based** | **match + ??** | **Agent isolation** |

---

## Relevance to lx

lx occupies a unique position: a scripting language with Rust-inspired error types, targeting agent orchestration. Current lx error handling:

- `Result` type: `Ok(value)` / `Err(reason)` — like Rust but dynamic
- `Maybe` type: `Some(value)` / `None` — like Rust's Option
- `^` operator: Propagates errors (returns Err from enclosing function) — like Rust's `?`
- `??` operator: Coalesces errors with fallback — like C#/JS `??` but for Result/Maybe
- Pattern matching on error variants for recovery
- Agent errors as structured variants (`Timeout`, `BudgetExhausted`, etc.)
- Agent isolation provides Erlang-like crash containment

The combination of typed error values with scripting ergonomics is rare. Most scripting languages use exceptions; most languages with Result types are compiled. lx's challenge is maintaining the safety benefits of explicit error handling while keeping the ergonomics expected in a scripting context.
