# Coroutines, Generators, and Yield Across Languages

Research for lx language design. Comprehensive survey of coroutine/generator/yield mechanisms across programming languages, their taxonomies, and implementation approaches.

---

## 1. Python: Generators and Coroutines

### Evolution Through PEPs

**PEP 255 (Python 2.2) — Simple Generators:** Introduced `yield` as a statement. A function containing `yield` returns a generator object implementing the iterator protocol (`__next__`). Each call to `next()` executes until the next `yield`, suspending the frame. The generator's locals and instruction pointer are preserved between calls.

**PEP 342 (Python 2.5) — Coroutines via Enhanced Generators:** Transformed `yield` from a statement into an expression, enabling bidirectional communication. Added three methods:
- `send(value)` — resumes the generator and makes `value` the result of the current `yield` expression. Calling `send(None)` is equivalent to `next()`. Sending a non-None value to a just-started generator raises `TypeError`.
- `throw(type, value, traceback)` — raises an exception at the generator's suspension point. If the generator catches it and yields another value, that value is returned.
- `close()` — implemented as `throw(GeneratorExit)`. If the generator yields after receiving `GeneratorExit`, a `RuntimeError` is raised. Ensures finalization via exception handling.

**PEP 380 (Python 3.3) — `yield from`:** Delegating to sub-generators. Opens a bidirectional channel from outermost caller to innermost sub-generator — values sent and yielded flow directly, exceptions propagate without boilerplate. Generators can now `return` a value (becomes the value of the `yield from` expression).

**PEP 492 (Python 3.5) — `async`/`await`:** Native coroutine syntax. `async def` creates a coroutine object; `await` replaces `yield from` for awaitable objects. Under the hood, `await` borrows from `yield from` with an added awaitable check.

**PEP 525 (Python 3.7) — Async Generators:** Allows `yield` inside `async def`, producing an async generator that supports `async for`. Combines streaming iteration with async I/O.

### Generator-Based Coroutines (Historical)

Python 3.4's `asyncio` used `@asyncio.coroutine` decorator with `yield from` for suspension. This was the bridge between PEP 380 generators and PEP 492 native coroutines. Python 3.10 removed generator-based coroutine support entirely.

The trampoline pattern from PEP 342 demonstrated cooperative multitasking: a scheduler maintains a queue of generators, calling `next()` on each in round-robin fashion. Generators yield to relinquish control. This pattern directly influenced lx's orchestrator yield model.

### `contextlib` Generators

`@contextlib.contextmanager` turns a generator with a single `yield` into a context manager. The code before `yield` runs on `__enter__`, the code after on `__exit__`. Demonstrates yield as a "cut point" that divides setup from teardown.

