# Goal

Add channel-based communication (CSP style) as an alternative to the existing ask/reply agent messaging. Channels enable streaming partial results between concurrent tasks, fan-out/fan-in patterns, and producer/consumer decoupling — patterns that request/reply (`~>?`) can't express cleanly.

# Why

- Current agent communication is strictly request/reply. Streaming LLM output token-by-token, sending incremental progress, or building processing pipelines all require workarounds.
- The research in `research/concurrency/design-patterns.md` covers Go's CSP model. Channels complement ask/reply — they're not a replacement but an addition for different concurrency shapes.
- `par` runs tasks concurrently but they can't communicate mid-flight. Channels let parallel branches exchange data during execution.

# What Changes

## New stdlib module: `std/channel`

New file `crates/lx/src/stdlib/channel.rs` implementing 5 functions:

**`channel.create capacity -> (Sender, Receiver)`** — Creates a bounded channel (using `tokio::sync::mpsc`). `capacity` is an Int (0 = unbounded). Returns a tuple of sender and receiver handles.

**`channel.send sender value -> Result`** — Sends a value on the channel. Blocks (async await) if the channel is full. Returns `Err` if the receiver has been dropped.

**`channel.recv receiver -> Result`** — Receives the next value from the channel. Blocks until a value is available. Returns `Err {kind: :closed}` if the sender has been dropped and the channel is empty.

**`channel.try_recv receiver -> Maybe`** — Non-blocking receive. Returns `Some value` if available, `None` if empty (doesn't wait).

**`channel.close sender -> ()`** — Explicitly closes the sender side. Receivers will get `Err :closed` after draining remaining items.

## Usage pattern

```
use std/channel

(tx, rx) = channel.create 10

par {
  [1 2 3 4 5] | each (item) {
    channel.send tx (item * 2) ^
  }
  channel.close tx

  results := []
  loop {
    channel.recv rx ? {
      Ok val -> { results <- [..results val] }
      Err _ -> break results
    }
  }
}
```

## Runtime representation

Sender and Receiver are stored in a global `LazyLock<DashMap<u64, ChannelEntry>>` registry (same pattern as `ws.rs` uses for WebSocket connections), keyed by a unique `u64` channel ID generated via `AtomicU64`. `channel.create` returns a tuple of two records containing the channel ID and role:
- `{__chan_id: 1, _role: "sender"}`
- `{__chan_id: 1, _role: "receiver"}`

`send`/`recv` extract `__chan_id` from the record and look up the underlying `tokio::sync::mpsc` channel by ID from the registry. For async operations, use `tokio::task::block_in_place` + `block_on` (same approach as `ws.rs`). This avoids adding new variants to `LxVal`.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/channel.rs` — channel create/send/recv/try_recv/close
- `tests/82_channels.lx` — tests for channel communication

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — register `mod channel;`, add `"channel" => channel::build()` to `get_std_module`, add `"channel"` to the `matches!` in `std_module_exists`

# Task List

### Task 1: Implement channel.create and internal registry

**Subject:** Create channel pairs with mpsc and store in global registry

**Description:** Create `crates/lx/src/stdlib/channel.rs`. Use the same registry pattern as `ws.rs`: a global `static CHANNELS: LazyLock<DashMap<u64, ChannelEntry>>` where `ChannelEntry` holds `Arc<tokio::sync::mpsc::Sender<LxVal>>` and `Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<LxVal>>>`, plus a `static NEXT_ID: AtomicU64` for ID generation. DashMap is already a dependency in `crates/lx/Cargo.toml`. tokio's `sync` feature is already enabled.

Implement `bi_create(args, span, ctx)` as a sync builtin (`SyncBuiltinFn` signature: `fn(&[LxVal], SourceSpan, &Arc<RuntimeCtx>) -> Result<LxVal, LxError>`):
1. Parse capacity from `args[0]` as usize (0 maps to a large default like 1_000_000 for "unbounded").
2. Create `tokio::sync::mpsc::channel(capacity)`.
3. Generate a unique ID via `NEXT_ID.fetch_add(1, Ordering::Relaxed)`.
4. Store sender and receiver in the registry.
5. Return a tuple of two records: `({__chan_id: id, _role: "sender"}, {__chan_id: id, _role: "receiver"})`.

Register the module in `stdlib/mod.rs`:
- Add `mod channel;` declaration
- Add `"channel" => channel::build()` arm in `get_std_module`
- Add `"channel"` to the `matches!` in `std_module_exists`

In `channel.rs`, define `pub fn build() -> IndexMap<String, LxVal>` using `crate::builtins::mk` to register sync builtins (same pattern as store and ws modules). Add `"create"` to the map.

Run `just diagnose`.

**ActiveForm:** Implementing channel creation and registry

---

### Task 2: Implement channel.send and channel.recv

**Subject:** Blocking send and receive on channels

**Description:** In `crates/lx/src/stdlib/channel.rs`:

`bi_send(args, span, ctx)` (sync builtin using `block_in_place`/`block_on` for the async send):
1. Extract `__chan_id` from `args[0]` (the sender record).
2. Look up the channel's sender `Arc` in the `CHANNELS` registry.
3. Use `tokio::task::block_in_place(|| Handle::current().block_on(async { sender.send(value).await }))`. If the receiver is dropped, return `Err "channel closed"`.
4. Return `Ok ()`.

`bi_recv(args, span, ctx)` (sync builtin using `block_in_place`/`block_on` for the async recv):
1. Extract `__chan_id` from `args[0]` (the receiver record).
2. Look up the channel's receiver `Arc<TokioMutex<Receiver>>` in the registry.
3. Use `block_in_place`/`block_on` to lock the receiver mutex and call `receiver.recv().await`.
4. If `Some(value)`, return `Ok value`. If `None` (sender dropped), return `Err {kind: :closed}`.

Add `"send"` and `"recv"` to the `build()` map.

Run `just diagnose`.

**ActiveForm:** Implementing send and recv

---

### Task 3: Implement try_recv, close, and write tests

**Subject:** Non-blocking receive, explicit close, and channel test suite

**Description:** Implement `bi_try_recv(args, span, ctx)` (sync builtin) — extract `__chan_id` from `args[0]`, look up receiver in registry, use `try_recv()` (non-async, no `block_in_place` needed). Returns `Some value` or `None`.

Implement `bi_close(args, span, ctx)` (sync builtin) — extract `__chan_id` from `args[0]`, remove the sender from the `CHANNELS` registry, causing receivers to eventually get `Err :closed`.

Add `"try_recv"` and `"close"` to the `build()` map.

Create `tests/82_channels.lx` (create the `tests/` directory if it does not exist) with tests:
1. **Basic send/recv** — create channel, send 3 values, recv 3 values, verify order.
2. **Close behavior** — send 2 values, close sender, recv 2 values, recv again returns Err.
3. **Par producer/consumer** — `par` block where one branch sends, another receives.
4. **try_recv** — verify returns None on empty channel, Some after send.
5. **Bounded channel** — create with capacity 1, verify send blocks when full (test via `sel` with timeout).

Run `just diagnose` and `just test`.

**ActiveForm:** Implementing try_recv, close, and writing tests

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
mcp__workflow__load_work_item({ path: "work_items/CHANNELS_CSP.md" })
```

Then call `next_task` to begin.
