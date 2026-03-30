# WU-02: Debounced draft persistence

## Fixes
- Fix 10: Draft persistence writes to storage on every keystroke. Replace with 800ms debounce to reduce write frequency.

## Files Modified
- `crates/lx-desktop/src/components/comment_thread.rs` (82 lines)
- `crates/lx-desktop/src/pages/issues/new_issue.rs` (167 lines)

## Preconditions
- `comment_thread.rs` line 17: `let mut body = dioxus_storage::use_persistent("lx_comment_draft", String::new);` — the draft signal writes immediately on every `body.set(v)`.
- `new_issue.rs` lines 52-67: A `use_effect` watches all fields and calls `localStorage.setItem` on every reactive change — this fires on every keystroke.
- `tokio::time::sleep` is the established async sleep pattern in this codebase (e.g., `contexts/live_updates.rs` line 59, `components/copy_text.rs` line 15).
- The crate depends on `tokio` (Cargo.toml line 48).

## Steps

### Step 1: Add debounced persistence to comment_thread.rs
- Open `crates/lx-desktop/src/components/comment_thread.rs`
- The `use_persistent` hook at line 17 already writes to persistent storage on every `.set()` call. To debounce, we need to separate the in-memory signal from the persistent write.
- Replace line 17:
```rust
  let mut body = dioxus_storage::use_persistent("lx_comment_draft", String::new);
```
- With:
```rust
  let mut body = use_signal(String::new);
  let mut persisted = dioxus_storage::use_persistent("lx_comment_draft", String::new);
  let mut debounce_ver = use_signal(|| 0u64);

  use_effect(move || {
    let val = body.read().clone();
    let ver = *debounce_ver.read();
    spawn(async move {
      tokio::time::sleep(std::time::Duration::from_millis(800)).await;
      if *debounce_ver.read() == ver {
        persisted.set(val);
      }
    });
  });
```
- Also initialize `body` from `persisted` on mount. Add after the above block:
```rust
  use_effect({
    let initial = persisted.read().clone();
    move || {
      if !initial.is_empty() && body.read().is_empty() {
        body.set(initial.clone());
      }
    }
  });
```
- Update the `on_change` handler at line 60 to also bump the debounce version. Find:
```rust
          on_change: move |v: String| body.set(v),
```
- Replace with:
```rust
          on_change: move |v: String| {
              body.set(v);
              debounce_ver.set(debounce_ver() + 1);
          },
```
- In the `submit` closure (lines 21-29), after `body.set(String::new())` (line 28), also clear the persisted draft immediately:
```rust
    body.set(String::new());
    persisted.set(String::new());
```
- Why: The debounce pattern uses a version counter. Each keystroke bumps the version and spawns an async task that sleeps 800ms then checks if the version is still current. If the user typed more, the version changed and the old task does nothing. Only the last task (after 800ms of inactivity) actually writes.

### Step 2: Add debounced persistence to new_issue.rs
- Open `crates/lx-desktop/src/pages/issues/new_issue.rs`
- Replace the entire save `use_effect` block (lines 52-67) with:
```rust
  let mut save_gen = use_signal(|| 0u64);

  use_effect(move || {
    let _ = title.read();
    let _ = description.read();
    let _ = status.read();
    let _ = priority.read();
    let _ = assignee.read();
    save_gen.set(save_gen.peek() + 1);
  });

  use_effect(move || {
    let gen = *save_gen.read();
    if gen == 0 {
      return;
    }
    spawn(async move {
      tokio::time::sleep(std::time::Duration::from_millis(800)).await;
      if *save_gen.read() != gen {
        return;
      }
      let draft = IssueDraft {
        title: title.read().clone(),
        description: description.read().clone(),
        status: status.read().clone(),
        priority: priority.read().clone(),
        assignee: assignee.read().clone(),
      };
      if let Ok(json) = serde_json::to_string(&draft) {
        let js = format!(r#"localStorage.setItem("lx-new-issue-draft", {})"#, serde_json::json!(json));
        let _ = document::eval(&js).await;
      }
    });
  });
```
- Why: The first `use_effect` subscribes to all 5 signals and bumps a generation counter on any change. The second `use_effect` subscribes to `save_gen`, sleeps 800ms, then checks if the generation is still current. The `gen == 0` guard skips the initial mount (before any user input), preventing a spurious save of default values. This is the standard debounce pattern. No need for an explicit version bump in each oninput handler since reactive tracking handles it.

## File Size Check
- `comment_thread.rs`: was 82 lines, now ~100 lines (under 300)
- `new_issue.rs`: was 167 lines, now ~190 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compilation errors.
- Open the new issue dialog, type rapidly in the title field. Confirm that `localStorage` is not written on every keystroke (use browser devtools Storage tab or add a `console.log` temporarily). After stopping typing for 800ms, the draft should appear in storage.
- Open the comment thread, type a draft, close and reopen — the draft should persist after 800ms of inactivity.
