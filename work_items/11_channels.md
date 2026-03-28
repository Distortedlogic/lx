# Work Item 7: Channels

Implement the channel system as a discovery and topology layer for agents. Channels are named groups that agents subscribe to. Channels do NOT carry messages — they are a registry. You query a channel to discover which agents are subscribed, then message agents directly via tell/ask.

This is separate from the existing `std/channel` module (`crates/lx/src/stdlib/channel.rs`), which provides message-passing channels with send/recv. The existing `std/channel` remains unchanged.

## Prerequisites

- Work item 5 (agent system refactor) — named agent registry exists.
- Work item 6 (agent messaging) — tell/ask work with name strings.

## Key Rules (from CLAUDE.md)

- No code comments or doc strings
- Never use `#[allow(...)]` macros
- 300 line file limit per file
- Use `just diagnose` (not raw cargo commands) to verify
- Prefer established crates over custom code

## Current State

- `crates/lx/src/stdlib/channel.rs` — existing message-passing channel module with `create`, `send`, `recv`, `try_recv`, `close`. Uses `DashMap<u64, ChannelEntry>` with numeric IDs and `tokio::sync::mpsc`. This module is UNRELATED and must NOT be modified.
- `crates/lx/src/stdlib/mod.rs` line 42 — `"channel" => channel::build()` registers `std/channel`.
- `crates/lx/src/lexer/token.rs` — no `Channel` token kind exists.
- `crates/lx/src/ast/mod.rs` — no channel-related AST nodes exist.
- `crates/lx/src/runtime/agent_registry.rs` (from work item 5) — `AgentHandle`, `register_agent`, `get_agent_mailbox`, `remove_agent`, `agent_exists`, `agent_names`.

## Design

`channel name` is a top-level declaration statement that creates a named channel. In lx source it looks like:

```lx
channel research
channel writing
```

The channel value is an `LxVal::Channel { name: Sym }` variant. Field access on a Channel value dispatches through `channel_dispatch` (same pattern as `LxVal::Store` at `crates/lx/src/interpreter/apply_helpers.rs` line 32-34). `members` is eagerly evaluated on field access (returns a `List<Str>` snapshot). `subscribe` returns a partially-applied `BuiltinFunc`.

Channels live in a process-global registry (like the agent registry), keyed by name string. Each channel stores a `Vec<String>` of subscribed agent names.

Agent classes can declare `subscribes = [channel1, channel2]` as a field. The runtime reads this field on `spawn` and auto-registers the agent on those channels before calling `run` or accepting messages.

## Files to Create

- `crates/lx/src/runtime/channel_registry.rs` — global channel registry + dispatch

## Files to Modify

- `crates/lx/src/runtime/mod.rs` — add `mod channel_registry; pub use channel_registry::*;`
- `crates/lx/src/lexer/token.rs` — add `ChannelKw` token kind (after `HttpKw,` line 98)
- `crates/lx/src/lexer/helpers.rs` — map `"channel"` to `TokenKind::ChannelKw` (in `ident_or_keyword`, line 16, after `"as"` arm at line 30)
- `crates/lx/src/ast/mod.rs` — add `ChannelDecl(Sym)` variant to `Stmt` enum (line 34)
- `crates/lx/src/parser/stmt.rs` — parse `channel <ident>` as a statement (in `stmt_parser`, line 30)
- `crates/lx/src/interpreter/exec_stmt.rs` — eval the `ChannelDecl` statement (in `eval_stmt`, line 27)
- `crates/lx/src/interpreter/apply_helpers.rs` — add `LxVal::Channel` arm in `eval_field_access` (line 16, after `LxVal::Store` arm at line 32-34)
- `crates/lx/src/value/mod.rs` — add `Channel { name: Sym }` variant to `LxVal` enum (after `Stream { id: u64 }` at line 113-115)
- `crates/lx/src/value/display.rs` — add display arm (after `LxVal::Stream` at line 81)
- `crates/lx/src/value/impls.rs` — add `Channel` to `structural_eq` (before `_ => false` at line 74) and `hash_value` (before `Func|MultiFunc` arm at line 128)
- `crates/lx/src/formatter/emit_stmt.rs` — add `Stmt::ChannelDecl` arm (after `Stmt::Use` at line 19)
- `crates/lx/src/builtins/agent/spawn.rs` (from work item 5) — after spawning, read `subscribes` field and auto-register on channels

## Step 1: Create channel registry

File: `crates/lx/src/runtime/channel_registry.rs`

