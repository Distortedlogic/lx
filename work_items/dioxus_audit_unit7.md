# Unit 7: status_bar.rs — use_effect wrapping spawn → use_future

## Violation

Rule: "use_action for event handlers" / general hook correctness — `use_effect` on line 8 wraps a `spawn(async move { ... })` to run a git command on mount. `use_effect` is synchronous and should not contain `spawn`. The correct hook for an async operation that runs on mount is `use_future`.

File: `crates/lx-desktop/src/layout/status_bar.rs`, lines 8-18.

## Current Code

```rust
#[component]
pub fn StatusBar() -> Element {
  let state = use_context::<StatusBarState>();
  use_effect(move || {
    spawn(async move {
      if let Ok(output) = tokio::process::Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output().await
        && output.status.success()
      {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let mut branch_sig = state.branch;
        branch_sig.set(branch);
      }
    });
  });
```

## Problem

`use_effect` runs synchronously every time its dependencies change (every render in this case, since it captures `state` which is a context). Inside it, `spawn` launches an async task — so every re-render of `StatusBar` spawns a new git process. This is wasteful and could cause race conditions.

## Required Changes

Replace lines 8-18 with `use_future`:

```rust
  use_future(move || async move {
    if let Ok(output) = tokio::process::Command::new("git").args(["rev-parse", "--abbrev-ref", "HEAD"]).output().await
      && output.status.success()
    {
      let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
      let mut branch_sig = state.branch;
      branch_sig.set(branch);
    }
  });
```

`use_future` runs the async block once on mount and re-runs when its dependencies change. Unlike `use_effect` + `spawn`, it does not spawn a new task on every render — it manages the task lifecycle correctly.

## Files Modified

- `crates/lx-desktop/src/layout/status_bar.rs` — lines 8-18 only. Rest of file unchanged.

## Verification

Run `just diagnose` and confirm no errors in `status_bar.rs`.
