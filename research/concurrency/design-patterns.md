# Concurrency Design Patterns: Messaging, Supervision, Scheduling, Fault Tolerance

Research for lx language design. Cross-cutting patterns that apply across actor systems and concurrency models.

---

## 1. Message Passing Patterns

### 1.1 Synchronous vs. Asynchronous

| Aspect | Synchronous | Asynchronous |
|---|---|---|
| Sender behavior | Blocks until receiver processes message | Returns immediately |
| Coupling | Tight (temporal) | Loose |
| Backpressure | Implicit (sender blocked) | Must be explicit (bounded mailbox, demand) |
| Deadlock risk | Higher (mutual blocking) | Lower (but starvation possible) |
| Ordering | Strong (request-response paired) | Weak (arrival order may vary) |

In the actor model, message passing is fundamentally asynchronous. Synchronous request-reply is built on top via patterns (ask, call). CSP channels can be either: unbuffered channels are synchronous (sender blocks until receiver reads); buffered channels are asynchronous up to the buffer size.

### 1.2 Fire-and-Forget (Tell)

The simplest pattern: send a message and continue without waiting for a response.

| System | Syntax |
|---|---|
| Erlang | `Pid ! Message` |
| Akka | `actorRef.tell(message)` or `actorRef ! message` |
| lx | `agent ~> message` |
| Go | `ch <- value` (buffered channel, non-blocking) |

Properties:
- No delivery guarantee without additional infrastructure
- No backpressure signal to sender
- Fastest option; lowest overhead
- Appropriate when: the sender doesn't need confirmation, fire-rate is controlled, or loss is acceptable

### 1.3 Request-Reply (Ask / Call)

Sender sends a message and awaits a response. Creates a temporary reply channel or mailbox entry.

| System | Mechanism |
|---|---|
| Erlang | `gen_server:call/2,3` — sends `{call, From, Request}`, blocks caller until `{reply, Reply, NewState}` |
| Akka | `context.ask(actorRef, replyTo => Request(replyTo))` — creates temporary actor for reply |
| lx | `agent ~>? message` — synchronous ask |
| Go | Send on channel, receive on reply channel |
| Orleans | Direct method call on grain reference (async, returns `Task<T>`) |

**Implementation details:**
- Erlang's `call` uses a unique reference (`make_ref()`) to match the reply, enabling selective receive optimization
- Akka's ask pattern creates a `Future` that completes when the reply arrives; timeout required to prevent resource leak
- Go typically uses a pair of channels or a struct with an embedded reply channel

**Timeout requirement:** All ask patterns need timeouts. Without them, a crashed recipient causes the caller to block forever. Erlang defaults to 5000ms; Akka requires explicit timeout.

### 1.4 Request-Reply with Adapter

When the reply message type doesn't match the asking actor's protocol, an adapter function transforms it:

```
// Akka Typed pattern
context.ask(otherActor, replyTo => OtherProtocol.Query(replyTo)) {
  case Success(response) => MyProtocol.WrappedResponse(response)
  case Failure(ex) => MyProtocol.QueryFailed(ex)
}
```

This avoids coupling actor protocols and is essential in typed actor systems where each actor has a specific message type.

### 1.5 Pipe-To Pattern

Send the result of an async operation to an actor (including self):

```
// Akka
val future: Future[Result] = externalService.query()
future.pipeTo(self)

// Equivalent in lx: pipe result of async work back to agent
```

Avoids closing over mutable actor state in future callbacks (which would break actor isolation).

### 1.6 Forward Pattern

Relay a message to another actor, preserving the original sender reference so the final recipient can reply directly to the originator:

```
// Akka: actorRef.forward(message) preserves sender()
// Erlang: manually pass From in the message
```

Useful for router/dispatcher actors that don't process messages themselves.

### 1.7 Mailbox Ordering Guarantees

**Single-sender ordering**: Most actor systems guarantee that messages from actor A to actor B arrive in send order. This is the minimum guarantee.

**Multi-sender ordering**: No general guarantee. Messages from A and C to B may interleave arbitrarily. The only guarantee is per-sender FIFO.

**Erlang specifics**: Messages between two processes on the same node are ordered. Messages via distribution (across nodes) maintain per-sender order but may be lost (network partition).

