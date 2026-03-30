# Unit 12: Toast Dedup + Animations

## Goal

Add deduplication (3.5s window), TTL clamping (1.5s-15s), and CSS entry/exit animations to the toast system.

## Preconditions

- No other units need to be complete first
- The toast system works end-to-end: `ToastState::push()` creates items, `ToastViewport` renders and auto-dismisses them

## Files to Modify

- `crates/lx-desktop/src/contexts/toast.rs`
- `crates/lx-desktop/src/components/toast_viewport.rs`
- `crates/lx-desktop/src/tailwind.css`

## Steps

### Step 1: Add TTL clamping in `toast.rs`

In the `push` method (line 54), after computing `ttl_ms` from the input, clamp it:

```rust
let ttl_ms = input.ttl_ms.unwrap_or_else(|| default_ttl(tone)).clamp(1500, 15000);
```

Replace line 56 (`let ttl_ms = input.ttl_ms.unwrap_or_else(|| default_ttl(tone));`) with the clamped version above.

### Step 2: Add dedup tracking to `ToastState` in `toast.rs`

Add a new field to `ToastState`:

```rust
#[derive(Clone, Copy)]
pub struct ToastState {
  pub toasts: Signal<Vec<ToastItem>>,
  dedup_log: Signal<Vec<(String, ToastTone, u64)>>,
}
```

Update `provide()` to initialize the new field:

```rust
pub fn provide() -> Self {
  let state = Self {
    toasts: Signal::new(Vec::new()),
    dedup_log: Signal::new(Vec::new()),
  };
  use_context_provider(|| state);
  state
}
```

### Step 3: Add dedup check in `push` method of `toast.rs`

At the start of `push`, before any other logic, add the dedup check. The full replacement for the `push` method body:

```rust
pub fn push(&self, input: ToastInput) -> Option<String> {
  let tone = input.tone;
  let now = timestamp_ms();
  let dedup_window_ms: u64 = 3500;

  {
    let mut log = self.dedup_log;
    let mut entries = log.write();
    entries.retain(|(_, _, ts)| now.saturating_sub(*ts) < dedup_window_ms);

    if entries.iter().any(|(t, tn, _)| t == &input.title && *tn == tone) {
      return None;
    }
    entries.push((input.title.clone(), tone, now));
  }

  let ttl_ms = input.ttl_ms.unwrap_or_else(|| default_ttl(tone)).clamp(1500, 15000);
  let id = format!("toast_{}_{}", now, random_suffix());
  let item = ToastItem {
    id: id.clone(),
    title: input.title,
    body: input.body,
    tone,
    ttl_ms,
    action: input.action,
    created_at: now,
  };
  let mut toasts = self.toasts;
  let mut list = toasts.write();
  list.insert(0, item);
  list.truncate(MAX_TOASTS);
  Some(id)
}
```

The return type changes from `String` to `Option<String>`. No callers in the codebase use the return value of `push` (verified). The change is transparent.

### Step 4: Add `dismissing` state to `ToastItem` in `toast.rs`

Add a `dismissing` field to `ToastItem` to track exit animation state:

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ToastItem {
  pub id: String,
  pub title: String,
  pub body: Option<String>,
  pub tone: ToastTone,
  pub ttl_ms: u64,
  pub action: Option<ToastAction>,
  pub created_at: u64,
  pub dismissing: bool,
}
```

Set `dismissing: false` in the `push` method when creating the item.

### Step 5: Add `start_dismiss` method to `ToastState` in `toast.rs`

Add a method that marks a toast as dismissing (triggers exit animation) rather than immediately removing it:

```rust
pub fn start_dismiss(&self, id: &str) {
  let mut toasts = self.toasts;
  let mut list = toasts.write();
  if let Some(item) = list.iter_mut().find(|t| t.id == id) {
    item.dismissing = true;
  }
}
```

### Step 6: Update the expiry loop in `toast_viewport.rs`

The `use_future` loop (lines 67-76) currently calls `state.dismiss()` directly. Change it to use a two-phase approach: first mark as dismissing, then remove after the exit animation duration (300ms).

Replace the entire `use_future` block:

```rust
use_future(move || async move {
  loop {
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    let now = timestamp_ms();

    let to_start_dismiss: Vec<String> = toasts
      .read()
      .iter()
      .filter(|t| !t.dismissing && now.saturating_sub(t.created_at) >= t.ttl_ms)
      .map(|t| t.id.clone())
      .collect();

    for id in &to_start_dismiss {
      state.start_dismiss(id);
    }

    let to_remove: Vec<String> = toasts
      .read()
      .iter()
      .filter(|t| t.dismissing && now.saturating_sub(t.created_at) >= t.ttl_ms + 300)
      .map(|t| t.id.clone())
      .collect();

    for id in to_remove {
      state.dismiss(&id);
    }
  }
});
```

### Step 7: Add CSS @keyframes to `tailwind.css`

Append these keyframes and utility classes after the existing `.animate-activity-enter` block (after line 123):

```css
@keyframes toast-slide-in {
  from {
    opacity: 0;
    transform: translateX(-100%);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

.animate-toast-enter {
  animation: toast-slide-in 300ms ease-out both;
}

@keyframes toast-slide-out {
  from {
    opacity: 1;
    transform: translateX(0);
  }
  to {
    opacity: 0;
    transform: translateX(-100%);
  }
}

.animate-toast-exit {
  animation: toast-slide-out 300ms ease-in both;
}
```

### Step 8: Apply animation classes in `toast_viewport.rs`

In the `render_toast` function, change the `li` element's class to include the animation class based on the `dismissing` state.

Replace the `li` opening (line 32):

```rust
li {
  class: format!(
    "pointer-events-auto rounded-sm border shadow-lg backdrop-blur-xl {} {}",
    tc,
    if toast.dismissing { "animate-toast-exit" } else { "animate-toast-enter" }
  ),
```

Since `render_toast` takes `&ToastItem`, the `dismissing` field is already accessible.

### Step 9: Update manual dismiss button in `toast_viewport.rs`

The dismiss button's `onclick` (line 50) currently calls `state.dismiss(&id)` directly. Change it to trigger the exit animation first:

```rust
onclick: move |_| state.start_dismiss(&id),
```

## Verification

1. Run `just diagnose` -- must compile with zero warnings
2. Visual checks in the running app:
   - Push two toasts with the same title and tone within 3.5s -- only the first should appear
   - Push a toast with the same title after 3.5s -- it should appear again
   - Push a toast with `ttl_ms: Some(500)` -- it should stay for 1.5s (clamped minimum)
   - Push a toast with `ttl_ms: Some(30000)` -- it should stay for 15s (clamped maximum)
   - New toasts slide in from the left with 300ms animation
   - Expiring toasts slide out to the left with 300ms animation
   - Clicking the X triggers the exit animation before removal
