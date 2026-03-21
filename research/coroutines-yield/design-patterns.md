# Coroutine Design Patterns: State Machines, CPS, Yield, Continuations

Research for lx language design. Implementation patterns, yield semantics, continuation theory, and orchestrator yield design.

---

## 1. State Machine Transformation

The dominant implementation strategy for stackless coroutines. Used by C#, Rust, Kotlin, and C++20.

### The Pattern

A function with N yield/await points is compiled into a state machine with N+1 states (plus terminal). The transformation:

1. **Enumerate states:** Each yield/await point is a state boundary. States are numbered (label/discriminant).
2. **Lift locals:** Local variables that survive across yield points become fields of a struct/class (the "continuation object" or "coroutine frame").
3. **Generate dispatch:** A switch/match on the state label at function entry jumps to the correct resumption point.
4. **Replace yields with returns:** Each yield saves state, sets the label to the next state, and returns (suspend).

### C# Implementation

The compiler generates a nested class implementing `IEnumerator<T>`:

```csharp
// Source:
IEnumerable<int> Range(int lo, int hi) {
    for (int i = lo; i < hi; i++)
        yield return i;
}

// Generated (simplified):
class RangeIterator : IEnumerator<int> {
    int state;      // -1=done, 0=start, 1=after-yield
    int current;
    int lo, hi, i;  // lifted locals + params

    bool MoveNext() {
        switch (state) {
            case 0:
                i = lo;
                goto case 1;
            case 1:
                if (i >= hi) { state = -1; return false; }
                current = i;
                i++;
                state = 1;
                return true;
            default: return false;
        }
    }
}
```

State values: -2 = before `GetEnumerator`, -1 = running/done, 0 = ready, positive = resumption points. `finally` blocks become separate methods called from both `MoveNext` (on normal flow) and `Dispose` (on early termination).

