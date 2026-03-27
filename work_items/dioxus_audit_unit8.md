# Unit 8: voice_banner.rs — use_effect for state init → use_hook

## Violation

Rule: General hook correctness — `use_effect` on line 14 exists solely to set a context signal to a value available synchronously. This defers the set to after render unnecessarily.

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

`voice_widget` is available immediately after `use_ts_widget` returns (line 11). The `use_effect` defers setting `ctx.widget` to after render, which means the widget handle is `None` during the first render pass.

Cannot replace with a direct `ctx.widget.set(Some(voice_widget))` in the component body because `TsWidgetHandle` does not implement `PartialEq` (it only derives `Clone, Copy`). Without `PartialEq`, `Signal::set` always notifies subscribers (cannot deduplicate), so calling `.set()` during the component body would trigger re-renders every render — an infinite loop.

## Required Changes

Replace lines 14-16 with `use_hook`:

```rust
  use_hook(|| ctx.widget.set(Some(voice_widget)));
```

`use_hook` runs its closure only on the first render and caches the return value. On subsequent renders, it returns the cached value without re-running the closure. This sets the widget context exactly once, immediately during the first render — no deferral, no re-render loop.

## Files Modified

- `crates/lx-desktop/src/pages/agents/voice_banner.rs` — replace lines 14-16 with one line after line 13.

## Verification

Run `just diagnose` and confirm no errors in `voice_banner.rs`.
