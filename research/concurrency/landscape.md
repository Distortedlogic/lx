# Concurrency Models and Actor Systems Across Languages

Research for lx language design. lx agents ARE actors with messaging (`~>`, `~>?`), parallel execution (`par`, `sel`, `pmap`), supervision trees, and pub/sub.

---

## 1. Actor Model Systems

### 1.1 Erlang/OTP

The BEAM VM is the gold standard for actor-based concurrency. Every Erlang process is a lightweight actor (~300 bytes initial heap) with its own mailbox, garbage collector, and reduction counter.

**Processes and Mailboxes**

Each process has an unbounded FIFO mailbox. Messages are copied into the receiver's heap (no shared memory between processes). The mailbox is a linked list; new messages append to the tail.

**Selective Receive**

Erlang's `receive` block pattern-matches against mailbox contents, not just the head. The runtime scans from the oldest message, trying each clause. Non-matching messages go to a "save queue" and are restored after a match is found. This enables out-of-order processing but can cause O(n) mailbox scans if patterns are too selective.

Optimization: creating a reference with `make_ref()` and matching on it lets the VM skip all messages received before the ref was created.

```erlang
receive
    {response, Ref, Value} -> Value
after 5000 ->
    timeout
end
```

**OTP Behaviors**

- **gen_server**: Request-reply server. Callbacks: `init/1`, `handle_call/3` (sync), `handle_cast/2` (async), `handle_info/2` (raw messages), `terminate/2`, `code_change/3`.
- **gen_statem**: State machine with two callback modes: `state_functions` (one function per state) and `handle_event_function` (single function dispatching on state). Replaced `gen_fsm` in OTP 20. Supports `code_change/4` for hot upgrades.
- **supervisor**: Manages child process lifecycles. See [Section 2: Supervision](#supervision-and-fault-tolerance) for strategies.

**Supervision Trees**

Hierarchical process structure: supervisors manage workers and other supervisors. Four restart strategies:

| Strategy | Behavior |
|---|---|
| `one_for_one` | Only the crashed child restarts |
| `one_for_all` | All children terminate and restart |
| `rest_for_one` | Crashed child + all children started after it restart |
| `simple_one_for_one` | Dynamic pool of identical children, `one_for_one` semantics |

Child restart types: `permanent` (always restart), `transient` (restart on abnormal exit only), `temporary` (never restart).

Restart intensity: `MaxR` restarts within `MaxT` seconds allowed before the supervisor itself terminates and escalates.

**"Let It Crash" Philosophy**

Processes are cheap and isolated. Rather than defensive error handling, let the process crash and let its supervisor restart it with known-good state. This separates error handling (supervisor) from business logic (worker).

**Hot Code Reloading**

The BEAM can load two versions of a module simultaneously. Running processes continue on the old version; new calls dispatch to the new version. `gen_server:code_change/3` and `gen_statem:code_change/4` handle state migration during upgrades.

**Distribution and Location Transparency**

Erlang nodes connect via EPMD (Erlang Port Mapper Daemon). PIDs are location-transparent: `Pid ! Message` works identically whether the process is local or on a remote node. Messages are serialized automatically for remote delivery.

**BEAM Scheduler**

- One scheduler thread per CPU core, each with its own run queue
- Preemptive scheduling via **reductions** (not time slices): each process gets ~4000 reductions before preemption
- A reduction = one unit of work (function call, arithmetic op, message send)
- **Work stealing**: idle schedulers steal processes from overloaded run queues
- **Dirty schedulers**: separate threads for long-running NIFs (CPU-intensive or I/O-blocking)
- Four priority levels: `max` (internal), `high`, `normal`, `low`
- Green threads: millions of processes on a single node, ~2KB initial stack

Sources:
- [Erlang OTP Design Principles](https://www.erlang.org/doc/system/design_principles.html)
- [Erlang Supervisor Behaviour](https://www.erlang.org/doc/system/sup_princ.html)
- [gen_statem Behaviour](https://www.erlang.org/doc/system/statem.html)
- [Deep Diving Into the Erlang Scheduler](https://blog.appsignal.com/2024/04/23/deep-diving-into-the-erlang-scheduler.html)
- [Erlang Scheduler Details](https://hamidreza-s.github.io/erlang/scheduling/real-time/preemptive/migration/2016/02/09/erlang-scheduler-details.html)
- [Learn You Some Erlang: Supervisors](https://learnyousomeerlang.com/supervisors)
- [Erlang Selective Receive Explained](https://blog.ndpar.com/2010/11/10/erlang-selective-receive/)

---

### 1.2 Akka (Scala/Java)

Akka implements the actor model on the JVM with typed actors, hierarchical supervision, persistence, clustering, and streams.

**Typed Actors**

Since Akka 2.6+, the typed API is primary. Actors are defined as `Behavior[T]` where `T` is the message protocol type. Behaviors are functional: each message handler returns the next `Behavior`.

```scala
object Counter {
  sealed trait Command
  case class Increment(replyTo: ActorRef[Int]) extends Command

  def apply(count: Int): Behavior[Command] = Behaviors.receive { (ctx, msg) =>
    msg match {
      case Increment(replyTo) =>
        replyTo ! (count + 1)
        Counter(count + 1)
    }
  }
}
```

**Actor Hierarchy and Lifecycle**

- Actors form a mandatory parent-child tree rooted at the guardian actor
- `ActorContext.spawn()` creates children; children cannot outlive parents
- Lifecycle signals: `PreRestart`, `PostStop`
- `context.watch(otherActor)` subscribes to `Terminated` signals
- `Behaviors.stopped` terminates the actor after the current message

**Supervision Strategies**

Typed Akka wraps behaviors with supervision decorators:

| Strategy | Behavior |
|---|---|
| `SupervisorStrategy.restart` | Restart immediately, no limit by default |
| `SupervisorStrategy.resume` | Keep current state, continue processing |
| `SupervisorStrategy.stop` | Stop the actor permanently |
| `SupervisorStrategy.restartWithBackoff(min, max, jitter)` | Exponential backoff restart |

Exception-specific strategies: different strategies for different exception types via `Behaviors.supervise(behavior).onFailure[ExceptionType](strategy)`.

**Persistence (Event Sourcing)**

`EventSourcedBehavior` persists events to a journal. On recovery, events replay to reconstruct state. Snapshots optimize recovery for long event histories. The **single-writer principle** ensures only one actor instance per `PersistenceId` exists at any time.

**Cluster Sharding**

Distributes actors across cluster nodes by entity ID. The shard region routes messages to the correct node. Entities are typically `EventSourcedBehavior` actors. Automatic rebalancing when nodes join/leave. Passivation removes idle entities from memory.

**Akka Streams**

Graph-based stream processing with automatic backpressure:
- **Source**: emits elements (0 outputs, 1 output)
- **Flow**: transforms elements (1 input, 1 output)
- **Sink**: consumes elements (1 input, 0 outputs)
- **RunnableGraph**: fully connected Source-Flow-Sink ready to materialize
- Windowed batching backpressure: multiple elements in-flight, batch demand signals
- Graph DSL for complex topologies (fan-in, fan-out, broadcast, merge, zip)

**Location Transparency**

ActorRefs are serializable and location-transparent. The same code works whether actors are local or remote. Akka Cluster provides automatic node discovery, failure detection, and membership management. All messages must be serializable for remote delivery.

Sources:
- [Akka Typed Actor Lifecycle](https://doc.akka.io/libraries/akka-core/current/typed/actor-lifecycle.html)
- [Akka Event Sourcing](https://doc.akka.io/libraries/akka-core/current/typed/persistence.html)
- [Akka Cluster Sharding](https://doc.akka.io/docs/akka/current/typed/cluster-sharding.html)
- [Akka Streams Basics](https://doc.akka.io/docs/akka/current/stream/stream-flows-and-basics.html)
- [Akka Location Transparency](https://doc.akka.io/docs/akka/current/general/remoting.html)
- [Akka Typed Supervision](https://akka.io/blog/article/2017/05/16/supervision)

---

### 1.3 Orleans (.NET)

Orleans implements the **Virtual Actor** model (invented at Microsoft Research) where actors (grains) always exist conceptually and are activated on demand.

**Virtual Actors / Grains**

Grains have stable identities (GUID, int, string, or compound keys) and exist perpetually in the virtual space. The runtime activates grains in memory when invoked and deactivates them when idle. No explicit create/destroy lifecycle. Callers never need to know where a grain is physically located.

**Silo Architecture**

- **Silo**: hosts grain activations, runs the Orleans runtime
- **Cluster**: group of interconnected silos for scalability and fault tolerance
- **Grain Directory**: maps grain identities to their current silo location
- Automatic placement: configurable strategies (random, prefer-local, resource-optimized, custom)
- Memory-based activation shedding: deactivate grains under memory pressure (Orleans 9.x)

**Turn-Based Concurrency**

Each grain processes one request at a time on a single thread. No locks, no concurrent access to grain state. This is the key simplification: grain code is inherently single-threaded. Reentrancy is opt-in via `[Reentrant]` attribute or `[AlwaysInterleave]` for specific methods.

**Grain Persistence**

Grains declare persistent state via `IPersistentState<T>`. Multiple named state objects per grain (e.g., "profile" and "inventory"). Pluggable storage providers: Azure Table, Cosmos DB, Redis, ADO.NET, DynamoDB. State is kept in memory while active; `WriteStateAsync()` persists changes.

**Timers and Reminders**

- **Timers**: in-memory, non-durable, tied to the current activation, high-frequency
- **Reminders**: persistent, survive grain deactivation and silo restarts, trigger reactivation

**Stateless Worker Grains**

Bypass the single-activation constraint: multiple activations of the same grain can run simultaneously across silos. Used for stateless, parallelizable operations.

**Streams**

Managed streaming: producers and consumers don't need to pre-register. Backed by Azure Event Hubs, Kinesis, etc. Grains can checkpoint stream positions for reliable processing.

**ACID Transactions**

Distributed, decentralized transactions across multiple grains with serializable isolation. No central coordinator.

Sources:
- [Orleans Overview](https://learn.microsoft.com/en-us/dotnet/orleans/overview)
- [Orleans Virtual Actors Paper](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/Orleans-MSR-TR-2014-41.pdf)
- [Orleans Timers and Reminders](https://learn.microsoft.com/en-us/dotnet/orleans/grains/timers-and-reminders)
- [Orleans Request Scheduling](https://learn.microsoft.com/en-us/dotnet/orleans/grains/request-scheduling)

---

### 1.4 Elixir/Phoenix

Elixir runs on the BEAM VM and inherits Erlang's process model with more ergonomic abstractions.

**GenServer**

The primary abstraction for stateful processes. Callbacks: `init/1`, `handle_call/3` (sync), `handle_cast/2` (async), `handle_info/2`, `terminate/2`. State is the return value of each callback.

```elixir
defmodule Counter do
  use GenServer
  def init(initial), do: {:ok, initial}
  def handle_call(:get, _from, count), do: {:reply, count, count}
  def handle_cast(:increment, count), do: {:noreply, count + 1}
end
```

**Supervisor and DynamicSupervisor**

`Supervisor` starts a fixed set of children defined at compile time. Strategies: `:one_for_one`, `:one_for_all`, `:rest_for_one`.

`DynamicSupervisor` starts children on demand at runtime. Only supports `:one_for_one`. Used for process pools, game sessions, connection handlers.

**Registry**

Process registry maps arbitrary terms (strings, tuples) to PIDs. Enables named lookups via `:via` tuples. Used with DynamicSupervisor to create named, supervised processes on demand.

**Task**

Lightweight abstraction for one-off async work:
- `Task.async/1` + `Task.await/1`: spawn and collect result
- `Task.Supervisor`: supervised tasks for fault tolerance
- `Task.async_stream/3`: parallel map with configurable concurrency

**Agent**

Simplified GenServer for pure state management. `Agent.get/2`, `Agent.update/2`, `Agent.get_and_update/2`.

**Phoenix PubSub**

Distributed publish-subscribe built on process groups (`pg`). Topics are strings. `Phoenix.PubSub.subscribe/2` and `Phoenix.PubSub.broadcast/3`. Works across nodes in a cluster. Used by Phoenix Channels for real-time web communication.

**Process Groups (pg)**

OTP's `pg` module (replaced `pg2`) provides named groups of processes across a cluster. Processes join/leave groups dynamically. Used as the foundation for Phoenix PubSub.

Sources:
- [Elixir GenServer docs](https://hexdocs.pm/elixir/GenServer.html)
- [Elixir Supervisor docs](https://hexdocs.pm/elixir/Supervisor.html)
- [GenServer, Registry, DynamicSupervisor Combined](https://dev.to/unnawut/genserver-registry-dynamicsupervisor-combined-4i9p)
- [Multiplayer Go with Registry, PubSub, DynamicSupervisors](https://blog.appsignal.com/2019/08/13/elixir-alchemy-multiplayer-go-with-registry-pubsub-and-dynamic-supervisors.html)

---

### 1.5 Rust Actor Frameworks

Rust's ownership system creates unique constraints and opportunities for actor patterns. All message types must be `Send + 'static` (transferable across threads, no borrowed references).

**Actix**

- Oldest and most mature Rust actor framework
- Custom runtime built on Tokio; async support added later (not primary design goal)
- Fastest message throughput and actor spawning in benchmarks
- Typed messages via `Handler<M>` trait implementations (multiple message types per actor)
- Bounded and unbounded mailboxes
- Arbiter system for thread management
- No built-in distribution; local concurrency focus
- Lifecycle: `Started`, `Stopping`, `Stopped` states

**Ractor**

- Erlang-inspired: models gen_server patterns directly
- Single enum message type per actor (like Erlang's pattern-matching receive)
- Built-in supervision: supervisors get notified on child start/stop/panic
- Distributed actors via companion crate `ractor_cluster` (similar to EPMD)
- Used at Meta for distributed overload protection in Rust Thrift servers
- Supports both Tokio and async-std runtimes
- More boilerplate (15 LoC for minimal actor) but closer to Erlang semantics

**Kameo**

- Balanced approach: distribution, supervision, and ergonomic API
- Multiple message types via trait implementations
- Built-in supervision strategies
- Bounded and unbounded mailboxes with backpressure
- Linked actors (Erlang-style process links)
- Derive macros reduce boilerplate (~6 LoC for minimal actor)
- Tokio-only runtime

**Xtra**

- Minimal, flexible framework
- Most runtime-flexible: Tokio, async-std, smol, wasm-bindgen
- Multiple message types, bounded mailboxes
- No supervision, no distribution
- Good for WebAssembly targets

**Key Rust Constraints**

- `Send`: type can be transferred to another thread (required for all actor messages)
- `Sync`: type can be shared between threads via `&T`
- No implicit sharing: actors own their state exclusively
- `Arc<Mutex<T>>` for shared state (but defeats actor model benefits)
- Move semantics enforce message ownership transfer (natural fit for actor isolation)

Sources:
- [Comparing Rust Actor Libraries](https://tqwewe.com/blog/comparing-rust-actor-libraries/)
- [Ractor GitHub](https://github.com/slawlor/ractor)
- [Actix Actor docs](https://actix.rs/docs/actix/actor/)
- [Kameo GitHub](https://github.com/tqwewe/kameo)
- [Xactor GitHub](https://github.com/sunli829/xactor)

---

### 1.6 Pony

Pony achieves **data-race freedom by construction** through a novel type system of reference capabilities.

**Reference Capabilities**

Six capabilities define what aliases can exist for a given reference:

| Capability | Mutable? | Shareable? | Deny Properties |
|---|---|---|---|
| `iso` (isolated) | yes | transferable | Denies ALL local and global aliases (read + write) |
| `trn` (transition) | yes | no | Denies global read/write, denies local write aliases |
| `ref` (reference) | yes | no | Denies global read/write aliases |
| `val` (value) | no | yes | Denies local and global write aliases |
| `box` | no | no | Denies global write aliases |
| `tag` | no (opaque) | yes | Denies local and global read/write |

Subtyping: `iso <: trn <: ref <: box` and `iso <: trn <: val <: box` and `tag` is the top type for identity.

**Deny Capabilities (vs. Grant Capabilities)**

Traditional capability systems say what you CAN do. Pony inverts this: capabilities express what is DENIED. The deny matrix defines what local and global aliases are forbidden for each capability. This is the key insight from the paper "Deny Capabilities for Safe, Fast Actors" (Clebsch et al., 2015).

**Data Race Prevention**

Two invariants prevent data races at compile time:
1. If a reference is mutable (`iso`, `trn`, `ref`), no other actor can read or write the object
2. If a reference is readable across actors (`val`, `box`), no actor can write the object

When local and global deny properties match (as with `iso` and `val`), the reference can be safely sent to another actor via message passing.

**Actor Integration**

- Actors are `ref` internally (full read/write to own state)
- Actors appear as `tag` externally (identity only, no direct state access)
- Asynchronous method calls (behaviors) require only `tag` visibility
- `iso` data can be sent between actors (consumed on send, reappears at receiver)
- `val` data can be freely shared across actors (immutable, no races)
- `consume` keyword transfers ownership; `recover` upgrades capabilities in controlled blocks

**Scheduling**

Pony uses cooperative scheduling with work-stealing across OS threads. No garbage collection pauses across actors (per-actor GC). Actors are GC'd when they have no references and an empty mailbox.

Sources:
- [Deny Capabilities for Safe, Fast Actors (paper)](https://www.ponylang.io/media/papers/fast-cheap.pdf)
- [Deny Capabilities Review (The Morning Paper)](https://blog.acolyer.org/2016/02/17/deny-capabilities/)
- [Pony Reference Capabilities Tutorial](https://tutorial.ponylang.io/reference-capabilities/reference-capabilities.html)
- [Pony Reference Capability Guarantees](https://tutorial.ponylang.io/reference-capabilities/guarantees.html)
- [Reference Capabilities in Pony for Everybody](https://zartstrom.github.io/pony/2016/08/28/reference-capabilities-in-pony.html)

---

## 2. Concurrency Models

### 2.1 CSP (Communicating Sequential Processes)

CSP, formalized by Tony Hoare (1978), models concurrency as sequential processes communicating through channels. Unlike the actor model, processes are anonymous and channels are named.

**Go: Goroutines and Channels**

Go is the most prominent CSP implementation. Goroutines are green threads (~2KB initial stack, grown dynamically) multiplexed onto OS threads by the Go runtime scheduler.

Channels are typed conduits:
- `ch := make(chan int)` — unbuffered (synchronous: sender blocks until receiver reads)
- `ch := make(chan int, 100)` — buffered (sender blocks only when full)
- `close(ch)` — signals no more values; receivers get zero-value after close

The `select` statement multiplexes across channels:
```go
select {
case msg := <-ch1:
    handle(msg)
case ch2 <- value:
    // sent
case <-done:
    return
default:
    // non-blocking
}
```

**Channel Patterns**

| Pattern | Description |
|---|---|
| Pipeline | Stages connected by channels: gen -> transform -> consume |
| Fan-out | Multiple goroutines read from one channel (work distribution) |
| Fan-in (merge) | Multiple channels merged into one via goroutines + WaitGroup |
| Done channel | `chan struct{}` closed to broadcast cancellation to all goroutines |
| Context cancellation | `context.WithCancel/Timeout/Deadline` propagates through call tree |
| Bounded parallelism | Fixed worker pool reading from shared input channel |
| Tee | One input channel duplicated to two output channels |
| Or-channel | First channel to produce a value wins; others are abandoned |

**Go Scheduler**

- M:N threading: M goroutines on N OS threads
- Work-stealing scheduler with per-thread local run queues + global queue
- Originally cooperative (yield at function calls); since Go 1.14, non-cooperative preemption via async signals
- `GOMAXPROCS` controls OS thread count (defaults to CPU count)

Sources:
- [Go Pipelines and Cancellation (official blog)](https://go.dev/blog/pipelines)
- [Go Scheduler Design](https://rakyll.org/scheduler/)
- [Go Concurrency Patterns](https://www.opcito.com/blogs/practical-concurrency-patterns-in-go)

---

### 2.2 Shared-Memory Concurrency

Traditional approach: multiple threads access shared data protected by synchronization primitives.

**Mutexes**

- `Mutex<T>` (Rust): wraps data, enforced at compile time via ownership. `lock()` returns a `MutexGuard` that auto-unlocks on drop. `T` must be `Send`.
- `sync.Mutex` (Go): manual Lock/Unlock, no compile-time enforcement.
- `std::mutex` (C++): RAII via `std::lock_guard`, no ownership tracking.

**RwLock**

Read-write lock: multiple concurrent readers OR one exclusive writer.
- Rust: `RwLock<T>` requires `T: Send + Sync` (because multiple threads hold `&T` for reads)
- Common pitfall: writer starvation when readers constantly hold the lock

**Atomics**

Lock-free primitives for simple values: `AtomicBool`, `AtomicU32`, `AtomicPtr<T>`, etc.
- Memory orderings: `Relaxed`, `Acquire`, `Release`, `AcqRel`, `SeqCst`
- Foundation for lock-free data structures (queues, stacks, counters)
- No generic `Atomic<T>` in Rust; only specific primitive types

**Rust's Send and Sync**

Rust's type system statically prevents data races:
- `Send`: safe to transfer ownership to another thread. Most types are Send; `Rc` is not (non-atomic refcount).
- `Sync`: safe to share `&T` across threads. `T: Sync` iff `&T: Send`. `Cell`, `RefCell` are not Sync.
- `Arc<T>`: atomic reference counting for shared ownership across threads. `Arc<Mutex<T>>` is the common shared-mutable pattern.
- Auto-traits: compiler derives Send/Sync automatically; `unsafe impl` to override.

**Lock-Free Data Structures**

- Compare-and-swap (CAS) loops for non-blocking updates
- Crossbeam (Rust): epoch-based memory reclamation, lock-free queues, deques
- ABA problem: solved by tagged pointers or epoch-based schemes

Sources:
- [Rust Shared-State Concurrency](https://doc.rust-lang.org/book/ch16-03-shared-state.html)
- [Rust Send and Sync](https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html)
- [Rust Atomics and Locks (Mara Bos)](https://mara.nl/atomics/basics.html)
- [Understanding Rust Thread Safety](https://onesignal.com/blog/thread-safety-rust/)

---

### 2.3 Async/Await

Cooperative concurrency within a single thread (or thread pool) using futures/promises.

**The Colored Functions Problem**

Bob Nystrom's "What Color is Your Function?" (2015) identifies a fundamental issue: async functions can only be called from other async functions, creating a "color" divide that splits every API surface in two. Synchronous ("blue") functions are universal; async ("red") functions are restricted.

| Language | Eager/Lazy | Built-in Runtime | Function Coloring |
|---|---|---|---|
| JavaScript | Eager (Promises start immediately) | Yes (event loop) | Yes |
| Python | Eager (coroutines) | Yes (asyncio) | Yes |
| Rust | **Lazy** (Futures do nothing until polled) | **No** (bring your own: Tokio, async-std, smol) | Yes |
| Go | N/A (goroutines) | Yes (runtime) | **No** (no coloring) |
| Erlang/Elixir | N/A (processes) | Yes (BEAM) | **No** (no coloring) |

**JavaScript Event Loop**

Single-threaded with a microtask queue (Promises) and macrotask queue (setTimeout, I/O callbacks). `async/await` is sugar over Promises. No true parallelism without Web Workers or worker threads.

**Python asyncio**

Event loop based. `async def` / `await`. `asyncio.gather()` for concurrent execution. `asyncio.TaskGroup` (3.11+) for structured concurrency. GIL prevents true parallelism in CPython.

**Rust Tokio**

- Futures are lazy: `async fn` returns a `Future` that does nothing until `.await`ed
- Tokio: multi-threaded work-stealing scheduler
- Cooperative scheduling with budget system: each task gets ~128 operations before forced yield
- `tokio::spawn` for fire-and-forget tasks; `tokio::select!` for racing futures
- `JoinSet` for managing groups of spawned tasks
- No built-in structured concurrency (tasks can outlive their spawner)

Sources:
- [What Color is Your Function? (Bob Nystrom)](https://journal.stuffwithstuff.com/2015/02/01/what-color-is-your-function/)
- [Tokio Cooperative Preemption](https://tokio.rs/blog/2020-04-preemption)
- [Rust Async Book](https://rust-lang.github.io/async-book/part-guide/async-await.html)
- [Function Color Defense](https://www.thecodedmessage.com/posts/async-colors/)

---

### 2.4 Structured Concurrency

The insight (Nathaniel J. Smith, 2018): unrestricted task spawning is the concurrency equivalent of `goto`. Structured concurrency constrains task lifetimes to lexical scopes, just as structured programming constrained control flow.

**Core Invariant**: control flow enters a scope at the top, concurrent work happens inside, and control flow exits at the bottom only after ALL child tasks complete. No task can outlive its spawning scope.

**Trio (Python)**

The pioneer of structured concurrency. Key primitives:
- **Nursery**: scope that owns child tasks. Parent blocks at nursery exit until all children finish.
- **Cancel Scope**: every nursery creates one. Cancellation propagates to all children.
- If any child raises an unhandled exception, all siblings are cancelled, then the exception propagates to the parent.

```python
async with trio.open_nursery() as nursery:
    nursery.start_soon(fetch, url1)
    nursery.start_soon(fetch, url2)
# Both tasks guaranteed complete here
```

Guarantees: no task leaks, no orphaned background work, exceptions always propagate, `with` blocks are reliable (no concurrent escape).

**Java (JEP 428 -> JEP 525)**

`StructuredTaskScope` API (incubating since JDK 19, previewing through JDK 25+):
- `fork()` spawns subtasks as virtual threads
- `join()` / `joinUntil(Instant)` waits for all subtasks
- **ShutdownOnFailure**: cancel all if any fails
- **ShutdownOnSuccess**: cancel remaining when first succeeds (race pattern)
- Custom policies via `handleComplete(Future)` override
- Thread dumps show task hierarchy in JSON format

**Swift Structured Concurrency**

- `async let`: static concurrency for known number of tasks
- `withTaskGroup` / `withThrowingTaskGroup`: dynamic concurrency for variable number of tasks
- Child task lifetime bound to parent scope
- Cancellation propagation is automatic through the task tree
- `Task.isCancelled` is cooperative: tasks must check and respond
- Error semantics: first thrown error cancels siblings, propagates to parent

**asyncio.TaskGroup (Python 3.11+)**

Python's stdlib structured concurrency, inspired by Trio:
```python
async with asyncio.TaskGroup() as tg:
    tg.create_task(coro1())
    tg.create_task(coro2())
# All tasks complete or all cancelled on error
```

Sources:
- [Notes on Structured Concurrency (Nathaniel J. Smith)](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/)
- [JEP 428: Structured Concurrency](https://openjdk.org/jeps/428)
- [Swift Structured Concurrency Proposal](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md)
- [Trio Documentation](https://trio.readthedocs.io/en/stable/)

---

### 2.5 Dataflow / Reactive

Event-driven processing where data flows through transformation stages with backpressure.

**ReactiveX (Rx)**

Observable streams with combinators: `map`, `filter`, `flatMap`, `merge`, `zip`, `buffer`, `window`, `debounce`. Backpressure via `Flowable` (RxJava 2+) with strategies: `BUFFER`, `DROP`, `LATEST`, `ERROR`, `MISSING`. The Reactive Streams spec (2013, Netflix/Pivotal/Lightbend) standardizes async stream processing with non-blocking backpressure.

**Elixir GenStage / Flow**

- **GenStage**: behavior for exchanging events between producers and consumers with demand-driven backpressure
  - Producers: emit events up to the demand requested by consumers
  - Consumers: subscribe to producers and request events when ready
  - Producer-consumers: both receive and emit events
  - Demand flows upstream; events flow downstream
- **Flow**: higher-level API built on GenStage for parallel data processing (map-reduce, partitions, windows)

**Rust Futures and Streams**

- `Future<Output = T>`: single async value, lazy, poll-based
- `Stream<Item = T>`: async iterator, yields multiple values
- Backpressure is inherent: `poll_next()` returns `Poll::Pending` when not ready
- `tokio_stream`, `futures::stream` provide combinators
- No built-in Reactive Streams implementation; community crates provide it

**Akka Streams**

See [Section 1.2](#12-akka-scalajava) for details. Key differentiator: Graph DSL for complex topologies with compile-time verification of graph connectivity.

Sources:
- [Reactive Streams Specification](https://www.reactive-streams.org/)
- [Elixir GenStage docs](https://github.com/elixir-lang/gen_stage)
- [GenStage Back-pressure Mechanism](https://dev.to/dcdourado/understanding-genstage-back-pressure-mechanism-1b0i)
- [Akka Streams Basics](https://doc.akka.io/docs/akka/current/stream/stream-flows-and-basics.html)
