# Agent-to-Agent Streaming (`~>>?`)

`~>>?` is the streaming variant of `~>?` (ask). Where `~>?` sends a request and waits for a single response, `~>>?` sends a request and receives a stream of partial results as the agent produces them.

## Problem

`~>?` is request-response: send message, block until the agent returns a complete result. For long-running agent work (reviewing a large codebase, processing many files, multi-step analysis), the caller blocks with no visibility into progress.

```
results = analyzer ~>? {task: "review" path: "src/"} ^
```

If `analyzer` takes 5 minutes, the caller sees nothing for 5 minutes, then gets everything at once. The caller can't:
- Show progress to the user as files are reviewed
- Start processing early results while later results are still coming
- Cancel early if the first few results show the approach is wrong
- Compose streaming agent output with other streaming operations

## `~>>?` — Streaming Ask

```
stream = analyzer ~>>? {task: "review" path: "src/"}
```

`~>>?` returns a `Stream` — a lazy sequence of values that arrive over time. The agent sends partial results as it works, and the caller consumes them incrementally.

### Stream Type

`Stream a` is an opaque type representing a lazy sequence of `Result a AgentErr` values. It implements the same collection interface as lists for consumption.

### Consumption

```
stream | each (partial) emit "reviewed: {partial.file}"

results = stream | collect ^

first_five = stream | take 5 | collect ^

critical = stream | filter (.severity == "critical") | collect ^

stream | fold [] (acc item) [..acc item]
```

`each`, `take`, `filter`, `fold`, `map` all work on streams. They process elements as they arrive — no buffering of the full result.

`collect` materializes the entire stream into a list (blocks until the agent is done or the stream is closed).

### Error Handling

Each stream element is a `Result`. Errors don't terminate the stream — they flow through as `Err` values:

```
stream | each (item) {
  item ? {
    Ok v  -> process v
    Err e -> log.warn "stream error: {e}"
  }
}
```

For fail-fast, use `^` inside `each`:

```
stream | each (item) {
  v = item ^
  process v
}
```

### Cancellation

Dropping a stream (letting it go out of scope) or using `take` cancels the upstream agent's remaining work:

```
first_problem = analyzer ~>>? {task: "review"} | filter (.severity == "critical") | take 1
```

After `take 1` receives one critical item, the stream closes. The runtime sends a cancellation signal to the agent.

### Timeout

```
stream = timeout 60 (analyzer ~>>?)
```

Per-element timeout using existing `timeout` builtin. If no element arrives within 60 seconds, the stream yields `Err Timeout`.

## Protocol

### Subprocess Protocol

Agent-side: the subprocess writes JSON-lines to stdout with a `"stream"` type:

```json
{"type": "stream", "id": "req-123", "value": {"file": "auth.rs", "issues": 3}}
{"type": "stream", "id": "req-123", "value": {"file": "db.rs", "issues": 0}}
{"type": "stream_end", "id": "req-123"}
```

The `id` field correlates stream elements with the original request. `stream_end` signals completion.

Error elements:
```json
{"type": "stream_error", "id": "req-123", "error": "permission denied: /etc/shadow"}
```

Cancellation (parent to child):
```json
{"type": "stream_cancel", "id": "req-123"}
```

### Agent-Side API

Inside a subprocess agent, use `emit_stream` (new) to send stream elements:

```
use std/agent

items | each (item) {
  result = process item ^
  agent.emit_stream result
}
agent.end_stream ()
```

`agent.emit_stream` writes a `stream` JSON-line. `agent.end_stream` writes `stream_end`. If the agent exits without calling `end_stream`, the runtime sends `stream_end` automatically.

## Patterns

### Incremental code review

```
use std/git
use std/user

stream = reviewer ~>>? {task: "review" diff: git.diff {range: "main..HEAD"} ^}
count = 0
stream | each (r) {
  count := count + 1
  user.progress count r.total "Reviewing {r.file}"
  r.issues | each (i) emit "  [{i.severity}] {i.file}:{i.line} — {i.msg}"
}
```

### Fan-out with streaming results

```
agents | flat_map (a) a ~>>? {task: "scan"}
  | filter (.confidence > 0.8)
  | take 20
  | collect ^
```

Stream from all agents simultaneously. Take the first 20 high-confidence results across all streams, then cancel remaining work.

### Pipeline with streaming stages

```
fetcher ~>>? {urls: urls}
  | map (page) extract_links page
  | flat_map identity
  | filter (link) !visited? link
  | each (link) crawler ~> {url: link}
```

### Streaming with reconcile

```
streams = agents | map (a) a ~>>? {task: "analyze"}
results = streams
  | map (s) s | collect ^
  | agent.reconcile :union
```

Collect all streams, then reconcile. Or reconcile incrementally:

```
agents | flat_map (a) a ~>>? {task: "analyze"}
  | window 10
  | map (batch) agent.reconcile :union batch
```

## Implementation

### Stream Value

New `Value::Stream` variant wrapping a receiver channel:

```rust
Stream {
    rx: Arc<Mutex<mpsc::Receiver<Result<Value, AgentError>>>>,
    cancel_tx: Option<mpsc::Sender<()>>,
}
```

### Parser

`~>>?` is already lexed as a single token (added in Session 31). Parser produces `Expr::StreamAsk { agent, message }`. Same precedence as `~>?`.

### Interpreter

`eval_stream_ask`:
1. Send the request to the agent subprocess (same as `~>?`)
2. Return a `Value::Stream` immediately (don't wait for response)
3. Background thread reads JSON-lines from subprocess stdout, filters for matching `id`, sends to channel
4. On `stream_end` or subprocess exit, close the channel

### Collection Operations on Streams

`each`, `map`, `filter`, `take`, `fold`, `collect` all check if their input is `Value::Stream`. If so, they pull from the channel instead of iterating a list. This means no new syntax — existing operations work on streams transparently.

### Dependencies

- `std::sync::mpsc` (bounded channel for stream elements)
- Existing subprocess infrastructure from `std/agent`

## Cross-References

- Request-response: `~>?` in [stdlib-agents.md](stdlib-agents.md)
- Data streaming: `|>>` in [concurrency-reactive.md](concurrency-reactive.md) — data-level vs agent-level
- Pipelines: [agents-pipeline.md](agents-pipeline.md) — agent pipelines with backpressure
- User progress: [stdlib-user.md](stdlib-user.md) — render stream progress to humans
- Cancellation: relates to `with ... as` scoped cleanup