**Priority mailboxes**: Some systems (Akka, Pony) support priority mailboxes where messages are dequeued by priority rather than arrival order. Erlang achieves this via selective receive.

Sources:
- [Message Passing and the Actor Model](http://dist-prog-book.com/chapter/3/message-passing.html)
- [Akka Tell vs Forward](https://www.baeldung.com/scala/akka-actor-tell-vs-forward)
- [Actor-based Concurrency Overview](https://berb.github.io/diploma-thesis/original/054_actors.html)

---

## 2. Supervision and Fault Tolerance

### 2.1 Supervision Trees

The hierarchical structure where supervisors monitor workers and other supervisors, forming a tree from root to leaves.

**Erlang/OTP Model (canonical)**

```
        [Application]
             |
        [Top Supervisor]
        /       |        \
  [Supervisor] [Worker]  [Supervisor]
   /    \                  /    \
[Worker][Worker]       [Worker][Worker]
```

**Restart Strategies**

| Strategy | When to use |
|---|---|
| `one_for_one` | Children are independent; failure of one doesn't affect others |
| `one_for_all` | Children are interdependent; if one fails, the group's shared invariant is broken |
| `rest_for_one` | Children have sequential dependencies (A starts B starts C); if B fails, C must also restart |
| `simple_one_for_one` | Dynamic pool of identical workers (connection handlers, job processors) |

**Restart Intensity**

Controls cascade failures: if `MaxR` restarts occur within `MaxT` seconds, the supervisor itself terminates and escalates to its parent. This prevents infinite restart loops.

Examples:
- `intensity=1, period=5` (default): at most 1 restart per 5 seconds
- `intensity=10, period=1`: up to 10 restarts per second (aggressive)
- `intensity=5, period=30`: at most 1 restart per 6 seconds (conservative)

**Child Specification**

Each child declares:
- `restart`: `permanent` | `transient` | `temporary`
- `shutdown`: `brutal_kill` | timeout in ms | `infinity`
- `type`: `worker` | `supervisor`

Children start in specification order and terminate in reverse order. This ensures dependencies are satisfied.

### 2.2 Akka Supervision

Differs from Erlang: supervision is a behavior decorator, not a separate process type.

```scala
Behaviors.supervise(myBehavior)
  .onFailure[IOException](SupervisorStrategy.restart)
  .onFailure[IllegalStateException](SupervisorStrategy.stop)
```

**Strategies:**
- `restart`: create new instance, reset state
- `resume`: keep current state, continue (dangerous but useful for transient errors)
- `stop`: terminate permanently
- `restartWithBackoff(minBackoff, maxBackoff, randomFactor)`: exponential backoff with jitter

**Key difference from Erlang**: Akka typed actors define supervision per-behavior, not per-supervisor. Each actor wraps its own behavior with supervision logic. The parent is still notified (via `Terminated` signal) but the restart decision is embedded in the child's behavior definition.

### 2.3 Circuit Breaker

Prevents repeated calls to a failing service. Three states:

```
[Closed] --failure threshold--> [Open] --timeout--> [Half-Open]
   ^                                                     |
   |                                                     |
   +------ probe succeeds ------<--------+              |
   +------ probe fails ---------<--------+
```

- **Closed**: requests flow normally; failures counted
- **Open**: requests immediately rejected; no calls to downstream
- **Half-Open**: one probe request allowed; success closes, failure reopens

**Implementation in actor systems**: wrap the actor's outbound calls. The circuit breaker is state local to the calling actor (or a dedicated circuit breaker actor).

Akka provides `akka.pattern.CircuitBreaker` with configurable `maxFailures`, `callTimeout`, and `resetTimeout`.

### 2.4 Bulkhead

Isolate failure domains by partitioning resources:

- **Thread pool bulkhead**: each downstream service gets its own thread pool; exhaustion in one pool doesn't affect others
- **Actor bulkhead**: separate supervisor trees for independent subsystems; crash in one tree doesn't cascade
- **Connection pool bulkhead**: dedicated connection pools per external dependency

The name comes from ship design: watertight compartments prevent a hull breach from flooding the entire vessel.

In lx, the supervisor tree naturally provides bulkheading: agents under different supervisors are isolated failure domains.

### 2.5 Let It Crash vs. Defensive Programming

| Approach | Philosophy | Error location |
|---|---|---|
| Let it crash (Erlang) | Processes are cheap; crash and restart with clean state | Supervisor handles errors |
| Defensive (traditional) | Catch and handle every possible error in-line | Business logic handles errors |

**When "let it crash" works best:**
- Process state can be reconstructed from external sources (DB, message replay)
- Crash cleanup is cheap (process memory is reclaimed)
- The error is transient (retry will likely succeed)
- The error is unexpected (no reasonable in-line handler exists)

**When defensive handling is better:**
- User-facing errors that need specific error messages
- Errors that require compensation actions (rollback)
- Errors with well-known recovery strategies

**Hybrid approach** (recommended for lx): validate inputs defensively (return errors for expected failures), let unexpected failures crash the agent, supervisor handles restart.

Sources:
- [Erlang Supervisor Behaviour](https://www.erlang.org/doc/system/sup_princ.html)
- [Akka Typed Supervision](https://akka.io/blog/article/2017/05/16/supervision)
- [Akka Supervision and Monitoring](https://doc.akka.io/docs/akka/2.5/general/supervision.html)
- [Fault Tolerance Patterns](https://system-design.space/en/chapter/resilience-patterns/)
- [Resilience Design Patterns](https://www.codecentric.de/en/knowledge-hub/blog/resilience-design-patterns-retry-fallback-timeout-circuit-breaker)
- [Error Handling in Distributed Systems (Temporal)](https://temporal.io/blog/error-handling-in-distributed-systems)

---

## 3. Scheduling

### 3.1 Preemptive vs. Cooperative Scheduling

| Aspect | Preemptive | Cooperative |
|---|---|---|
| Who decides | Runtime/OS interrupts the task | Task yields voluntarily |
| Fairness | Guaranteed (no task can starve others) | Depends on tasks yielding |
| Latency | Bounded (preemption ensures progress) | Unbounded (bad actor blocks everything) |
| Overhead | Context switch cost at arbitrary points | Lower overhead; yield at known-good points |
| Complexity | Runtime must save/restore full state | Simpler runtime; state saved at yield points |
| Examples | BEAM, OS threads, Go (since 1.14) | Tokio, JavaScript, Python asyncio |

### 3.2 BEAM Reductions

The BEAM scheduler is preemptive based on "reductions" rather than wall-clock time:
- Each process gets ~4000 reductions per scheduling quantum
- One reduction = one fundamental operation (function call, message send, arithmetic)
- After exhausting reductions, the process is preempted regardless of what it's doing
- This ensures fair scheduling even with compute-heavy processes
- BIFs (built-in functions) and NIFs (native functions) consume reductions proportionally

**Why reductions, not time slices?** Time slices are OS-dependent and coarse. Reductions give deterministic, fine-grained fairness independent of CPU speed.

### 3.3 Work Stealing

Idle processors steal tasks from busy processors' run queues.

**BEAM work stealing:**
- One run queue per scheduler (one scheduler per core)
- Periodic migration: load balancer moves processes from overloaded queues
- Compaction: schedulers can be suspended when load is low (power saving)
- Migration path: check other schedulers' queue lengths, steal from the longest

**Go work stealing:**
- Each P (processor) has a local run queue (capacity 256)
- When local queue is empty: steal from other P's queue, then check global queue, then poll network
- Stealing takes half of the victim's queue (batch steal)
- `GOMAXPROCS` controls P count

**Tokio work stealing:**
- Each worker thread has a local queue
- Global injection queue for newly spawned tasks
- Idle workers steal from other workers' local queues
- Budget system limits per-task operations to prevent cooperative scheduling abuse

### 3.4 Green Threads / Lightweight Processes

| Runtime | Name | Initial Size | Max Count (practical) | Scheduling |
|---|---|---|---|---|
| BEAM | Process | ~300 bytes + 233 words heap | Millions | Preemptive (reductions) |
| Go | Goroutine | ~2KB stack (growable) | Millions | Cooperative + async preemption |
| Tokio | Task | Future state size (varies) | Millions | Cooperative (budget-based) |
| Pony | Actor | Small (per-actor GC) | Millions | Cooperative + work stealing |
| JVM | Virtual Thread (Loom) | ~1KB | Millions | Cooperative (yield at blocking) |

### 3.5 Scheduling in Actor Systems

Actors introduce scheduling constraints beyond simple task scheduling:
- **Mailbox draining**: how many messages to process before yielding? (Erlang: until reductions exhausted. Pony: one batch per scheduling quantum)
- **Priority actors**: some actors may need higher scheduling priority (Erlang's process priority, Akka's mailbox priority)
- **Fairness across actors**: work stealing ensures no single actor monopolizes a core
- **Timer scheduling**: periodic timer callbacks must be integrated with the message processing loop

Sources:
- [BEAM Scheduler Deep Dive](https://blog.appsignal.com/2024/04/23/deep-diving-into-the-erlang-scheduler.html)
- [Erlang Scheduler Details](https://hamidreza-s.github.io/erlang/scheduling/real-time/preemptive/migration/2016/02/09/erlang-scheduler-details.html)
- [Go Scheduler Design](https://rakyll.org/scheduler/)
- [Go Scheduler Part 2](https://www.ardanlabs.com/blog/2018/08/scheduling-in-go-part2.html)
- [Tokio Cooperative Preemption](https://tokio.rs/blog/2020-04-preemption)
- [Goroutine Preemption](https://hidetatz.github.io/goroutine_preemption/)

---

## 4. Deadlock Detection and Prevention

### 4.1 Conditions for Deadlock

Four conditions (Coffman, 1971) must hold simultaneously:
1. **Mutual exclusion**: resource held exclusively
2. **Hold and wait**: process holds one resource while waiting for another
3. **No preemption**: resources cannot be forcibly taken
4. **Circular wait**: cycle in the wait-for graph

### 4.2 Prevention Strategies

| Strategy | How it works | Trade-off |
|---|---|---|
| Eliminate circular wait | Impose total ordering on resource acquisition | Constrains design |
| Eliminate hold-and-wait | Acquire all resources atomically | Reduces concurrency |
| Allow preemption | Timeout and release held resources | May cause livelock |
| Break mutual exclusion | Use immutable/shared data | Not always possible |

### 4.3 Detection in Distributed Systems

**Wait-For Graph (WFG):** nodes = processes, edges = "waits for". Cycle = deadlock. In distributed systems, no single node has the complete WFG.

**Algorithms:**
- **Chandy-Misra-Haas (1983):** probe-based detection. Blocked process sends probe messages along wait-for edges. If a probe returns to its initiator, a cycle exists. Low message overhead but detection latency.
- **Timeout-based:** if a request doesn't complete within a timeout, assume potential deadlock and abort. Simple but imprecise (false positives). This is what most production systems use.
- **Centralized coordinator:** one node collects the global WFG and detects cycles. Single point of failure; scalability bottleneck.

### 4.4 Actor Model and Deadlock

Pure actor systems (async-only messaging) cannot deadlock in the traditional sense because actors don't hold locks. However, they can **livelock** (actors keep sending messages but make no progress) or **starve** (an actor's mailbox never gets processed).

**Synchronous ask patterns reintroduce deadlock risk**: if actor A asks actor B, and B asks A, both block waiting for each other. Prevention: avoid synchronous ask between actors that might be in a call chain. Use async tell + reply-to pattern instead.

**Erlang's approach**: `gen_server:call` has a default 5-second timeout. If the callee doesn't respond, the caller crashes (which the supervisor handles). This converts potential deadlock into a supervised failure.

Sources:
- [Deadlocks in Distributed Systems](https://bool.dev/blog/detail/deadlocks-in-distributed-systems)
- [Deadlock Detection in Distributed Systems (GeeksforGeeks)](https://www.geeksforgeeks.org/deadlock-detection-in-distributed-systems/)
- [Deadlock Prevention Algorithms (Wikipedia)](https://en.wikipedia.org/wiki/Deadlock_prevention_algorithms)

---

## 5. Backpressure

### 5.1 The Problem

Fast producers overwhelm slow consumers. Without backpressure, the system either drops messages, exhausts memory (unbounded mailbox growth), or crashes.

### 5.2 Strategies

| Strategy | Mechanism | Used by |
|---|---|---|
| Bounded mailbox (block) | Sender blocks when mailbox full | Akka (BoundedMailbox), Go (unbuffered/full channels) |
| Bounded mailbox (drop) | Oldest or newest message dropped | Some Akka configurations |
| Demand-driven pull | Consumer requests N items; producer sends at most N | GenStage, Reactive Streams, Akka Streams |
| Rate limiting | Fixed throughput cap regardless of demand | Application-level |
| Windowed batching | Request multiple items ahead; refill when window partially drained | Akka Streams |
| Credit-based flow control | Receiver issues credits; sender deducts per message | AMQP, custom protocols |
| Load shedding | Reject new requests entirely when overloaded | Orleans (activation shedding) |

### 5.3 Demand-Driven Pull (GenStage)

The most elegant solution for stream processing:

1. Consumer subscribes to producer
2. Consumer sends demand: "give me N events"
3. Producer emits at most N events
4. Consumer processes events, then requests more

This never overwhelms the consumer because the producer cannot push more than requested. The consumer controls the rate.

```
[Producer] <--demand-- [Consumer]
[Producer] --events--> [Consumer]
```

GenStage supports multi-stage pipelines where demand propagates upstream and events flow downstream. Each stage can buffer, batch, or transform independently.

### 5.4 Akka Streams Backpressure

Akka Streams uses a windowed batching strategy:
- Multiple elements can be in-flight simultaneously (not stop-and-wait)
- New demand is not sent per-element but batched (e.g., request 8 more when 4 have been processed)
- This reduces signaling overhead while maintaining backpressure
- Asynchronous boundaries between stages introduce buffering; backpressure signals cross these boundaries

### 5.5 Go Channel Backpressure

Go channels provide natural backpressure:
- **Unbuffered channel**: sender blocks until receiver reads (synchronous handoff)
- **Buffered channel**: sender blocks when buffer is full
- **select + default**: non-blocking send; caller can decide what to do when channel is full

```go
select {
case ch <- msg:
    // sent successfully
default:
    // channel full; drop, log, or apply backpressure
}
```

### 5.6 Actor Mailbox Backpressure Considerations

Traditional actor mailboxes are unbounded. This is a deliberate design choice (actors should always accept messages), but creates memory risk. Solutions:

- Bounded mailbox + sender blocking (breaks pure async model)
- Bounded mailbox + message dropping (data loss)
- Demand-driven protocol layered on top of actor messaging
- Work-pulling pattern: workers pull tasks from a coordinator rather than being pushed tasks

**Work-pulling pattern** (recommended for lx):
```
[Coordinator] has task queue
[Worker 1] --"ready"--> [Coordinator] --task--> [Worker 1]
[Worker 2] --"ready"--> [Coordinator] --task--> [Worker 2]
```

Workers only receive work when they're ready, providing natural backpressure without bounded mailboxes.

Sources:
- [GenStage Back-pressure Mechanism](https://dev.to/dcdourado/understanding-genstage-back-pressure-mechanism-1b0i)
- [Akka and Back-pressure](http://blog.abhinav.ca/blog/2014/01/13/akka-and-backpressure/)
- [Akka Streams Basics](https://doc.akka.io/docs/akka/current/stream/stream-flows-and-basics.html)
- [Backpressure in Message-Based Systems](https://clearmeasure.com/backpressure-in-message-based-systems/)
- [Understanding Backpressure in Event-Driven Architectures](https://www.devx.com/web-development-zone/understanding-backpressure-in-event-driven-architectures/)

---

## 6. Location Transparency

### 6.1 Definition

The ability to interact with an actor (send messages, receive replies) without knowing or caring whether it's in the same process, on the same machine, or on a remote node. The programming model is identical regardless of physical location.

### 6.2 Implementation Approaches

**Erlang/OTP:**
- PIDs encode node information: `<0.42.0>` (local) vs `<node@host.42.0>` (remote)
- `Pid ! Message` works transparently for both
- Distribution protocol handles serialization, connection management, node monitoring
- EPMD (Erlang Port Mapper Daemon) provides node discovery
- `net_kernel` manages connections between nodes
- `monitor_node/2` detects node failures

**Akka:**
- `ActorRef` is location-transparent by design
- "Go from remote to local by way of optimization, not from local to remote by way of generalization"
- All messages must be serializable (even if local, to ensure code works when distributed)
- Akka Cluster provides membership, failure detection, and split-brain resolution
- Cluster Singleton: exactly one instance of an actor across the cluster
- Cluster Sharding: automatic distribution of entities by ID

**Orleans:**
- Grain references are inherently location-transparent
- Grain Directory maps grain ID to current silo
- Automatic activation: if a grain isn't active anywhere, the runtime activates it on an appropriate silo
- Placement strategies determine where new activations go
- Caller never specifies or knows the silo; just calls methods on the grain reference

### 6.3 Serialization Requirement

Location transparency requires all messages to be serializable. This constrains message types:
- No closures or function pointers (in most systems)
- No raw pointers or references
- Rust: `Send + 'static` is necessary but not sufficient; also need `Serialize + Deserialize` for remote
- Erlang: external term format handles all Erlang terms automatically
- Akka: pluggable serializers (Protocol Buffers, Jackson, Kryo)

### 6.4 Trade-offs

Location transparency is not free:
- Serialization overhead (even for local messages in some designs)
- Network latency for remote messages (10-1000x local)
- Partial failure: remote nodes can fail independently
- Message ordering guarantees may weaken across nodes
- Split-brain scenarios require consensus protocols

**Design principle for lx**: maintain location transparency in the programming model (`~>` works the same everywhere), but allow agents to detect locality for optimization.

Sources:
- [Akka Location Transparency](https://doc.akka.io/docs/akka/current/general/remoting.html)
- [Akka.NET Location Transparency](https://getakka.net/articles/concepts/location-transparency.html)
- [Orleans Virtual Actors Paper](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/Orleans-MSR-TR-2014-41.pdf)

---

## 7. Structured Concurrency Patterns for Agents

Structured concurrency is directly relevant to lx's `par` and `sel` primitives.

### 7.1 Par (All Must Succeed)

Corresponds to Trio's nursery, Swift's TaskGroup, Java's ShutdownOnFailure:

```
par {
    result_a <- agent_a ~>? query
    result_b <- agent_b ~>? query
}
// Both results available here; if either fails, both cancel
```

**Invariant**: the `par` block exits only when ALL branches complete. If any branch fails, all others are cancelled and the error propagates.

### 7.2 Sel (First Wins)

Corresponds to Go's `select`, Java's ShutdownOnSuccess, Erlang's selective receive:

```
sel {
    result <- fast_agent ~>? query
    result <- slow_agent ~>? query
}
// First result used; losing branch cancelled
```

**Invariant**: the `sel` block exits when the FIRST branch completes. All other branches are cancelled.

### 7.3 Pmap (Parallel Map)

Bounded parallel execution across a collection:

```
results <- pmap items |item| {
    agent ~>? process(item)
}
```

Corresponds to Go's bounded parallelism pattern (fixed worker pool), Elixir's `Task.async_stream`, Swift's TaskGroup with dynamic fork.

### 7.4 Cancellation Propagation

When a parent agent is cancelled:
1. All child agents receive cancellation signal
2. Children should check for cancellation cooperatively (like Swift `Task.isCancelled`)
3. Children have a grace period to clean up
4. After grace period, forceful termination
5. Parent waits for all children to terminate before itself terminating

This follows the structured concurrency invariant: no child outlives its parent scope.

### 7.5 Supervision Integration

Structured concurrency and supervision trees serve complementary roles:
- **Structured concurrency** (`par`, `sel`, `pmap`): short-lived, scoped concurrent work within a single agent's execution
- **Supervision trees**: long-lived agent lifecycle management across the system

An agent can use `par` internally while itself being supervised. If the supervisor restarts the agent, the in-progress `par` block is cancelled as part of the agent's termination.

Sources:
- [Notes on Structured Concurrency](https://vorpus.org/blog/notes-on-structured-concurrency-or-go-statement-considered-harmful/)
- [Two Approaches to Structured Concurrency](https://www.lesswrong.com/posts/pGySnaGL8WYiDT8vq/two-approaches-to-structured-concurrency)
- [Swift Structured Concurrency](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0304-structured-concurrency.md)
- [JEP 428](https://openjdk.org/jeps/428)
- [Go Pipelines and Cancellation](https://go.dev/blog/pipelines)
