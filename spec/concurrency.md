# Concurrency

Structured concurrency only — no unstructured spawn/await. Every concurrent operation has a clear scope and lifetime. This prevents dangling futures, which are exactly the kind of state-tracking bug I'm worst at.

These primitives are also the foundation for agentic workflows — `par` runs parallel agent tasks, `sel` races agent responses against timeouts, `pmap` fans out to agent pools. See [agents.md](agents.md) for agent-specific patterns built on these constructs.

## `pmap` — Parallel Map

The most common concurrent pattern: apply a function to every element in parallel.

```
results = urls | pmap fetch
results = urls | pmap (url) http.get url ^
```

`pmap` has the same signature as `map` but runs the function concurrently across elements. Order of results matches order of inputs.

Default parallelism: number of CPU cores. All elements are spawned concurrently; the runtime manages scheduling.

If any element fails (returns `Err`), the remaining in-flight operations are cancelled and the error propagates.

## `par` — Parallel Block

Run independent expressions concurrently and collect all results as a tuple:

```
(a b c) = par {
  fetch url1 ^
  fetch url2 ^
  fetch url3 ^
}
```

Each statement in a `par` block runs concurrently. The block completes when all statements finish. Results are returned as a tuple in order.

Error behavior: if any expression in a `par` block errors, all siblings are cancelled and the error propagates to the caller. This is structured concurrency — no zombie tasks.

## `sel` — Select (Race)

Run concurrent expressions and take the first to complete. All others are cancelled.

```
result = sel {
  fetch url   -> Ok it
  timeout 5   -> Err "timed out"
}
```

Each arm is `expr -> handler`. The expressions run concurrently. When the first completes, its result is bound to `it` in the handler, the handler runs, and all other arms are cancelled.

Common use: timeouts.

```
resp = sel {
  http.get slow_api -> it
  timeout 30       -> Err "api took too long"
}
```

## Cancellation

Cancellation is cooperative. When a `par` or `sel` block cancels a sibling:
- In-flight shell commands receive SIGTERM
- In-flight HTTP requests are aborted
- In-flight `par`/`sel` blocks are recursively cancelled
- The cancelled expression's result is discarded

## Patterns

Fan-out/fan-in:

```
raw_results = urls | pmap fetch
processed = raw_results | filter ok? | map (?? ()) | sort_by (.date)
```

Parallel with timeout:

```
(users posts) = par {
  sel { fetch_users ^ -> it; timeout 5 -> Err "users timeout" }
  sel { fetch_posts ^ -> it; timeout 5 -> Err "posts timeout" }
}
```

Batch processing with controlled parallelism:

```
items | chunks 10 | each (batch) {
  batch | pmap process
}
```

## `sel` Binding

In `sel` arms, `it` refers to the result of the completed expression:

```
result = sel {
  fetch url   -> it            -- it = return value of (fetch url)
  timeout 5   -> Err "timeout"
}
```

`it` is implicitly bound in each handler. You can also destructure:

```
sel {
  http.get url -> it.body | json.parse ^
  timeout 30   -> Err "too slow"
}
```

## Error Handling in Concurrent Blocks

`par` fails fast: the first error cancels all siblings and propagates.

```
(a b) = par {
  fetch url1 ^    -- if this errors, url2 fetch is cancelled
  fetch url2 ^
}
```

To collect results independently (some may fail):

```
results = urls | pmap (url) fetch url    -- [Result a e], no ^
successes = results | filter ok? | map (?? ())
failures = results | filter err? | map (r) r ? { Err e -> e; _ -> () }
```

Without `^` in the `pmap` body, individual failures become `Err` values in the result list instead of cancelling the whole operation.

## Concurrency Limits

`pmap` runs all elements concurrently by default (the runtime manages scheduling across CPU cores). For I/O-bound work with external rate limits, use `pmap_n`:

```
results = urls | pmap_n 10 fetch
```

`pmap_n limit f xs` runs at most `limit` concurrent tasks at a time. When a task completes, the next element is spawned. Results are still returned in input order.

For more complex rate limiting (delays between batches), batch manually:

```
urls | chunks 10 | each (batch) {
  results = batch | pmap fetch
  time.sleep (time.ms 100)
}
```

## Mutable State Restriction

Capturing a mutable binding (`:=`) inside a `par`, `sel`, or `pmap` body is a **compile error**:

```
count := 0
-- ERROR: cannot capture mutable `count` in concurrent context
xs | pmap (x) { count <- count + 1; process x }
```

This prevents data races. If you need to aggregate results from concurrent work, collect the results first, then process sequentially:

```
results = xs | pmap process
total = results | sum
```

Mutable bindings defined *inside* the concurrent body are fine — they're local to each concurrent task:

```
xs | pmap (x) {
  acc := 0              -- local to each task, no sharing
  acc <- acc + x
  acc * 2
}
```

## Runtime Model

**Current implementation**: `par`, `sel`, and `pmap` execute sequentially — statements run one at a time. This simplifies the interpreter but means no actual parallelism. Real concurrent execution via `tokio` is planned for a future phase.

The `LX_THREADS` env var is reserved for future concurrent execution.

## Event-Driven Concurrency

For reactive patterns where agents respond to events rather than direct messages, use `std/events` (pub/sub). For shared mutable state across concurrent agents, use `std/blackboard`. See [stdlib-modules.md](stdlib-modules.md) for APIs.

```
use std/events
use std/blackboard

bus = events.create ()
board = blackboard.create ()

par {
  events.subscribe bus "result" (evt) blackboard.write evt.key evt.val board
  worker1 ~>? {task: "part1" bus} ^
  worker2 ~>? {task: "part2" bus} ^
}
```

## Cross-References

- Agent patterns built on par/sel/pmap: [agents.md](agents.md)
- Agent streaming (`~>>?`): [agents.md](agents.md#streaming)
- Agent stdlib (spawn, ask, channel): [stdlib-agents.md](stdlib-agents.md)
- Shared workspace: [stdlib-modules.md](stdlib-modules.md#stdblackboard)
- Pub/sub events: [stdlib-modules.md](stdlib-modules.md#stdevents)
- Implementation: [impl-interpreter.md](../design/impl-interpreter.md) (par/sel/pmap evaluation), [impl-builtins.md](../design/impl-builtins.md) (pmap built-in)
- Design decisions: [design.md](design.md) (structured concurrency, mutable capture restriction)
- Test suite: [13_concurrency.lx](../tests/13_concurrency.lx)
