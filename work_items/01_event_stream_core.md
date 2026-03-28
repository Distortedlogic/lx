# Work Item: Event Stream Core + std/events Module

## Goal

Build the in-memory event stream data structure, JSONL persistence layer, and `std/events` module exposing `xadd`, `xrange`, `xread`, `xlen`, `xtrim` to lx programs. Wire the `EventStream` into `RuntimeCtx` so all interpreters share it.

## Preconditions

- `crates/lx/src/runtime/mod.rs` exists with the current `RuntimeCtx` struct (fields: `emit`, `http`, `yield_`, `log`, `llm`, `source_dir`, `workspace_members`, `dep_dirs`, `tokio_runtime`, `test_threshold`, `test_runs`)
- `crates/lx/src/stdlib/mod.rs` exists with `get_std_module` and `std_module_exists` functions
- `crates/lx/src/stdlib/channel.rs` exists as a reference pattern for stdlib modules
- `crates/lx/src/value/mod.rs` exists with `LxVal` enum, `LxVal::record()`, `LxVal::list()`, `LxVal::str()`, `LxVal::int()`, `LxVal::None`
- `LxVal` implements `Serialize` (via manual impl in `crates/lx/src/value/serde_impl.rs` line 19). The `StreamEntry.fields` map has `LxVal` values, which serialize correctly — the existing `Serialize` impl has a `_ =>` catch-all arm at line 62 that handles any variant.
- `crates/lx/src/stdlib/helpers.rs` exists with `extract_handle_id` and `std_module!` macro
- `tokio` is already a workspace dependency (used throughout the crate)
- `serde_json` is already a workspace dependency
- `parking_lot` is already a workspace dependency
- `indexmap` is already a workspace dependency
- `chrono` is already a workspace dependency

## Files to Create

### 1. `crates/lx/src/event_stream.rs`

This is the core event stream data structure. Must be under 300 lines.

**Struct: `EventStream`**

```rust
pub struct EventStream {
    entries: parking_lot::RwLock<Vec<StreamEntry>>,
    seq_counter: std::sync::atomic::AtomicU64,
    last_ms: std::sync::atomic::AtomicU64,
    notify: tokio::sync::Notify,
    jsonl_path: Option<parking_lot::Mutex<std::io::BufWriter<std::fs::File>>>,
}
```

