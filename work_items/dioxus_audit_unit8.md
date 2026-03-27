# Unit 8: voice_banner.rs — remove unnecessary use_effect for state init

## Violation

Rule: General hook correctness — `use_effect` on line 14 exists solely to set a context signal to a value that is already available synchronously. This is an unnecessary indirection.

File: `crates/lx-desktop/src/pages/agents/voice_banner.rs`, lines 14-16.

## Current Code

```rust
  let (voice_element_id, voice_widget) = use_ts_widget("voice", serde_json::json!({}));
  let (agent_element_id, agent_widget) = use_ts_widget("agent", serde_json::json!({}));
  let mut ctx = use_context::<VoiceContext>();
  use_effect(move || {
    ctx.widget.set(Some(voice_widget));
  });
```

## Problem

`voice_widget` is available immediately after `use_ts_widget` returns (line 11). The `use_effect` on line 14 defers setting `ctx.widget` to after render, which means the widget handle is `None` during the first render. Setting it directly would make it available immediately.

## Required Changes

Replace lines 14-16 with a direct set:

```rust
  ctx.widget.set(Some(voice_widget));
```

This runs during the component body (before render), making the widget handle available immediately in the first render pass.

## Files Modified

- `crates/lx-desktop/src/pages/agents/voice_banner.rs` — remove lines 14-16, add one line after line 13.

## Verification

Run `just diagnose` and confirm no errors in `voice_banner.rs`.
