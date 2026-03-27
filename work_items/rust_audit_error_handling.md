# Error Handling — Additional Swallowed Errors

Fixes for swallowed errors not covered by the existing error_handling_fixes.md.

---

## Task 1: Accept broadcast send as fire-and-forget in activity_api

**File:** `crates/lx-api/src/activity_api.rs:27`

Current: `let _ = EVENT_TX.send(event.clone());`

The broadcast send error (no subscribers) is silently dropped. `tracing` is NOT available in the `lx-api` crate (its only deps are dioxus, serde, serde_json, tokio). Having no subscribers is the normal startup condition for broadcast channels.

Fix: This is intentional fire-and-forget. No code change needed — remove from audit scope.

---

## Task 2: Log audio playback error in voice_banner

**File:** `crates/lx-desktop/src/pages/agents/voice_banner.rs:233`

Current: `let _ = voice_pipeline::play_wav(...)`

The `lx-desktop` crate has `dioxus::logger::tracing` available (used elsewhere in the crate).

Fix: Change to:
```rust
if let Err(e) = voice_pipeline::play_wav(...) {
    tracing::warn!("audio playback failed: {e}");
}
```

Add `use dioxus::logger::tracing;` if not already imported in this file.

---

## Task 3: Fix silent null/zero in serde conversions

**File:** `crates/lx/src/value/serde_impl.rs`

**Line 83:** `n.as_f64().unwrap_or(0.0)` — inside `impl From<serde_json::Value> for LxVal`, in the `Number` arm. The `as_i64()` check already failed at this point, so the number must be u64 or f64. `as_f64()` returns `None` only for u64 values > 2^53. Fix: try u64 fallback:
```rust
n.as_f64()
    .or_else(|| n.as_u64().map(|u| u as f64))
    .unwrap_or(0.0)
```

**Line 101:** `serde_json::to_value(v).unwrap_or(serde_json::Value::Null)` — inside `impl From<&LxVal> for serde_json::Value`. The `Serialize` impl (lines 19-64) handles every `LxVal` variant — every arm serializes a primitive, delegates to a serializable type, or falls back to a string via `format!("<{}>", self.type_name())`. Serialization cannot fail for any `LxVal` variant. Fix: Change to:
```rust
serde_json::to_value(v).expect("LxVal is always serializable")
```
