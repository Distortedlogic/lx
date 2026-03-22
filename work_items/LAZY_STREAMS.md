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

A Stream is an `LxVal::Stream { id: u64 }` variant backed by a global registry of stream states (following the same pattern as `std/store`, which uses `LazyLock<DashMap<u64, StoreState>>` + `AtomicU64` and returns `LxVal::Store { id }`). Each stream state holds the source data, current index, and any transformation chain. The `_next` operation is performed by Rust-side `bi_collect`/`bi_each`/`bi_fold` consuming the stream, not by an lx-level `_next` function field.

**Why a dedicated variant instead of record-with-`_next`-function:** `SyncBuiltinFn` is a plain `fn` pointer (`fn(&[LxVal], miette::SourceSpan, &Arc<crate::runtime::RuntimeCtx>) -> Result<LxVal, LxError>`) — it cannot capture per-stream state like `Arc<AtomicUsize>`. The store module solves this same problem with a dedicated `LxVal::Store { id }` variant + global `DashMap` keyed by ID, and streams follow suit with `LxVal::Stream { id }`.

Stream combinators (`map`, `filter`, `take`, `batch`) create new stream IDs whose state references the inner stream ID plus the transformation function. Terminal operations (`collect`, `each`, `fold`) drive the pull loop from Rust.

## Integration with pipes

Streams work with the existing pipe operator: `stream.from list | stream.map f | stream.filter p | stream.collect`.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/stream.rs` — all stream functions
- `tests/86_streams.lx` — tests for lazy stream behavior (the `tests/` directory at repo root must be created if it does not exist; `just test` expects it)

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — register `mod stream;`, add `"stream" => stream::build()` to `get_std_module`, add `"stream"` to `std_module_exists`
- `crates/lx/src/value/mod.rs` — add `Stream { id: u64 }` variant to the `LxVal` enum (following the `Store { id: u64 }` pattern at line 99)

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

Add `Stream { id: u64 }` variant to `LxVal` in `crates/lx/src/value/mod.rs` (after the `Store` variant). Update all trait impls that match on `LxVal` variants — follow the `Store` pattern in each file:
- `crates/lx/src/value/display.rs` — add `LxVal::Stream { id } => write!(f, "<Stream#{id}>")`
- `crates/lx/src/value/impls.rs` — add `PartialEq` arm `(LxVal::Stream { id: i1 }, LxVal::Stream { id: i2 }) => i1 == i2` and `Hash` arm `LxVal::Stream { id } => id.hash(state)`
- `crates/lx/src/value/serde_impl.rs` — add `LxVal::Stream { id } => marker_map!(serializer, ("__stream", id))`

Implement `bi_from(source)`:
- If source is `LxVal::List`: allocate a new stream ID, insert `FromList` state, return `LxVal::Stream { id }`.
- If source is `LxVal::Func` or `LxVal::BuiltinFunc`: allocate ID, insert `FromFunc` state, return `LxVal::Stream { id }`.

Implement a helper `fn stream_id(v: &LxVal, span: SourceSpan) -> Result<u64, LxError>` that extracts `id` from `LxVal::Stream { id }` (following the `store_id` helper pattern in `store/mod.rs`).

Implement internal `fn pull_next(id: u64, span, ctx) -> Result<LxVal, LxError>` that matches on the `StreamState` variant and returns `LxVal::Some(val)` or `LxVal::None`.

Implement `bi_collect(stream)`:
- Extract `id` via `stream_id`.
- Loop: call `pull_next(id, ...)`, if `LxVal::Some(val)` append to result list, if `LxVal::None` return the list.

Register module in `stdlib/mod.rs`: add `mod stream;`, add `"stream" => stream::build()` to `get_std_module`, add `"stream"` to `std_module_exists`. Add `"from"` and `"collect"` to `build()`.

Run `just diagnose`.

**ActiveForm:** Implementing stream.from and stream.collect

---

### Task 2: Implement stream.map, stream.filter, stream.take

**Subject:** Lazy transformation combinators for streams

**Description:** In `crates/lx/src/stdlib/stream.rs`:

`bi_map(stream, f)`:
- Extract `id` via `stream_id`. Allocate new stream ID, insert `Map { inner_id, func: f }` state. Return `LxVal::Stream { id: new_id }`.
- Update `pull_next` to handle `Map`: call `pull_next(inner_id, ...)`, if `LxVal::Some(val)`, call `call_value_sync(&func, val, span, ctx)` and wrap in `LxVal::Some`, else return `LxVal::None`.

`bi_filter(stream, pred)`:
- Extract `id` via `stream_id`, allocate new ID, insert `Filter { inner_id, pred }` state. Return `LxVal::Stream { id: new_id }`.
- In `pull_next`: loop — call `pull_next(inner_id, ...)`, if `LxVal::Some(val)` and `call_value_sync(&pred, val, span, ctx)` is truthy, return `LxVal::Some(val)`. If not truthy, continue loop. If `LxVal::None`, return `LxVal::None`.

`bi_take(stream, n)`:
- Extract `id` via `stream_id`, allocate new ID with `Take { inner_id, remaining: AtomicUsize::new(n) }`. Return `LxVal::Stream { id: new_id }`.
- In `pull_next`: if `remaining.load() > 0`, decrement and delegate to `pull_next(inner_id, ...)`, else return `LxVal::None`.

Add `"map"`, `"filter"`, `"take"` to `build()`.

Run `just diagnose`.

**ActiveForm:** Implementing lazy map, filter, take

---

### Task 3: Implement stream.batch, stream.each, stream.fold, and write tests

**Subject:** Batching, consumption functions, and test suite

**Description:** Implement:

`bi_batch(stream, size)`: Extract `id` via `stream_id`, allocate new ID with `Batch { inner_id, size }`. Return `LxVal::Stream { id: new_id }`. In `pull_next`: collect up to `size` elements from `pull_next(inner_id, ...)` into a list, return `LxVal::Some(LxVal::list(batch))` or `LxVal::None` if inner is exhausted and batch is empty. Partial last batch returns `LxVal::Some(LxVal::list(partial))`.

`bi_each(stream, f)`: extract `id` via `stream_id`, loop `pull_next(id, ...)`, call `call_value_sync(&f, elem, span, ctx)` for each `LxVal::Some(val)`. Return `LxVal::Unit`.

`bi_fold(stream, init, f)`: extract `id` via `stream_id`, loop `pull_next(id, ...)`, accumulate with `call_value_sync(&f, LxVal::Tuple(Arc::new(vec![acc, elem])), span, ctx)`. Return final accumulator.

Add all to `build()`.

Create `tests/` directory at repo root if it does not exist (it currently does not; `just test` expects `tests/`). Then create `tests/86_streams.lx`:
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
