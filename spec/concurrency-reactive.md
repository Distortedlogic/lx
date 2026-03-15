# Reactive Dataflow

Streaming pipelines where results flow downstream as they become available, not after the entire upstream completes. This fills the gap between `par` (all-at-once, wait-for-all) and sequential execution (one-at-a-time).

## Problem

When researching a codebase, an agent searches for X, and X's result tells it to search for Y and Z, and Z triggers W. This is a reactive dataflow graph. `par` can't express "as results arrive, feed them downstream." Sequential execution can't express "start downstream before upstream finishes."

```
files = glob "src/**/*.rs"
contents = files | map read_file
matches = contents | map (grep "TODO")
```

This reads ALL files, THEN greps ALL contents. If there are 1000 files, the agent waits for all 1000 reads before any grep starts.

## `|>>` — Streaming Pipe

`|>>` is a binary operator that pushes items downstream as they complete. Same precedence as `|`.

```
results = glob "src/**/*.rs"
  |>> fs.read
  |>> (content) re.find_all r/TODO.*/ content
  |>> classify_todo
```

Each file is read as `glob` yields it. Each content is grepped as `read` completes. Each match is classified as `grep` finds it. The pipeline is lazy — downstream stages start before upstream finishes.

### Semantics

- `xs |>> f` applies `f` to each element of `xs` as it becomes available
- Returns a lazy sequence (like `~>>?` streams)
- Elements flow through in order of completion, not input order
- To preserve input order: `xs |>> f | sort_by (.index)` (caller's responsibility)
- Errors in any stage propagate as `Err` values in the output stream — they do not cancel the pipeline (unlike `pmap` with `^`)
- The entire pipeline is lazy — nothing executes until the result is consumed (by `each`, `collect`, `fold`, etc.)

### Composition

```
glob "src/**/*.rs"
  |>> fs.read
  |>> (content) re.find_all r/TODO.*/ content
  |>> classify_todo
  | collect
  | group_by (.severity)
```

`|>>` stages are streaming. The final `|` (regular pipe) triggers collection — `collect` materializes the lazy stream into a list. After collection, normal eager operations apply.

### With Concurrency

```
urls |>> fetch                   -- sequential streaming (one at a time)
urls |>> par_n 5 fetch           -- concurrent streaming (5 at a time)
```

`|>>` itself is sequential — one element at a time flows through. For concurrent streaming, use `par_n` (a streaming variant of `pmap_n`):

- `par_n limit f` returns a streaming function that processes up to `limit` items concurrently
- Results arrive in completion order
- Composes with `|>>`: `xs |>> par_n 10 fetch |>> process`

### With Agent Communication

```
agents |>> (a) a ~>? {task: "review"} |>> aggregate
```

Ask each agent as the agent list becomes available. Aggregate each response as it arrives.

### Backpressure

If a downstream stage is slower than upstream, the upstream blocks (does not buffer unboundedly). This prevents memory exhaustion on large datasets.

The buffer size is configurable per stage:

```
xs |>> buffer 100 fetch |>> process
```

`buffer n f` allows up to `n` items to queue between stages. Default buffer: 1 (pure streaming, no lookahead).

### Cancellation

Consuming only part of a streaming pipeline cancels the rest:

```
glob "src/**/*.rs"
  |>> fs.read
  |>> (content) re.find_all r/TODO.*/ content
  | take 10
  | collect
```

After `take 10` has 10 items, upstream stages are cancelled. Resources are cleaned up via `defer` semantics.

### Error Handling

```
results = urls |>> fetch |>> process | collect

successes = results | filter ok?
failures = results | filter err?
```

Errors flow through as `Err` values. No short-circuit — the pipeline continues processing other elements. Use `filter ok?` / `filter err?` to separate after collection.

For fail-fast behavior, consume with `each` and `^`:

```
urls |>> fetch |>> (r) { r ^; process r } | each identity
```

## `collect` Built-in

`collect` materializes a lazy stream into a list. It is the bridge between streaming (`|>>`) and eager (`|`) pipelines.

```
stream |>> transform | collect | sort_by (.date)
```

Without `collect`, a `|>>` pipeline remains lazy. `collect` forces evaluation of all remaining elements.

## Implementation Notes

Current: `|>>` desugars to sequential `each` + accumulator. No true async streaming. Real streaming requires the tokio runtime (same dependency as real `par`/`sel`).

`|>>` is a new binary operator at the same precedence level as `|`. The parser recognizes `|>>` as a single token (not `|` followed by `>>`).

## Cross-References

- Eager pipelines: [concurrency.md](concurrency.md) (par/sel/pmap)
- Agent streaming: [agents.md](agents.md) (~>>?)
- Backpressure question: [open-questions.md](open-questions.md)