```rust
use std::sync::{Arc, LazyLock};

use dashmap::DashMap;
use miette::SourceSpan;

use crate::error::LxError;
use crate::runtime::RuntimeCtx;
use crate::value::{BuiltinFunc, BuiltinKind, LxVal};

static CHANNEL_REGISTRY: LazyLock<DashMap<String, Vec<String>>> = LazyLock::new(DashMap::new);

pub fn create_channel(name: &str) {
    CHANNEL_REGISTRY.entry(name.to_string()).or_insert_with(Vec::new);
}

pub fn channel_exists(name: &str) -> bool {
    CHANNEL_REGISTRY.contains_key(name)
}

pub fn channel_subscribe(channel_name: &str, agent_name: &str) -> Result<(), String> {
    let mut entry = CHANNEL_REGISTRY.get_mut(channel_name)
        .ok_or_else(|| format!("channel '{}' does not exist", channel_name))?;
    if !entry.contains(&agent_name.to_string()) {
        entry.push(agent_name.to_string());
    }
    Ok(())
}

pub fn channel_unsubscribe(channel_name: &str, agent_name: &str) {
    if let Some(mut entry) = CHANNEL_REGISTRY.get_mut(channel_name) {
        entry.retain(|n| n != agent_name);
    }
}

pub fn channel_unsubscribe_all(agent_name: &str) {
    for mut entry in CHANNEL_REGISTRY.iter_mut() {
        entry.value_mut().retain(|n| n != agent_name);
    }
}

pub fn channel_members(channel_name: &str) -> Option<Vec<String>> {
    CHANNEL_REGISTRY.get(channel_name).map(|e| e.clone())
}

pub fn channel_dispatch(channel_name: &str, method: &str, span: SourceSpan) -> Result<LxVal, LxError> {
    match method {
        "members" => {
            match channel_members(channel_name) {
                Some(names) => Ok(LxVal::list(names.into_iter().map(LxVal::str).collect())),
                None => Err(LxError::runtime(
                    format!("channel '{}' does not exist", channel_name), span
                )),
            }
        },
        "subscribe" => {
            Ok(LxVal::BuiltinFunc(BuiltinFunc {
                name: "channel.subscribe",
                arity: 2,
                kind: BuiltinKind::Sync(bi_channel_subscribe_impl),
                applied: vec![LxVal::str(channel_name)],
            }))
        },
        "name" => Ok(LxVal::str(channel_name)),
        _ => Err(LxError::type_err(
            format!("Channel has no method '{}'", method), span, None
        )),
    }
}

fn bi_channel_subscribe_impl(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
    let channel_name = args[0].require_str("channel.subscribe", span)?;
    let agent_name = args[1].require_str("channel.subscribe", span)?;
    channel_subscribe(channel_name, agent_name)
        .map_err(|e| LxError::runtime(e, span))?;
    Ok(LxVal::ok_unit())
}
```

The `channel_unsubscribe_all` function is called when an agent stops (from `Expr::Stop` evaluation in work item 5, or from `agent.kill`). Update the stop/kill code paths to call `channel_unsubscribe_all(&agent_name)` before removing the agent from the agent registry.

`channel_dispatch` handles field access on `LxVal::Channel` values:
- `members`: eagerly evaluates and returns a `List<Str>` snapshot of subscribed agent names. `research.members` returns the list directly.
- `subscribe`: returns a `BuiltinFunc` with arity 2 and 1 pre-applied arg (channel name). `research.subscribe "AgentName"` applies the agent name as the second arg, triggering execution.
- `name`: returns the channel name as a string.

## Step 2: Wire into RuntimeCtx

File: `crates/lx/src/runtime/mod.rs`

Add after the `agent_registry` module declaration:

```rust
pub mod channel_registry;
pub use channel_registry::*;
```

## Step 3: Add ChannelKw token

File: `crates/lx/src/lexer/token.rs`

Add `ChannelKw,` after `HttpKw,` (line 98), before `Export,` (line 100).

File: `crates/lx/src/lexer/helpers.rs`

In `ident_or_keyword` (line 16), add `"channel" => TokenKind::ChannelKw,` after `"as" => TokenKind::As,` (line 30), before the `_ =>` default (line 31).

## Step 4: Add ChannelDecl to Stmt enum

File: `crates/lx/src/ast/mod.rs`

Add a variant to the `Stmt` enum (line 34), after `Expr(ExprId)` (line 47):

```rust
ChannelDecl(Sym),
```

No `#[walk(skip)]` needed -- `Sym` has no child AST nodes to walk.

## Step 5: Parse channel declaration

File: `crates/lx/src/parser/stmt.rs`

In `stmt_parser` (line 30), add `let a_chan = arena.clone();` alongside the other arena clones at lines 37-42.

Define the channel_decl parser:

```rust
let channel_decl = just(TokenKind::ChannelKw)
    .ignore_then(select! { TokenKind::Ident(name) => name })
    .map_with(move |name, e| a_chan.borrow_mut().alloc_stmt(Stmt::ChannelDecl(name), ss(e.span())));
```

Add `channel_decl` to the `choice((...))` block (line 56-83). Insert it before `expr_stmt` (the last item in the choice), since `channel` is now a keyword token and won't conflict with identifier parsing.

## Step 6: Add LxVal::Channel variant

File: `crates/lx/src/value/mod.rs`

Add to the `LxVal` enum after `Stream { id: u64 }` (line 113-115):