Sources: [PEP 255](https://peps.python.org/pep-0255/), [PEP 342](https://peps.python.org/pep-0342/), [PEP 380](https://peps.python.org/pep-0380/), [PEP 492](https://peps.python.org/pep-0492/), [PEP 525](https://peps.python.org/pep-0525/)

---

## 2. Lua: Coroutines as Foundation

### Core API

Lua provides four functions in the `coroutine` table:

| Function | Purpose |
|---|---|
| `coroutine.create(f)` | Creates a coroutine from function `f`, returns a thread value in "suspended" state |
| `coroutine.resume(co, ...)` | Resumes coroutine `co`, passing arguments. Returns `true, values` on success, `false, error` on failure |
| `coroutine.yield(...)` | Suspends the running coroutine, returning values to the `resume` caller |
| `coroutine.wrap(f)` | Creates a coroutine and returns a function that resumes it on each call (without the boolean status prefix) |
| `coroutine.status(co)` | Returns "suspended", "running", "dead", or "normal" |

### Bidirectional Data Exchange

The resume-yield pair forms a bidirectional channel:
- First `resume` arguments become the function parameters
- `yield` arguments return to the `resume` caller
- Subsequent `resume` arguments become the return values of the corresponding `yield`
- The function's final `return` values go to the last `resume`

### Asymmetric Design

Lua explicitly chose asymmetric coroutines — `yield` always returns to the `resume` caller, not to an arbitrary coroutine. As the Lua authors (de Moura and Ierusalimschy) proved in their 2004 paper "Revisiting Coroutines," asymmetric coroutines are expressively equivalent to symmetric ones but far easier to reason about — control always returns to the invoker, like a subroutine call.

### Three States

A coroutine is in one of: **suspended** (initial state, after `yield`), **running** (currently executing), **dead** (function returned or errored), or **normal** (resumed another coroutine and is waiting).

### Stackful Implementation

Lua coroutines are stackful — `yield` can be called from any depth in the call stack, not just the coroutine's top-level function. This is the critical difference from Python/JS generators. LuaJIT 1.x used Coco (by Mike Pall) for C-level stack switching; LuaJIT 2.x switched to operating on a single C stack, switching only Lua stacks (heap-allocated objects). Lua 5.2 adopted yieldable C calls based on Mike Pall's 2005 patch.

Sources: [Programming in Lua 9.1](https://www.lua.org/pil/9.1.html), [Revisiting Coroutines (de Moura & Ierusalimschy)](https://www.inf.puc-rio.br/~roberto/docs/MCC15-04.pdf), [Coco](http://coco.luajit.org/)

---

## 3. JavaScript: Generators and Async Generators

### Generators (`function*`)

Calling a generator function returns a generator object implementing both `Iterable` and `Iterator` protocols. Three control methods:
- `next(value)` — resumes to next `yield`, returning `{value, done}`. The argument becomes the `yield` expression's value. First `next()` argument is discarded (no yield to receive it yet).
- `return(value)` — forces the generator to return, executing `finally` blocks.
- `throw(error)` — throws at the current `yield` point.

`yield*` delegates to another iterable, forwarding `next/return/throw` calls through. Enables recursive generator composition.

### Stackless Limitation

JavaScript generators are stackless — `yield` can only appear directly in the `function*` body, not inside callbacks or nested functions. You cannot `yield` inside `Array.forEach`. This is a fundamental constraint of the state machine transformation.

### Async Generators

`async function*` combines async/await with generators. Produces an async iterator consumed via `for await...of`. `yield` produces values; `await` suspends for async operations. The `next()` method returns a Promise resolving to `{value, done}`.

### Generators as Async (Historical)

Before native `async`/`await` (ES2017), libraries used generators for async flow control:

**co (TJ Holowaychuk):** Wraps a generator function, returning a Promise. When you `yield` a Promise, `co` resolves it and sends the result back via `next(value)`. Supports yielding promises, thunks, arrays (parallel), objects (parallel), and nested generators. Error handling via `try/catch` inside the generator.

**Koa (v1):** Middleware was generator-based — `yield next` passed control downstream, execution after `yield` ran on the way back up. Koa v2 switched to `async/await`.

**Redux-Saga:** Still uses generators because they enable features async/await cannot: cancellation, fork/join, race conditions, and take/put patterns. The saga runtime interprets yielded effect objects (not promises) — the generator never executes side effects directly, making them trivially testable.

Why generators won't be replaced in Redux-Saga: "async/await simply don't allow for certain things — like cancellation" ([redux-saga issue #987](https://github.com/redux-saga/redux-saga/issues/987)).

Sources: [Exploring ES6 Ch. 22](https://exploringjs.com/es6/ch_generators.html), [co library](https://github.com/tj/co), [Redux-Saga](https://github.com/redux-saga/redux-saga)

---

## 4. Kotlin: Suspend Functions and Structured Concurrency

### Surface API

Kotlin coroutines are built on `suspend` functions. The `suspend` modifier marks functions that can be paused and resumed. Key constructs:

| Construct | Purpose |
|---|---|
| `suspend fun` | Function that can suspend; receives hidden `Continuation` parameter |
| `CoroutineScope` | Structured boundary for coroutine lifecycle |
| `launch` | Fire-and-forget coroutine builder (returns `Job`) |
| `async` | Coroutine builder returning `Deferred<T>` (awaitable result) |
| `Dispatchers` | Thread pool policies (Main, IO, Default, Unconfined) |
| `Flow<T>` | Cold asynchronous stream (analogous to async generators) |

### CPS Transformation

Every `suspend fun` is CPS-transformed at compile time. The compiler appends a hidden `$completion: Continuation<T>` parameter, and the return type becomes `Any?` — the function returns either the actual result (synchronous completion) or the sentinel `COROUTINE_SUSPENDED` (asynchronous suspension).

### State Machine Compilation

The compiler transforms each suspend function into a state machine:
1. Each suspension point (call to another `suspend fun`) becomes a state boundary
2. A `label` field in the Continuation object tracks the current state
3. Local variables shared across states are lifted to fields of the Continuation
4. A `when(label)` dispatch at the function entry jumps to the correct segment
5. Before suspending, local state is saved to fields; after resuming, it's restored

If a suspend function calls another suspend function only at the tail position, no state machine is needed — the continuation is forwarded directly (tail call optimization).

### Structured Concurrency

Child coroutines are scoped to their parent's `CoroutineScope`. If a parent is cancelled, all children are cancelled. If a child fails, the parent and siblings are cancelled. This prevents orphaned coroutines and resource leaks.

Sources: [Kotlin Coroutines Spec](https://kotlinlang.org/spec/asynchronous-programming-with-coroutines.html), [KEEP Coroutines Proposal](https://github.com/Kotlin/KEEP/blob/master/proposals/coroutines.md), [Coroutines Under the Hood (Kt. Academy)](https://kt.academy/article/cc-under-the-hood)

---

## 5. C#: Yield Return and Async State Machines

### Iterator Blocks (`yield return` / `yield break`)

Any method returning `IEnumerable<T>` or `IEnumerator<T>` that contains `yield return` is an iterator block. The compiler generates a nested class implementing `IEnumerator<T>` with:

- **`<>1__state` field:** Tracks execution position. Values: -2 (before `GetEnumerator`), -1 (running/after), 0 (ready), positive values (resumption points after `yield return`)
- **`<>2__current` field:** Stores the most recently yielded value
- **Local variables as instance fields:** All locals become fields (e.g., `<count>5__1`)
- **`MoveNext()` method:** A big switch on state. Each case executes code up to the next `yield return`, sets state to the next positive value, stores the value in `current`, returns `true`
- **`yield break`:** Sets state to -1, executes pending `finally` blocks, returns `false`

For `IEnumerable` returns, the same object serves as both enumerable and enumerator (single-thread optimization via thread ID check).

### Async/Await State Machines

`async` methods undergo a similar transformation. The compiler generates an `IAsyncStateMachine` with a `MoveNext()` method. Each `await` becomes a state boundary. The `AsyncTaskMethodBuilder` manages the task lifecycle.

### `IAsyncEnumerable<T>` (C# 8.0)

Combines `yield return` with `async`/`await`. Methods can be both async and iterators. `MoveNextAsync()` returns `ValueTask<bool>`. Consumed via `await foreach`. The compiler generates a combined async+iterator state machine. Allocation is minimized — the same object serves as enumerable, enumerator, and state machine, with at most two heap allocations regardless of yield count.

Sources: [Iterator Block Implementation Details (C# in Depth)](https://csharpindepth.com/articles/IteratorBlockImplementation), [Iterating with Async Enumerables (MSDN)](https://learn.microsoft.com/en-us/archive/msdn-magazine/2019/november/csharp-iterating-with-async-enumerables-in-csharp-8)

---

## 6. Rust: Coroutines, Async, and Gen Blocks

### The Coroutine Trait (Unstable, `#![feature(coroutines)]`)

```rust
pub trait Coroutine<R = ()> {
    type Yield;
    type Return;
    fn resume(self: Pin<&mut Self>, resume: R) -> CoroutineState<Self::Yield, Self::Return>;
}

pub enum CoroutineState<Y, R> {
    Yielded(Y),
    Complete(R),
}
```

Coroutines are annotated with `#[coroutine]` and use `yield`. They look like closures but compile to state machines. `resume()` takes `Pin<&mut Self>` — enabling self-referential state across yield points. Each coroutine literal gets a unique anonymous type.

### Async/Await Desugaring

`async fn` returns an `impl Future<Output = T>`. The `Future` trait's `poll` method:

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
```

The compiler generates a state machine enum. Each `.await` point becomes a variant. Local variables are stored in enum variants based on liveness analysis — variables needed only before an await go in earlier variants; those needed after go in later ones. Overlapping lifetimes share memory via multi-variant layouts.

**Pin/Unpin:** `poll` takes `Pin<&mut Self>` because async state machines can be self-referential (holding references to their own fields across await points). `Pin` guarantees the future won't be moved after first poll. Types that are not self-referential implement `Unpin` and can be freely moved.

**Waker/Context:** The `Context` carries a `Waker`. When an async operation completes (e.g., I/O ready), it calls `waker.wake()`, signaling the executor to re-poll that specific future.

### Gen Blocks (RFC 3513, Nightly)

`gen { .. }` blocks produce `impl Iterator<Item = T>`. The `yield` keyword produces values. Desugars to the same coroutine infrastructure but implements `Iterator` instead of `Future`.

Key limitations:
- **No self-references across yield:** Because `Iterator::next` takes `&mut self` (not `Pin<&mut Self>`), gen blocks cannot hold borrows across yield points
- **Must return `()`:** The trailing expression and any `return` must be unit or `!`
- **Fused:** After returning `None` once, always returns `None`

The `?` operator in gen blocks desugars specially — on error, it yields the error value (after `FromResidual` conversion) and then returns.

`gen` is reserved as a keyword in Rust 2024. `gen fn` syntax and `async gen` blocks are future work.

Sources: [Rust Coroutines (Unstable Book)](https://doc.rust-lang.org/beta/unstable-book/language-features/coroutines.html), [RFC 3513 Gen Blocks](https://rust-lang.github.io/rfcs/3513-gen-blocks.html), [Generators (without.boats)](https://without.boats/blog/generators/), [Optimizing Await (Tyler Mandry)](https://tmandry.gitlab.io/blog/posts/optimizing-await-1/)

---

## 7. Go: Goroutines (M:N Scheduling)

Goroutines are not coroutines — they are preemptively scheduled, multiplexed lightweight threads. But they occupy the same design space.

### The GMP Model

- **G (Goroutine):** The unit of work. Starts with a 2KB stack that grows dynamically (segmented/copyable stacks). Millions can coexist.
- **M (Machine):** An OS thread. The actual execution unit provided by the kernel.
- **P (Processor):** A logical scheduling context. Each P has a local run queue. Count set by `GOMAXPROCS` (default: number of CPU cores).

A G runs on an M via a P. When a G blocks (syscall, channel op), the P detaches from the M and finds or creates another M. When a G completes, the P pulls the next G from its local queue (or steals from other Ps).

### Scheduling

Go 1.14+ uses two preemption mechanisms:
1. **Cooperative (function-call-based):** The compiler inserts a preemption check at function entry (reuses the stack growth check). When the runtime sets a flag, the next function call yields.
2. **Asynchronous (signal-based):** For tight loops without function calls, the runtime sends `SIGURG` to the thread. The signal handler saves context and yields to the scheduler.

### Not Coroutines

Goroutines lack explicit `yield`. They are preempted, not cooperatively scheduled. There is no caller-callee relationship between goroutines. Communication is via channels (CSP model), not yield/resume.

Sources: [Scheduling In Go Part II (Ardan Labs)](https://www.ardanlabs.com/blog/2018/08/scheduling-in-go-part2.html), [GMP Model (dev.to)](https://dev.to/aceld/understanding-the-golang-goroutine-scheduler-gpm-model-4l1g)

---

## 8. Ruby: Fibers

### Core API

| Method | Purpose |
|---|---|
| `Fiber.new { \|args\| ... }` | Create a fiber. Block receives arguments from first `resume` |
| `fiber.resume(args)` | Start or resume fiber. Arguments become `Fiber.yield`'s return value |
| `Fiber.yield(args)` | Suspend fiber, return values to `resume` caller |
| `fiber.transfer(args)` | Transfer control to another fiber (symmetric, cannot mix with resume/yield) |
| `fiber.alive?` | Returns true if fiber hasn't terminated |
| `fiber.raise(exception)` | Raise exception at fiber's yield point |
| `fiber.kill` | Terminate fiber with uncatchable exception |

### States

Created, Suspended, Running, Terminated. Fibers are cooperative — never preempted. Each fiber has its own stack (stackful), so `Fiber.yield` works from any call depth.

### `transfer` vs `resume`/`yield`

`transfer` implements symmetric coroutine control — the calling fiber suspends and the target resumes from its last interruption. Cannot mix `transfer` with `resume`/`yield` on the same fiber — a fiber started via `transfer` can never `yield` or be `resume`d.

### Non-Blocking Fibers (Ruby 3.0+)

`Fiber.new(blocking: false)` creates a non-blocking fiber. When it encounters a blocking operation (sleep, I/O wait), it yields to the scheduler instead of blocking the thread. Requires setting `Fiber.set_scheduler(scheduler)` with an object implementing the `Fiber::Scheduler` interface (hooks for `io_wait`, `process_wait`, `kernel_sleep`, etc.). Ruby provides no default scheduler — it's user-supplied.

### Fiber Storage

Fibers inherit parent storage. `Fiber[key]` and `Fiber[key] = value` provide fiber-local variables (symbol-keyed).

Sources: [Ruby Fiber docs](https://docs.ruby-lang.org/en/master/Fiber.html), [Ruby Fibers 101 (Saeloun)](https://blog.saeloun.com/2022/03/01/ruby-fibers-101/)

---

## 9. Scheme/Racket: Continuations

### `call/cc` (Undelimited)

`(call-with-current-continuation proc)` captures the entire remaining computation as a first-class function. Calling that function abandons the current computation and jumps to the captured point. Powerful but unwieldy — captures everything up to the program's top level.

Problems with `call/cc`:
- Captures too much (the entire rest of the program)
- Non-composable — two `call/cc` captures don't compose well
- Difficult to use for practical control flow
- As noted: "call/cc is the wrong abstraction" ([Racket docs](https://docs.racket-lang.org/reference/cont.html))

### Delimited Continuations (`shift`/`reset`)

Proposed by Danvy and Filinski (1990). Two primitives:
- `reset expr` — marks a continuation boundary (the delimiter)
- `shift k expr` — captures the continuation from the current point up to the nearest `reset` as function `k`, then evaluates `expr` (which can call `k` zero or more times)

Key difference from `call/cc`: the captured continuation is bounded, not the entire program. When `shift` captures `k`, calling `k` wraps the invocation in an implicit `reset`, preventing escapes to outer scope.

An alternative pair is `prompt`/`control` (Felleisen, 1988), which does NOT wrap `k` invocations in a `reset`, allowing captured continuations to escape to the outer prompt. This makes `prompt`/`control` more expressive but harder to reason about.

### Tagged Delimiters

For nested delimiters, tagged variants (`prompt-at`/`control-at`, `reset-at`/`shift-at`) allow capturing up to a specific named delimiter, not just the nearest one.

### Racket's Implementation

Racket provides `call-with-composable-continuation` and continuation marks as its primary continuation API. Continuation prompts serve as delimiters. Racket's continuation model is more nuanced than basic `shift`/`reset`, supporting continuation barriers and prompt tags.

Sources: [Racket Continuations](https://docs.racket-lang.org/reference/cont.html), [Delimited Continuations (Frumin)](https://cs.ru.nl/~dfrumin/notes/delim.html), [shift/reset tutorial (Asai & Kiselyov)](http://pllab.is.ocha.ac.jp/~asai/cw2011tutorial/main-e.pdf)

---

## 10. C++20: Coroutines

### Three Keywords

Any function containing `co_await`, `co_yield`, or `co_return` is a coroutine.

- **`co_await expr`** — suspend until the awaited operation completes
- **`co_yield expr`** — equivalent to `co_await promise.yield_value(expr)`; suspend and produce a value
- **`co_return expr`** — complete the coroutine with a final value

### Architecture: Three Components

**Promise Type:** User-defined type controlling coroutine behavior. Must provide:
- `get_return_object()` — produce the return object before execution starts
- `initial_suspend()` — return an awaitable (lazy = `suspend_always`, eager = `suspend_never`)
- `final_suspend() noexcept` — control behavior after completion
- `return_void()` or `return_value(T)` — handle `co_return`
- `unhandled_exception()` — handle uncaught exceptions
- Optionally: `yield_value(T)` — handle `co_yield`, `await_transform(T)` — transform `co_await` arguments

**Coroutine Handle (`std::coroutine_handle<Promise>`):** Non-owning handle to the coroutine frame. Provides `resume()`, `destroy()`, `done()`, and access to the promise object.

**Coroutine Frame:** Heap-allocated (by default). Contains: promise object, copies of function parameters, suspension point marker, local variables/temporaries live across suspension points.

### Awaitable Interface

An awaiter must implement:
- `await_ready()` — return `true` to skip suspension
- `await_suspend(coroutine_handle<>)` — called on suspension. Return `void` (always suspend), `bool` (conditionally suspend), or another `coroutine_handle<>` (symmetric transfer)
- `await_resume()` — called on resumption, provides the result

### Allocation

The compiler may elide heap allocation if it can prove the coroutine's lifetime is nested within the caller's and the frame size is known at the call site (inlining the frame into the caller's stack/coroutine frame). Custom `operator new` on the promise type overrides allocation.

### Stackless Design

C++20 coroutines are stackless. Suspension stores state in the heap-allocated frame and returns to the caller. Cannot yield from nested function calls — only at `co_await`/`co_yield`/`co_return` points in the coroutine body itself.

Sources: [cppreference: Coroutines](https://en.cppreference.com/w/cpp/language/coroutines.html), [C++ Coroutines Tutorial (Stanford)](https://www.scs.stanford.edu/~dm/blog/c++-coroutines.html)

---

## 11. Taxonomy of Coroutines

### Stackful vs. Stackless

| Property | Stackful | Stackless |
|---|---|---|
| Yield from nested calls | Yes — can yield from any depth | No — only at top level |
| Memory per coroutine | Full stack (typically 2KB-8KB, growable) | Compact frame (only live variables) |
| Context switch cost | Stack switch (save/restore registers + stack pointer) | State machine transition (write fields, return) |
| Self-referential state | Natural (references on stack remain valid) | Requires pinning (Rust) or is disallowed |
| Languages | Lua, Go, Ruby (Fibers), Scheme | Python, JS, Rust, C#, Kotlin, C++20 |

Stackful coroutines maintain a full call stack per coroutine. Stackless coroutines are compiled to state machines — local variables become struct fields, yield points become states, and "suspension" is just returning from a function.

Performance data (from SC'25 workshop paper): stackful coroutines are created 2.4x faster, while stackless coroutines switch context 3.5x faster and have smaller frames. For small-state tasks, overall performance is nearly identical.

### Symmetric vs. Asymmetric

| Property | Symmetric | Asymmetric |
|---|---|---|
| Transfer model | Coroutine A transfers to coroutine B directly | Yield always returns to the caller/resumer |
| API | Single `transfer(target)` operation | Pair of `resume`/`yield` operations |
| Control flow reasoning | Harder — any coroutine can go anywhere | Easier — follows caller/callee discipline |
| Languages | Ruby `Fiber#transfer`, Modula-2 | Most: Lua, Python, JS, Kotlin, C#, Rust, C++ |

De Moura and Ierusalimschy (2004) proved symmetric and asymmetric coroutines are equally expressive. Most languages choose asymmetric because it's easier to understand — control returns to the invoker, mirroring subroutine call semantics.

### First-Class vs. Compiler-Generated

| Approach | Mechanism | Languages |
|---|---|---|
| First-class continuations | Continuations are values, can be stored/passed/invoked | Scheme, Racket |
| Compiler state machines | Compiler transforms yield into switch/enum states | Rust, C#, Kotlin, C++20 |
| Runtime stack switching | Runtime saves/restores entire stacks | Lua, Go, Ruby |

First-class continuations are the most general — they subsume all other control flow. But they're the hardest to optimize and reason about. Compiler-generated state machines are the most efficient for stackless coroutines. Runtime stack switching is a middle ground for stackful coroutines.

Sources: [Stackless vs Stackful (Varun Ramesh)](https://blog.varunramesh.net/posts/stackless-vs-stackful-coroutines/), [Revisiting Coroutines (de Moura & Ierusalimschy)](https://www.inf.puc-rio.br/~roberto/docs/MCC15-04.pdf)