Source: [Iterator Block Implementation (C# in Depth)](https://csharpindepth.com/articles/IteratorBlockImplementation)

### Rust Implementation

Async functions compile to enum state machines:

```rust
// Source:
async fn example(x: i32) -> String {
    let a = step_one(x).await;     // await point 1
    let b = step_two(a).await;     // await point 2
    format!("{b}")
}

// Generated (conceptual):
enum ExampleFuture {
    Start { x: i32 },
    WaitingStepOne { fut: StepOneFuture },
    WaitingStepTwo { fut: StepTwoFuture },
    Done,
}

impl Future for ExampleFuture {
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        loop {
            match &mut *self {
                Start { x } => {
                    let fut = step_one(*x);
                    *self = WaitingStepOne { fut };
                }
                WaitingStepOne { fut } => match Pin::new(fut).poll(cx) {
                    Poll::Ready(a) => {
                        let fut = step_two(a);
                        *self = WaitingStepTwo { fut };
                    }
                    Poll::Pending => return Poll::Pending,
                },
                WaitingStepTwo { fut } => match Pin::new(fut).poll(cx) {
                    Poll::Ready(b) => {
                        *self = Done;
                        return Poll::Ready(format!("{b}"));
                    }
                    Poll::Pending => return Poll::Pending,
                },
                Done => panic!("polled after completion"),
            }
        }
    }
}
```

Key optimization: variables are stored per-variant based on liveness. When the state machine is in `WaitingStepOne`, those bytes are interpreted as `StepOneFuture`; in `WaitingStepTwo`, the same memory holds `StepTwoFuture`. This multi-variant layout prevents exponential size growth in nested futures.

Source: [How Rust optimizes async/await (Tyler Mandry)](https://tmandry.gitlab.io/blog/posts/optimizing-await-1/), [Async/Await OS in Rust (phil-opp)](https://os.phil-opp.com/async-await/)

### Kotlin Implementation

Suspend functions compile to state machines inside a `Continuation` object:

```kotlin
// Source:
suspend fun processOrder(orderId: String): Receipt {
    val order = fetchOrder(orderId)    // suspend point 1
    val payment = processPayment(order) // suspend point 2
    return generateReceipt(payment)     // suspend point 3
}

// Generated (simplified):
fun processOrder(orderId: String, $completion: Continuation<Receipt>): Any? {
    val cont = $completion as? ProcessOrderContinuation
        ?: ProcessOrderContinuation($completion)

    when (cont.label) {
        0 -> {
            cont.orderId = orderId
            cont.label = 1
            val result = fetchOrder(orderId, cont)
            if (result == COROUTINE_SUSPENDED) return COROUTINE_SUSPENDED
            // synchronous completion: fall through
        }
        1 -> {
            val order = cont.result as Order
            cont.label = 2
            val result = processPayment(order, cont)
            if (result == COROUTINE_SUSPENDED) return COROUTINE_SUSPENDED
        }
        2 -> {
            val payment = cont.result as Payment
            cont.label = 3
            val result = generateReceipt(payment, cont)
            if (result == COROUTINE_SUSPENDED) return COROUTINE_SUSPENDED
        }
        3 -> {
            return cont.result as Receipt
        }
    }
}
```

The `Continuation` interface has `resumeWith(Result<T>)`. The `COROUTINE_SUSPENDED` sentinel tells the caller "this function paused, don't use the return value." When the async operation completes, it calls `cont.resumeWith(result)`, which re-enters the function at the saved label.

Tail call optimization: if a suspend function calls another only at the tail, no state machine is generated — the continuation is forwarded directly.

Source: [Kotlin Coroutines Spec](https://kotlinlang.org/spec/asynchronous-programming-with-coroutines.html), [Suspending State Machines (Pedro Felix)](https://labs.pedrofelix.org/guides/kotlin/coroutines/coroutines-and-state-machines)

---

## 2. Continuation-Passing Style (CPS)

### The Transformation

In direct style, a function returns a value. In CPS, a function takes an extra argument — the continuation — and passes its result to the continuation instead of returning:

```
// Direct style:
f(x) = x + 1

// CPS:
f(x, k) = k(x + 1)
```

In CPS, every call is a tail call. There is no implicit stack — the continuation chain is explicit. Control flow (loops, conditionals, exceptions) becomes explicit through continuation composition.

### CPS and Async/Await

CPS is the theoretical foundation for async/await. The relationship:

| Concept | CPS | Async/Await |
|---|---|---|
| "What happens next" | Explicit continuation function | Implicit (compiler generates it) |
| Suspension | Call continuation later | Return `COROUTINE_SUSPENDED` / `Poll::Pending` |
| Resumption | Invoke the continuation | Call `resume()` / `poll()` again |
| Nesting | Continuations nest as closures | Compiler flattens into state machine |

Kotlin's implementation is the most explicitly CPS-flavored: each suspend function literally receives a `Continuation` parameter. The compiler then optimizes the closure nesting into a flat state machine.

As Eric Lippert (C# language designer) wrote: "CPS is the basis for techniques such as async/await in C#." The nested continuations that CPS produces are exactly what async/await flattens with syntactic sugar.

### CPS in Compilers

Functional language compilers (SML/NJ, Haskell's GHC, early Scheme) use CPS as an intermediate representation. Properties that make it useful:
- Procedure returns become calls (uniform representation)
- All intermediate values are named
- Argument evaluation order is explicit
- Tail calls are syntactically obvious
- Equivalent in power to SSA (Static Single Assignment) form

Source: [CPS (Wikipedia)](https://en.wikipedia.org/wiki/Continuation-passing_style), [CPS and Asynchrony (Eric Lippert)](https://learn.microsoft.com/en-us/archive/blogs/ericlippert/continuation-passing-style-revisited-part-five-cps-and-asynchrony)

---

## 3. Stack Switching (Stackful Coroutines)

### Mechanism

Stackful coroutines save and restore the entire execution stack:

1. **Stack allocation:** Each coroutine gets its own stack (via `mmap`, `VirtualAlloc`, or similar). Typical sizes: 2KB-8KB initial, growable.
2. **Context save:** On yield, save CPU registers (including stack pointer, instruction pointer, callee-saved registers) to a context struct.
3. **Context restore:** On resume, restore registers from the target coroutine's context struct. Execution continues where it left off.

### Platform Implementations

| Platform | Mechanism |
|---|---|
| Linux | `mmap` for stack allocation, inline assembly or `swapcontext` for switching |
| Windows | Fibers API (`CreateFiber`/`SwitchToFiber`) or manual context switching |
| Go | Custom assembly per-architecture. Goroutine stacks start at 2KB, grow by copying to larger allocation |
| Lua (LuaJIT 1.x) | Coco library — `mmap`-based stacks, inline assembly for switching |
| Lua (LuaJIT 2.x) | Single C stack, heap-allocated Lua stacks only |

### Segmented vs. Contiguous Stacks

**Segmented stacks (Go pre-1.4):** Stack is a linked list of segments. When a function prologue detects insufficient space, a new segment is allocated and linked. Problem: "hot split" — a function near a segment boundary repeatedly allocates/frees segments.

**Contiguous/copyable stacks (Go 1.4+):** Stack is a single contiguous allocation. When it fills, allocate a new stack 2x the size, copy everything, update all pointers. Eliminates hot split but requires the runtime to know which values on the stack are pointers.

### Trade-offs vs. Stackless

Stackful advantages:
- Yield from any call depth (libraries don't need to be coroutine-aware)
- Simpler mental model (looks like regular code)
- Natural fit for blocking APIs

Stackless advantages:
- Smaller memory footprint (only live variables, not full stack)
- Faster context switch (state machine transition vs. stack copy/swap)
- Compiler can optimize across yield points
- No minimum stack size overhead

---

## 4. Generators as Iterators

### The Yield-to-Iterator Pattern

The most common use of `yield`: turning imperative code into a lazy iterator.

```python
# Python: Fibonacci as lazy iterator
def fib():
    a, b = 0, 1
    while True:
        yield a
        a, b = b, a + b

# Take first 10
for x in itertools.islice(fib(), 10):
    print(x)
```

Properties:
- **Lazy:** Values computed on demand, not upfront
- **Infinite sequences:** Natural representation (no collection to store)
- **Composable:** Chain with map/filter/take without materializing
- **Memory efficient:** O(1) memory regardless of sequence length

### Language-Specific Iterator Protocols

| Language | Generator Output | Iterator Protocol |
|---|---|---|
| Python | Generator object | `__iter__` + `__next__`, raises `StopIteration` |
| JavaScript | Generator object | `Symbol.iterator`, returns `{value, done}` |
| Rust (gen blocks) | `impl Iterator<Item=T>` | `next() -> Option<T>` |
| C# | `IEnumerable<T>` | `GetEnumerator()` → `MoveNext()` + `Current` |
| Kotlin | `Sequence<T>` via `sequence { }` | `Iterator<T>` with `hasNext()` + `next()` |

### Tree Traversal Pattern

Generators excel at recursive traversal where manual iteration requires explicit stacks:

```python
def inorder(node):
    if node is None:
        return
    yield from inorder(node.left)
    yield node.value
    yield from inorder(node.right)
```

Without generators, this requires an explicit stack, a state machine, or callbacks.

---

## 5. Generators as Async (Historical Pattern)

Before `async`/`await` syntax, generators served as the async mechanism. The pattern:

1. Write a generator that `yield`s promises/thunks
2. A runtime/library resumes the generator with the resolved value
3. Errors are thrown into the generator via `throw()`
4. The generator reads like synchronous code

### The Trampoline

```javascript
// Simplified co-style runner:
function run(genFn) {
    const gen = genFn();
    function step(value) {
        const result = gen.next(value);
        if (result.done) return result.value;
        return result.value.then(step, err => gen.throw(err));
    }
    return step();
}

// Usage:
run(function*() {
    const user = yield fetchUser(id);     // yield a promise
    const posts = yield fetchPosts(user); // yield another
    return posts;
});
```

### Evolution Timeline

| Year | Language | Mechanism |
|---|---|---|
| 2013 | Python 3.4 | `@asyncio.coroutine` + `yield from` |
| 2013 | JavaScript | co library + `function*` + `yield` |
| 2014 | Koa v1 | Generator middleware (`yield next`) |
| 2015 | Python 3.5 | Native `async`/`await` (PEP 492) |
| 2017 | JavaScript | Native `async`/`await` (ES2017) |
| 2017 | Koa v2 | Migrated to `async`/`await` |

The generator-as-async pattern demonstrated that `yield` is fundamentally about pausing execution and receiving a value from an external driver. `async`/`await` is syntactic sugar over this insight.

### Why Redux-Saga Still Uses Generators

Redux-Saga yields effect descriptions (plain objects), not promises. The saga middleware interprets these effects. This provides capabilities async/await cannot:
- **Cancellation:** The middleware can stop advancing the generator
- **Testing:** Effects are data; assertions don't require mocking
- **Fork/join:** Multiple generators run concurrently, coordinated by the middleware
- **Take/put:** Generators block on specific Redux actions

Source: [co library](https://github.com/tj/co), [Redux-Saga](https://github.com/redux-saga/redux-saga)

---

## 6. Yield for Cooperative Scheduling

### The Pattern

Multiple coroutines share a single thread. Each yields voluntarily to let others run. A scheduler/dispatcher decides who runs next.

```python
# Simplified cooperative scheduler
from collections import deque

def scheduler(tasks):
    queue = deque(tasks)
    while queue:
        task = queue.popleft()
        try:
            task.send(None)
            queue.append(task)
        except StopIteration:
            pass
```

### Language Support

| Language | Mechanism |
|---|---|
| Lua | `coroutine.resume`/`coroutine.yield` with a scheduler loop |
| Python | Generator-based with a trampoline (PEP 342 example) |
| Ruby | `Fiber.resume`/`Fiber.yield` with scheduler |
| Ruby 3.0+ | `Fiber::Scheduler` interface for automatic scheduling of non-blocking fibers |
| Go | Goroutines with automatic preemptive scheduling (not truly cooperative) |

### Advantages Over Threads

- No locks needed (single-threaded, cooperative)
- Deterministic interleaving (yield points are explicit)
- No context switch overhead at OS level
- Easier to debug (predictable execution order)

### Disadvantages

- One coroutine blocking blocks everything (no preemption)
- Programmer must insert yield points
- Cannot utilize multiple CPU cores (single thread)

---

## 7. Yield for Effects (Algebraic Effects)

### The Generalization

Algebraic effects generalize exceptions, state, generators, and async/await into a single mechanism. An effect handler is like `try`/`catch` but the "catch" block receives a continuation that can resume the "throwing" code.

```
// Pseudocode:
handle {
    let x = perform ReadLine          // "throw" an effect
    let y = perform ReadLine
    print(x + y)
} with {
    ReadLine -> resume("hello")       // "catch" with resumption
}
```

The handler receives the continuation (everything after `perform`) as a function it can call zero, one, or multiple times.

### Relationship to Yield

| Concept | Generator Yield | Effect Perform |
|---|---|---|
| Pause execution | Yes | Yes |
| Value to caller/handler | Yielded value | Effect description |
| Resume with value | `send(value)` | `resume(value)` |
| Multiple handlers | No (single caller) | Yes (handler stack) |
| Multiple resumptions | No (one-shot) | Possible (multi-shot) |
| Scope | Lexical (generator body) | Dynamic (nearest matching handler) |

### Languages with Algebraic Effects

| Language | Status | Implementation |
|---|---|---|
| Koka | Production | Compiles effects via optimized CPS ("generalized evidence passing") |
| Eff | Research | Direct operational semantics, multi-shot handlers |
| OCaml 5 | Production | One-shot delimited continuations via fiber-based runtime |
| Unison | Production | Algebraic abilities (effects) as core language feature |

### Implementation via Delimited Continuations

Algebraic effects can be implemented as syntactic sugar over `shift`/`reset`:
- `handle { ... }` ≈ `reset { ... }`
- `perform effect` ≈ `shift k { handler(effect, k) }`
- `resume(value)` ≈ calling the captured `k(value)`

Source: [Algebraic Effects for Functional Programming (Leijen)](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/08/algeff-tr-2016-v2.pdf), [Implementing Algebraic Effects (yelouafi)](https://gist.github.com/yelouafi/5f8550b887ab7ffcf3284602330bd37d)

---

## 8. Continuations

### Undelimited Continuations (`call/cc`)

`call/cc` captures the entire remaining computation as a first-class function:

```scheme
(+ 1 (call/cc (lambda (k)
    (k 42))))
;; => 43  (k receives "add 1 to ___", called with 42)
```

The continuation `k` represents "everything that would happen after this expression." Calling `k` abandons the current computation and jumps to the captured point.

Problems:
- Captures everything up to the top level (too coarse)
- Non-composable (two `call/cc` captures interact poorly)
- Cannot express `yield` directly (would need to capture and store the continuation, then call it later)
- Performance: saving the entire stack is expensive

### Delimited Continuations (`shift`/`reset`)

Capture only a bounded portion of the continuation:

```scheme
(reset
  (+ 1 (shift k (k (k 5)))))
;; k = (lambda (v) (+ 1 v))
;; (k (k 5)) = (k 6) = 7
```

`reset` marks the boundary. `shift` captures everything between itself and the nearest `reset` as function `k`. The body of `shift` replaces the entire `reset` expression.

Critical property: calling `k` wraps the invocation in an implicit `reset`, making shift/reset composable. `prompt`/`control` does NOT add this implicit delimiter, allowing more expressive but harder-to-reason-about patterns.

### One-Shot vs. Multi-Shot Continuations

| Property | One-Shot | Multi-Shot |
|---|---|---|
| Resumption | At most once | Any number of times |
| Implementation | Can destructively reuse the stack/frame | Must copy stack/frame for each resumption |
| Safety | Compatible with mutable state | Breaks reasoning about mutable state |
| Performance | Faster (no copying overhead) | Slower (must clone state) |
| Type system | Affine (use at most once) | Unrestricted |
| Languages | OCaml 5, most practical systems | Scheme `call/cc`, Eff, research languages |

Multi-shot continuations break the rule that "every code block entered is exited at most once." This makes them incompatible with mutable references — resuming twice means the same mutable variable is modified along two different execution paths. OCaml 5 chose one-shot only for this reason.

The Affect type system (POPL 2025) uses affine types to track whether continuations are one-shot, enabling the compiler to optimize accordingly.

Source: [Delimited Continuations (Frumin)](https://cs.ru.nl/~dfrumin/notes/delim.html), [One-Shot Algebraic Effects as Coroutines (Piróg et al.)](https://www.logic.cs.tsukuba.ac.jp/~sat/pdf/tfp2020.pdf), [Affect: An Affine Type and Effect System (POPL 2025)](https://dl.acm.org/doi/10.1145/3704841)

---

## 9. Orchestrator Yield: lx's Pattern

### The lx Yield Model

lx uses `yield` for agent-orchestrator communication. From `pkg/agent.lx`:

```
run = () {
    self.init ()
    loop {
        msg = yield {status: "ready"}
        result = self.handle msg
        yield result
    }
}
```

This is the **yield-as-pause/resume** pattern with bidirectional data exchange:
1. Agent yields a status/result to the orchestrator
2. Orchestrator receives the yielded value, decides what to do
3. Orchestrator sends a message back, which becomes the value of the `yield` expression
4. Agent processes the message and yields the result

### How lx's Yield Maps to the Landscape

| Aspect | lx | Closest Analog |
|---|---|---|
| Bidirectional data | `msg = yield {status: "ready"}` | Python `gen.send(value)` |
| Orchestrator control | Orchestrator decides what/when to send | Redux-Saga middleware interpreting effects |
| Agent streaming (`~>>?`) | Stream values to orchestrator | Async generators / IAsyncEnumerable |
| Approval gates | `yield {kind: "approval" data: summary}` | Algebraic effect perform/handle |
| Cooperative scheduling | Yield returns control to orchestrator | Lua coroutine.yield to scheduler |

### Design Space for lx

**Yield as effect:** lx's yield is closest to algebraic effects — the agent "performs" an effect (status report, approval request, result), and the orchestrator "handles" it by deciding whether to resume, with what value, or whether to cancel.

**One-shot semantics:** lx yields are one-shot — the orchestrator resumes the agent exactly once per yield. There is no need for multi-shot resumption in the agent model (you don't want to replay an agent's computation from a checkpoint).

**Stackless fit:** lx agents yield only at the top level of their `run` loop, making stackless (state machine) implementation natural. The orchestrator doesn't need to interrupt agents at arbitrary call depths.

**Backpressure via yield:** The `yield` mechanism provides natural backpressure — the agent cannot produce more work until the orchestrator resumes it. This prevents runaway agents without explicit rate limiting.

### Related Patterns in lx's Codebase

The brain orchestrator (`brain/orchestrator.lx`) uses the same pattern for cognitive loops:

```
input := yield {kind: "ready"}
...
input <- yield {
    kind: "response"
    data: result
}
```

The `<-` assignment from yield shows the bidirectional channel: yield sends a response, receive the next input. This is Python's `send()` pattern embedded into lx syntax.

### Comparison: Yield vs. Message Passing

lx has both yield (for orchestrator control) and message passing (`~>`, `~>?` for inter-agent communication). The distinction:

| Property | Yield | Message Passing |
|---|---|---|
| Direction | Agent → Orchestrator → Agent | Agent → Agent |
| Coupling | Tight (caller/callee) | Loose (fire-and-forget or ask) |
| Blocking | Always (agent suspends until resumed) | `~>` is async, `~>?` blocks |
| Purpose | Control flow, status, approval | Data exchange, delegation |
| Analogy | Generator yield/send | Actor tell/ask |

Yield is for vertical control (orchestrator managing agent lifecycle). Message passing is for horizontal communication (agents collaborating as peers).

---

## 10. Summary: Implementation Decision Matrix

| If you need... | Use... | Example |
|---|---|---|
| Lazy iteration | Generator → Iterator state machine | Rust gen blocks, C# yield return, Python generators |
| Async I/O | Future/Promise state machine | Rust async/await, C# async/await, Kotlin suspend |
| Bidirectional pause/resume | Coroutine with send/resume | Python gen.send(), lx yield, Lua resume/yield |
| Yield from nested calls | Stackful coroutines | Lua, Go goroutines, Ruby Fibers |
| Structured effect handling | Algebraic effects or delimited continuations | Koka, OCaml 5, Scheme shift/reset |
| Agent orchestration | Yield-as-effect with one-shot resume | lx yield, Redux-Saga |
| Cooperative multitasking | Scheduler + yield | Lua scheduler, Python trampoline, Ruby Fiber::Scheduler |
| General control flow | First-class continuations | Scheme call/cc (avoid if possible — use delimited) |
