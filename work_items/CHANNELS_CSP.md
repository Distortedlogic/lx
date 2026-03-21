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

Sender and Receiver are stored in a global `DashMap` registry (same pattern as `Store`), keyed by a unique channel ID. `channel.create` returns a tuple of two records containing the channel ID and role:
- `{_channel_id: "ch_001", _role: "sender"}`
- `{_channel_id: "ch_001", _role: "receiver"}`

`send`/`recv` look up the underlying `tokio::sync::mpsc` channel by ID from the registry. This avoids adding new variants to `LxVal`.

# Files Affected

**New files:**
- `crates/lx/src/stdlib/channel.rs` — channel create/send/recv/try_recv/close
- `tests/82_channels.lx` — tests for channel communication

**Modified files:**
- `crates/lx/src/stdlib/mod.rs` — register `mod channel;`, add to `get_std_module`

# Task List

### Task 1: Implement channel.create and internal registry

**Subject:** Create channel pairs with mpsc and store in global registry

**Description:** Create `crates/lx/src/stdlib/channel.rs`. Use a global `DashMap<String, ChannelEntry>` registry where `ChannelEntry` holds `Arc<tokio::sync::mpsc::Sender<LxVal>>` and `Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<LxVal>>>`.

Implement `bi_create(capacity)`:
1. Parse capacity as usize (0 maps to a large default like 1_000_000 for "unbounded").
2. Create `tokio::sync::mpsc::channel(capacity)`.
3. Generate a unique ID (uuid or counter).
4. Store sender and receiver in the registry.
5. Return a tuple of two records: `({_channel_id: id, _role: "sender"}, {_channel_id: id, _role: "receiver"})`.

Register the module in `stdlib/mod.rs`. Add `"create"` to the `build()` map.

Run `just diagnose`.

**ActiveForm:** Implementing channel creation and registry

---

### Task 2: Implement channel.send and channel.recv

**Subject:** Blocking send and receive on channels

**Description:** In `crates/lx/src/stdlib/channel.rs`:

`bi_send(sender_record, value)`:
1. Extract `_channel_id` from the sender record.
2. Look up the channel in the registry.
3. Call `sender.send(value).await`. If the receiver is dropped, return `Err "channel closed"`.
4. Return `Ok ()`.

`bi_recv(receiver_record)`:
1. Extract `_channel_id` from the receiver record.
2. Look up the channel in the registry.
3. Lock the receiver mutex, call `receiver.recv().await`.
4. If `Some(value)`, return `Ok value`. If `None` (sender dropped), return `Err {kind: :closed}`.

Add `"send"` and `"recv"` to the `build()` map.

Run `just diagnose`.

**ActiveForm:** Implementing send and recv

---

### Task 3: Implement try_recv, close, and write tests

**Subject:** Non-blocking receive, explicit close, and channel test suite

**Description:** Implement `bi_try_recv(receiver)` — same as recv but uses `try_recv()` (non-async). Returns `Some value` or `None`.

Implement `bi_close(sender)` — drops the sender from the registry, causing receivers to eventually get `Err :closed`.

Add `"try_recv"` and `"close"` to the `build()` map.

Create `tests/82_channels.lx` with tests:
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