**Struct: `StreamEntry`**

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct StreamEntry {
    pub id: String,
    pub kind: String,
    pub agent: String,
    pub ts: u64,
    pub span: Option<SpanInfo>,
    pub fields: indexmap::IndexMap<crate::sym::Sym, crate::value::LxVal>,
}
```

**Struct: `SpanInfo`**

```rust
#[derive(Debug, Clone, serde::Serialize)]
pub struct SpanInfo {
    pub line: usize,
    pub col: usize,
}
```

**Methods on `EventStream`:**

- `pub fn new(jsonl_path: Option<std::path::PathBuf>) -> Self`
  - If `jsonl_path` is `Some`, create parent dirs if needed, open the file in append mode, wrap in `BufWriter`
  - Initialize `seq_counter` to 0, `last_ms` to 0
  - Create a `tokio::sync::Notify` instance

- `pub fn xadd(&self, kind: &str, agent: &str, span: Option<SpanInfo>, fields: indexmap::IndexMap<crate::sym::Sym, crate::value::LxVal>) -> String`
  - Get current wall clock ms: `chrono::Utc::now().timestamp_millis() as u64`
  - Generate ID: load `last_ms`. If current ms equals `last_ms`, increment `seq_counter`. If current ms is greater, store new ms in `last_ms` and reset `seq_counter` to 0. Use `Ordering::SeqCst` for both atomics.
  - Format ID as `"{ms}-{seq}"`
  - Build a `StreamEntry { id: id.clone(), kind: kind.to_string(), agent: agent.to_string(), ts: ms, span, fields }`
  - Push entry to `self.entries` write lock
  - If `jsonl_path` writer exists, serialize the entry as JSON via `serde_json::to_string`, write line + `\n`, flush
  - Call `self.notify.notify_waiters()`
  - Return the `id` string

- `pub fn xrange(&self, start: &str, end: &str, count: Option<usize>) -> Vec<StreamEntry>`
  - Read lock `self.entries`
  - `"-"` means index 0 (beginning), `"+"` means last entry (end)
  - For numeric IDs, compare lexicographically (the `{ms}-{seq}` format sorts correctly as strings)
  - Filter entries where `entry.id >= start && entry.id <= end`
  - If `count` is `Some(n)`, take at most `n` entries
  - Return cloned entries

- `pub async fn xread(&self, last_id: &str, timeout_ms: Option<u64>) -> Option<StreamEntry>`
  - If `last_id` is `"$"`, set effective_last to the ID of the last entry in the stream (or `""` if empty)
  - Otherwise, effective_last = last_id
  - First check: read lock entries, find first entry with `entry.id > effective_last`. If found, return `Some(entry.clone())`
  - If not found, enter a wait loop:
    - If `timeout_ms` is `Some(ms)`, use `tokio::select!` between `self.notify.notified()` and `tokio::time::sleep(Duration::from_millis(ms))`
    - If `timeout_ms` is `None`, just `self.notify.notified().await`
    - After notification, re-check entries for `entry.id > effective_last`
    - On timeout, return `None`

- `pub fn xlen(&self) -> usize`
  - Read lock `self.entries`, return `.len()`

- `pub fn xtrim(&self, maxlen: usize)`
  - Write lock `self.entries`
  - If `entries.len() > maxlen`, drain from the front: `entries.drain(..entries.len() - maxlen)`

**Helper: `entry_to_lxval`**

```rust
pub fn entry_to_lxval(entry: &StreamEntry) -> crate::value::LxVal
```
- Build an `IndexMap<Sym, LxVal>` with keys: `"id"`, `"kind"`, `"agent"`, `"ts"`, `"span"` (record with `line`/`col` or `None`), plus all keys from `entry.fields`
- Return `LxVal::record(map)`

**ID comparison helper:**

```rust
fn id_ge(a: &str, b: &str) -> bool
fn id_gt(a: &str, b: &str) -> bool
```
- Parse both sides as `(ms: u64, seq: u64)` by splitting on `"-"`
- Compare `(ms_a, seq_a)` vs `(ms_b, seq_b)` numerically
- If parsing fails, fall back to string comparison

### 2. `crates/lx/src/stdlib/events.rs`

The `std/events` module. Follows the same pattern as `crates/lx/src/stdlib/channel.rs`.

**Function: `pub fn build() -> IndexMap<Sym, LxVal>`**

Use the same pattern as `crates/lx/src/stdlib/channel.rs` `build()` function (line 26). Sync builtins use `crate::builtins::mk(name, arity, fn_ptr)`. Async builtins use `crate::builtins::mk_async(name, arity, fn_ptr)`.

Register these builtins:

| Name | Builtin name | Arity | Sync/Async | Registration |
|------|-------------|-------|------------|--------------|
| `xadd` | `events.xadd` | 1 | sync | `mk("events.xadd", 1, bi_xadd)` |
| `xrange` | `events.xrange` | 2 | sync | `mk("events.xrange", 2, bi_xrange)` |
| `xread` | `events.xread` | 2 | async | `mk_async("events.xread", 2, bi_xread)` |
| `xlen` | `events.xlen` | 1 | sync | `mk("events.xlen", 1, bi_xlen)` |
| `xtrim` | `events.xtrim` | 1 | sync | `mk("events.xtrim", 1, bi_xtrim)` |

**Builtin implementations:**

- `bi_xadd(args: &[LxVal], span, ctx) -> Result<LxVal, LxError>`
  - `args[0]` must be a Record
  - Extract `kind` field as string (required, error if missing)
  - Extract `agent` field as string (default `"main"`)
  - Extract all other fields from the record into an `IndexMap<Sym, LxVal>`
  - Call `ctx.event_stream.xadd(kind, agent, None, fields)`
  - Return `LxVal::str(id)`

- `bi_xrange(args: &[LxVal], span, ctx) -> Result<LxVal, LxError>`
  - Arity 2. `args[0]` = start ID string, `args[1]` = end ID string.
  - Call `ctx.event_stream.xrange(start, end, None)`. Map results through `entry_to_lxval`. Return `LxVal::list(...)`.

- `bi_xread(args: Vec<LxVal>, span: SourceSpan, ctx: Arc<RuntimeCtx>) -> Pin<Box<dyn Future<Output = Result<LxVal, LxError>>>>`
  - Async builtin, arity 2. Follows the same pattern as `bi_recv` in `crates/lx/src/stdlib/channel.rs` (line 75): function signature takes `(Vec<LxVal>, SourceSpan, Arc<RuntimeCtx>)`, returns `Pin<Box<dyn Future<...>>>`, body wrapped in `Box::pin(async move { ... })`.
  - Registered via `crate::builtins::mk_async("events.xread", 2, bi_xread)`.
  - `args[0]` = last_id string, `args[1]` = options record (extract `timeout_ms` as optional u64 via `args[1].int_field("timeout_ms")`). If no options needed, caller passes `{}`.
  - Call `ctx.event_stream.xread(last_id, timeout_ms).await`. Return `entry_to_lxval(entry)` or `LxVal::None`.

- `bi_xlen(args: &[LxVal], span, ctx) -> Result<LxVal, LxError>`
  - Arity 1 (ignore arg; caller passes `()` unit). Return `LxVal::int(ctx.event_stream.xlen())`.

- `bi_xtrim(args: &[LxVal], span, ctx) -> Result<LxVal, LxError>`
  - `args[0]` = Record with `maxlen` field (required, Int)
  - Extract maxlen as usize
  - Call `ctx.event_stream.xtrim(maxlen)`
  - Return `LxVal::Unit`

## Files to Modify

### 3. `crates/lx/src/runtime/mod.rs`

Add a new field to `RuntimeCtx`:

```rust
pub event_stream: Arc<crate::event_stream::EventStream>,
```

Add a `SmartDefault` annotation:

```rust
#[default(Arc::new(crate::event_stream::EventStream::new(None)))]
pub event_stream: Arc<crate::event_stream::EventStream>,
```

**Where to insert:** After the `tokio_runtime` field (line 36), before `test_threshold`.

**Required import:** Add `use crate::event_stream::EventStream;` is NOT needed because the field type uses full path via `crate::event_stream::EventStream`. But the `SmartDefault` derive macro needs the type visible. Use the full path in the default annotation.

### 4. `crates/lx/src/lib.rs`

Add `pub mod event_stream;` to the module declarations.

**Where:** In the module declarations (lines 4-20), add `pub mod event_stream;` after `pub mod env;` (line 7) and before `pub mod error;` (line 8).

### 5. `crates/lx/src/stdlib/mod.rs`

**Add module declaration:** Add `mod events;` after `mod env;` (line 6).

**Add to `get_std_module`:** In the `match path[1]` block (around line 41), add:

```rust
"events" => events::build(),
```

Insert alphabetically after the `"env"` arm.

**Add to `std_module_exists`:** In the `matches!` macro (around line 91), add `"events"` to the list, after `"env"`.

### 6. `crates/lx-cli/src/main.rs`

**In `run_file` function:** Before creating `ctx_val`, determine the JSONL path. Use the file's parent directory:

```rust
let jsonl_dir = std::path::Path::new(path).parent().unwrap_or(std::path::Path::new(".")).join(".lx");
let jsonl_path = jsonl_dir.join("stream.jsonl");
```

When constructing `ctx_val`, set:

```rust
ctx_val.event_stream = Arc::new(lx::event_stream::EventStream::new(Some(jsonl_path)));
```

This goes after line 178 (`ctx_val.dep_dirs = dep_dirs;`) and before `apply_manifest_backends`.

## Step-by-Step Instructions

1. Create `crates/lx/src/event_stream.rs` with the `EventStream`, `StreamEntry`, `SpanInfo` structs, all methods listed above, and the `entry_to_lxval` helper.

2. Create `crates/lx/src/stdlib/events.rs` with the `build()` function and all five builtin implementations (`bi_xadd`, `bi_xrange`, `bi_xread`, `bi_xlen`, `bi_xtrim`).

3. Add `pub mod event_stream;` to `crates/lx/src/lib.rs`.

4. Add `event_stream` field to `RuntimeCtx` in `crates/lx/src/runtime/mod.rs`.

5. Register the `events` module in `crates/lx/src/stdlib/mod.rs` (three changes: mod declaration, `get_std_module` match arm, `std_module_exists` match list).

6. Wire JSONL path in `crates/lx-cli/src/main.rs` `run_file` function.

## Deliverable

After this work item:
- `use std/events` works in lx programs
- `events.xadd {kind: "test", data: 42}` appends an entry and returns its ID string
- `events.xrange "-" "+"` returns all entries as a list of records
- `events.xread "$" {timeout_ms: 5000}` blocks until a new entry or timeout
- `events.xlen ()` returns the count
- `events.xtrim {maxlen: 100}` trims oldest entries
- Every entry written via `xadd` is also appended as a JSON line to `.lx/stream.jsonl`
- The `EventStream` is `Arc`-shared across all interpreters via `RuntimeCtx`, thread-safe for concurrent writes from multiple agents
- `xread` is properly async (does not block the tokio runtime), uses `Notify` for efficient wake-up
