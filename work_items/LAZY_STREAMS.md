**Depends on: CHANNELS_CSP (optional — channel integration requires channels to exist, but core streams work independently)**

# Goal

Add lazy stream abstractions to lx so pipeline operations (`map`, `filter`, `fold`) can process data incrementally without materializing entire intermediate lists. Currently `items | map f | filter p | fold init g` creates 3 full lists in memory. For agentic workflows processing large result sets or streaming LLM output, this wastes memory and delays first results.

# Why

- The research in `research/pipes/` covers lazy evaluation across 15+ languages (Haskell, Clojure, Elixir, Rust iterators). All emphasize lazy pipelines for performance.
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

A Stream is a record with an internal `_next` function: `{_type: :stream, _next: Func}`. The `_next` function returns `Some value` for the next element or `None` when exhausted. Stream combinators wrap the inner `_next` with transformation logic:

```
stream.map s f = {
  _type: :stream
  _next: () { s._next () ? { Some val -> Some (f val); None -> None } }
}
```

This keeps streams as pure lx values — no new Rust types needed.

## Integration with pipes

Streams work with the existing pipe operator: `stream.from list | stream.map f | stream.filter p | stream.collect`.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/stream.rs` — all stream functions
- `tests/83_streams.lx` — tests for lazy stream behavior

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — register `mod stream;`, add to `get_std_module`

# Task List

### Task 1: Implement stream.from and stream.collect

**Subject:** Create streams from lists and functions, force evaluation with collect

**Description:** Create `crates/lx/src/stdlib/stream.rs`. Implement:

`bi_from(source)`:
- If source is `LxVal::List`: create an internal index counter, return a stream record where `_next` returns `Some list[i]` and increments i, or `None` when exhausted. The index state is held in a `Arc<AtomicUsize>` captured by the closure.
- If source is `LxVal::Func` or `LxVal::BuiltinFunc`: return a stream record where `_next` calls the function with no args and returns the result directly (caller's function should return `Some val` or `None`).

`bi_collect(stream)`:
- Extract `_next` from the stream record.
- Loop: call `_next()`, if `Some val` append to result list, if `None` return the list.

Register module in `stdlib/mod.rs`. Add `"from"` and `"collect"` to `build()`.

Run `just diagnose`.

**ActiveForm:** Implementing stream.from and stream.collect

---

### Task 2: Implement stream.map, stream.filter, stream.take

**Subject:** Lazy transformation combinators for streams

**Description:** In `crates/lx/src/stdlib/stream.rs`:

`bi_map(stream, f)`:
- Create a new stream where `_next` calls the inner stream's `_next`, and if `Some val`, returns `Some (f val)`, else `None`.

`bi_filter(stream, pred)`:
- Create a new stream where `_next` loops: call inner `_next`, if `Some val` and `pred val` is truthy, return `Some val`. If `Some val` but pred is false, continue loop. If `None`, return `None`.

`bi_take(stream, n)`:
- Track count in `Arc<AtomicUsize>`. `_next` checks count < n, if so increment and delegate to inner `_next`, else return `None`.

Add `"map"`, `"filter"`, `"take"` to `build()`.

Run `just diagnose`.

**ActiveForm:** Implementing lazy map, filter, take

---

### Task 3: Implement stream.batch, stream.each, stream.fold, and write tests

**Subject:** Batching, consumption functions, and test suite

**Description:** Implement:

`bi_batch(stream, size)`: `_next` collects up to `size` elements from inner stream into a list, returns `Some [batch]` or `None` if inner is exhausted and batch is empty. Partial last batch returns `Some [partial]`.

`bi_each(stream, f)`: consume stream, call `f(elem)` for each. Return `Unit`.

`bi_fold(stream, init, f)`: consume stream, accumulate with `f(acc, elem)`. Return final accumulator.

Add all to `build()`.

Create `tests/83_streams.lx`:
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
