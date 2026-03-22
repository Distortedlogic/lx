**Depends on: CHANNELS_CSP (optional — channel integration requires channels to exist, but core streams work independently)**

# Goal

Add lazy stream abstractions to lx so pipeline operations (`map`, `filter`, `fold`) can process data incrementally without materializing entire intermediate lists. Currently `items | map f | filter p | fold init g` creates 3 full lists in memory. For agentic workflows processing large result sets or streaming LLM output, this wastes memory and delays first results.

# Why

- Lazy evaluation is well-established across languages (Haskell, Clojure, Elixir, Rust iterators). All emphasize lazy pipelines for performance.
- LLM responses can be thousands of tokens. Processing them as a stream (line by line, chunk by chunk) lets downstream consumers start working immediately.
- `channel.recv` (from CHANNELS_CSP work item) naturally produces streams — lazy streams would compose with channels.
- Current eager pipelines hit memory limits on large datasets (e.g., processing all files in a repository).

# What Changes

## New stdlib module: `std/stream`

New file `crates/lx/src/stdlib/stream.rs` implementing 8 functions:

**`stream.from source -> Stream`** — Creates a stream from a source. `source` can be: a List (iterates elements), a Func (called repeatedly until it returns None), or a channel Receiver (reads until closed).

**`stream.map stream f -> Stream`** — Lazy map: applies `f` to each element when consumed, not when created.

**`stream.filter stream pred -> Stream`** — Lazy filter: skips elements where `pred` returns false.

**`stream.take stream n -> Stream`** — Takes first `n` elements, then stops.

**`stream.batch stream size -> Stream`** — Groups elements into lists of `size`. Last batch may be smaller.

**`stream.collect stream -> [Any]`** — Forces the stream, collecting all elements into a list.

**`stream.each stream f -> ()`** — Consumes the stream, calling `f` on each element for side effects.

**`stream.fold stream init f -> Any`** — Consumes the stream, folding with `f(acc, elem)`.

## Stream representation

A Stream is a record `{_type: :stream, _id: Int}` backed by a global registry of stream states (following the same pattern as `std/store`, which uses `LazyLock<DashMap<u64, StoreState>>` + `AtomicU64`). Each stream state holds the source data, current index, and any transformation chain. The `_next` operation is performed by Rust-side `bi_collect`/`bi_each`/`bi_fold` consuming the stream, not by an lx-level `_next` function field.

**Why not record-with-`_next`-function:** `SyncBuiltinFn` is a plain `fn` pointer (`fn(&[LxVal], SourceSpan, &Arc<RuntimeCtx>) -> Result<LxVal, LxError>`) — it cannot capture per-stream state like `Arc<AtomicUsize>`. The store module solves this same problem with global `DashMap` keyed by ID, and streams should follow suit.

Stream combinators (`map`, `filter`, `take`, `batch`) create new stream IDs whose state references the inner stream ID plus the transformation function. Terminal operations (`collect`, `each`, `fold`) drive the pull loop from Rust.

## Integration with pipes