```rust
Channel {
    name: Sym,
},
```

The `IntoStaticStr` derive from `strum` auto-generates `type_name()` returning `"Channel"` for this variant.

File: `crates/lx/src/value/display.rs`

Add after `LxVal::Stream { id } => write!(f, "<Stream#{id}>"),` (line 81):

```rust
LxVal::Channel { name } => write!(f, "Channel({})", name),
```

File: `crates/lx/src/value/serde_impl.rs`

No change needed -- caught by existing `_ =>` catch-all arm at line 62.

File: `crates/lx/src/value/impls.rs`

In `structural_eq` (line 36), add before the `_ => false` catch-all (line 74):

```rust
(LxVal::Channel { name: n1 }, LxVal::Channel { name: n2 }) => n1 == n2,
```

In `hash_value` (line 78), add before the `Func|MultiFunc|BuiltinFunc|TaggedCtor` arm (line 128):

```rust
LxVal::Channel { name } => name.hash(state),
```

## Step 7: Update eval_field_access

File: `crates/lx/src/interpreter/apply_helpers.rs`

Add a match arm for `LxVal::Channel` in the `FieldKind::Named(name)` match (line 16), after the `LxVal::Store { .. }` arm (line 32-34), before the `other =>` fallback (line 35):

```rust
LxVal::Channel { name: channel_name } => {
    Ok(crate::runtime::channel_registry::channel_dispatch(
        channel_name.as_str(), name.as_str(), span
    )?)
},
```

## Step 8: Evaluate ChannelDecl statement

File: `crates/lx/src/interpreter/exec_stmt.rs`

Add a match arm in `eval_stmt` (line 27), after `Stmt::KeywordDecl(_) => unreachable!("keyword not desugared"),` (line 116), before `Stmt::ClassDecl(data)` (line 117):

```rust
Stmt::ChannelDecl(name) => {
    let channel_name = name.as_str().to_string();
    crate::runtime::channel_registry::create_channel(&channel_name);
    let channel_val = LxVal::Channel { name: *name };
    let env = self.env.child();
    env.bind(*name, channel_val);
    self.env = Arc::new(env);
    Ok(LxVal::Unit)
},
```

## Step 9: Add formatter handling

File: `crates/lx/src/formatter/emit_stmt.rs`

Add a match arm in `emit_stmt` (line 9), after `Stmt::Use(u) => self.emit_use(u),` (line 19), before `Stmt::Expr(eid)` (line 20):

```rust
Stmt::ChannelDecl(name) => {
    self.write("channel ");
    self.write(name.as_str());
},
```

No desugarer changes needed -- `ChannelDecl` is a new `Stmt` variant that passes through the desugarer unchanged (handled by the `_ =>` arm in `Desugarer::transform_stmts` at `crates/lx/src/folder/desugar.rs` line 46-48).

## Step 10: Auto-subscribe on spawn

File: `crates/lx/src/builtins/agent/spawn.rs` (from work item 5)

After registering the agent in the agent registry and before starting the agent's run/handle loop, check for a `subscribes` field on the class:

```rust
if let Some(subscribes_val) = class.defaults.get(&intern("subscribes")) {
    if let LxVal::List(channels) = subscribes_val {
        for ch in channels.iter() {
            if let LxVal::Channel { name: ch_name } = ch {
                crate::runtime::channel_registry::channel_subscribe(
                    ch_name.as_str(), &agent_name
                ).unwrap_or_else(|e| eprintln!("auto-subscribe failed: {}", e));
            } else if let Some(ch_name) = ch.as_str() {
                crate::runtime::channel_registry::channel_subscribe(
                    ch_name, &agent_name
                ).unwrap_or_else(|e| eprintln!("auto-subscribe failed: {}", e));
            }
        }
    }
}
```

The `subscribes` field contains a list. Each element is either a `LxVal::Channel` value (when the channel was declared with `channel name` and referenced directly) or a string (channel name). Handle both.

## Step 11: Unsubscribe on agent stop/kill

In the agent stop code (work item 5, `Expr::Stop` eval and `agent.kill` builtin), before removing the agent from the registry, call:

```rust
crate::runtime::channel_registry::channel_unsubscribe_all(&agent_name);
```

This removes the agent from all channels.

## Verification

After all changes:
1. `just diagnose` must pass with no warnings.
2. `just test` must pass all existing tests.
3. Write a test `.lx` file:

```lx
channel workers

Agent Fast = {
  subscribes = [workers]
  handle = (msg) { {from: "Fast", result: msg.task} }
}

Agent Slow = {
  subscribes = [workers]
  handle = (msg) { {from: "Slow", result: msg.task} }
}

spawn Fast
spawn Slow

peers = workers.members
assert (len peers) == 2

results = peers | map (p) { ask p {task: "compute"} }
assert (len results) == 2
```

This test verifies: channel declaration, auto-subscribe on spawn, `members` returns correct list, agents are addressable by the names from `members`.
