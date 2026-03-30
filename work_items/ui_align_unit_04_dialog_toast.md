# UI Alignment Unit 04: Dialog, Toast, and Close Button Fixes

## Goal

Three fixes:
- A) Toast auto-dismiss: add a `use_future` timer in `ToastViewport` that dismisses expired toasts every 500ms
- B) NewIssueDialog and NewAgentDialog close buttons: replace literal `"x"` text with Material Symbols icon
- C) Dialog backdrop: add CSS keyframe animation for fade-in

---

## Fix A: Toast Auto-Dismiss

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/toast_viewport.rs`

The `ToastState` stores `created_at` (ms timestamp) and `ttl_ms` per toast, but nothing ever checks them to auto-dismiss. Add a `use_future` that ticks every 500ms, reads the toast list, and dismisses any toast where `now - created_at >= ttl_ms`.

**old_string:**
```
#[component]
pub fn ToastViewport() -> Element {
  let state = use_context::<ToastState>();
  let toasts = state.toasts;

  if toasts.read().is_empty() {
    return rsx! {};
  }
```

**new_string:**
```
fn timestamp_ms() -> u64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_millis() as u64)
    .unwrap_or(0)
}

#[component]
pub fn ToastViewport() -> Element {
  let state = use_context::<ToastState>();
  let toasts = state.toasts;

  use_future(move || async move {
    loop {
      tokio::time::sleep(std::time::Duration::from_millis(500)).await;
      let now = timestamp_ms();
      let expired: Vec<String> = toasts
        .read()
        .iter()
        .filter(|t| now.saturating_sub(t.created_at) >= t.ttl_ms)
        .map(|t| t.id.clone())
        .collect();
      for id in expired {
        state.dismiss(&id);
      }
    }
  });

  if toasts.read().is_empty() {
    return rsx! {};
  }
```

---

## Fix B: Close Button Icon in NewIssueDialog

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/issues/new_issue.rs`

**old_string:**
```
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-lg",
            onclick: move |_| on_close.call(()),
            "x"
          }
```

**new_string:**
```
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)]",
            onclick: move |_| on_close.call(()),
            span { class: "material-symbols-outlined text-lg", "close" }
          }
```

## Fix B (continued): Close Button Icon in NewAgentDialog

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/pages/agents/new_agent.rs`

**old_string:**
```
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-lg",
            onclick: move |_| on_close.call(()),
            "x"
          }
```

**new_string:**
```
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)]",
            onclick: move |_| on_close.call(()),
            span { class: "material-symbols-outlined text-lg", "close" }
          }
```

---

## Fix C: Dialog Backdrop Fade-In Animation

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/tailwind.css`

Add a CSS keyframe and utility class after the scrollbar styles at the end of the file.

**old_string:**
```
::-webkit-scrollbar-thumb:hover {
  background: var(--surface-bright);
}
```

**new_string:**
```
::-webkit-scrollbar-thumb:hover {
  background: var(--surface-bright);
}

@keyframes fade-in {
  from { opacity: 0; }
  to { opacity: 1; }
}

.animate-fade-in {
  animation: fade-in 150ms ease-out;
}
```

Then apply the animation class to the dialog overlay.

**File:** `/home/entropybender/repos/lx/crates/lx-desktop/src/components/ui/dialog.rs`

**old_string:**
```
      class: "fixed inset-0 z-50 bg-black/50",
```

**new_string:**
```
      class: "fixed inset-0 z-50 bg-black/50 animate-fade-in",
```

---

## Verification

1. Toasts now automatically disappear after their TTL expires (default 3500ms for success, 4000ms for info, 8000ms for warn, 10000ms for error)
2. The close button in NewIssueDialog and NewAgentDialog shows a Material "close" icon instead of literal "x" text
3. Dialog backdrops fade in over 150ms when opened