Streams work with the existing pipe operator: `stream.from list | stream.map f | stream.filter p | stream.collect`.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/stream.rs` — all stream functions
- `tests/86_streams.lx` — tests for lazy stream behavior

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — register `mod stream;`, add `"stream" => stream::build()` to `get_std_module`, add `"stream"` to `std_module_exists`

# Task List

### Task 1: Implement stream.from and stream.collect

**Subject:** Create streams from lists and functions, force evaluation with collect

**Description:** Create `crates/lx/src/stdlib/stream.rs`. Set up a global stream registry following the `std/store` pattern: `static STREAMS: LazyLock<DashMap<u64, StreamState>>` and `static NEXT_ID: AtomicU64`.

Define `StreamState` enum variants:
- `FromList { items: Arc<Vec<LxVal>>, index: AtomicUsize }` — iterates a list
- `FromFunc { func: LxVal }` — calls func repeatedly until None
- `Map { inner_id: u64, func: LxVal }` — lazy map (Task 2)
- `Filter { inner_id: u64, pred: LxVal }` — lazy filter (Task 2)
- `Take { inner_id: u64, remaining: AtomicUsize }` — take n (Task 2)
- `Batch { inner_id: u64, size: usize }` — batch (Task 3)

Implement `bi_from(source)`:
- If source is `LxVal::List`: allocate a new stream ID, insert `FromList` state, return `record!{"_type" => LxVal::str("stream"), "_id" => LxVal::int(id)}`.
- If source is `LxVal::Func` or `LxVal::BuiltinFunc`: allocate ID, insert `FromFunc` state, return stream record.

Implement internal `fn pull_next(id: u64, span, ctx) -> Result<LxVal, LxError>` that matches on the `StreamState` variant and returns `Some val` or `None`.

Implement `bi_collect(stream)`:
- Extract `_id` from the stream record.
- Loop: call `pull_next(id, ...)`, if `Some val` append to result list, if `None` return the list.

Register module in `stdlib/mod.rs`: add `mod stream;`, add `"stream" => stream::build()` to `get_std_module`, add `"stream"` to `std_module_exists`. Add `"from"` and `"collect"` to `build()`.

Run `just diagnose`.

**ActiveForm:** Implementing stream.from and stream.collect

---

### Task 2: Implement stream.map, stream.filter, stream.take

**Subject:** Lazy transformation combinators for streams

**Description:** In `crates/lx/src/stdlib/stream.rs`:

`bi_map(stream, f)`:
- Extract `_id` from stream record. Allocate new stream ID, insert `Map { inner_id, func: f }` state. Return new stream record.
- Update `pull_next` to handle `Map`: call `pull_next(inner_id, ...)`, if `Some val`, call `call_value_sync(&func, val, span, ctx)` and wrap in `Some`, else return `None`.

`bi_filter(stream, pred)`:
- Extract `_id`, allocate new ID, insert `Filter { inner_id, pred }` state.
- In `pull_next`: loop — call `pull_next(inner_id, ...)`, if `Some val` and `call_value_sync(&pred, val, span, ctx)` is truthy, return `Some val`. If not truthy, continue loop. If `None`, return `None`.

`bi_take(stream, n)`:
- Extract `_id`, allocate new ID with `Take { inner_id, remaining: AtomicUsize::new(n) }`.
- In `pull_next`: if `remaining.load() > 0`, decrement and delegate to `pull_next(inner_id, ...)`, else return `None`.

Add `"map"`, `"filter"`, `"take"` to `build()`.

Run `just diagnose`.

**ActiveForm:** Implementing lazy map, filter, take

---

### Task 3: Implement stream.batch, stream.each, stream.fold, and write tests

**Subject:** Batching, consumption functions, and test suite

**Description:** Implement:

`bi_batch(stream, size)`: Extract `_id`, allocate new ID with `Batch { inner_id, size }`. In `pull_next`: collect up to `size` elements from `pull_next(inner_id, ...)` into a list, return `Some [batch]` or `None` if inner is exhausted and batch is empty. Partial last batch returns `Some [partial]`.

`bi_each(stream, f)`: extract `_id`, loop `pull_next(id, ...)`, call `call_value_sync(&f, elem, span, ctx)` for each `Some val`. Return `Unit`.

`bi_fold(stream, init, f)`: extract `_id`, loop `pull_next(id, ...)`, accumulate with `call_value_sync(&f, Tuple(acc, elem), span, ctx)`. Return final accumulator.

Add all to `build()`.

Create `tests/86_streams.lx`:
1. **from + collect roundtrip** — `stream.from [1 2 3] | stream.collect` equals `[1 2 3]`.
2. **Lazy map** — `stream.from [1 2 3] | stream.map (x) x * 2 | stream.collect` equals `[2 4 6]`.
3. **Lazy filter** — filter evens from `[1 2 3 4 5]`, collect, equals `[2 4]`.
4. **Lazy composition** — map then filter then collect without materializing intermediates.
5. **take** — `stream.from [1 2 3 4 5] | stream.take 3 | stream.collect` equals `[1 2 3]`.
6. **batch** — batch `[1 2 3 4 5]` by 2, collect, equals `[[1 2] [3 4] [5]]`.
7. **fold** — fold stream with sum, verify result.
8. **Generator function** — `stream.from` with a stateful function that counts 1..5 then returns None.

Run `just diagnose` and `just test`.

**ActiveForm:** Implementing batch, each, fold, and writing tests

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/LAZY_STREAMS.md" })
```

Then call `next_task` to begin.
