# Unit 10: NewIssueDialog enrichment

## Goal

Replace the plain textarea with MarkdownEditor, add draft persistence via localStorage, and prepare select fields for custom Select integration after Unit 06 lands.

## Preconditions

- `crates/lx-desktop/src/components/markdown_editor.rs` exists with `MarkdownEditor` component accepting props: `value: String`, `on_change: EventHandler<String>`, `on_submit: Option<EventHandler<String>>`, `placeholder: Option<String>`, `class: Option<String>`
- Unit 06 (Custom Select) should be complete for the final select replacement step (Step 5). Steps 1-4 and 6-7 can be done independently.

## Files to Modify

- `crates/lx-desktop/src/pages/issues/new_issue.rs`

## Steps

### 1. Add import for MarkdownEditor

At the top of `crates/lx-desktop/src/pages/issues/new_issue.rs`, add:

```rust
use crate::components::markdown_editor::MarkdownEditor;
```

The existing imports are:

```rust
use dioxus::prelude::*;

use super::types::{AgentRef, PRIORITY_ORDER, STATUS_ORDER};
```

Add the new import after the styles import.

### 2. Replace textarea with MarkdownEditor

Find the `textarea` block (lines 43-48 in the current file):

```rust
textarea {
    class: "w-full rounded border border-[var(--outline-variant)] px-3 py-2 bg-transparent outline-none text-sm min-h-[100px] resize-y placeholder:text-[var(--outline)]/40",
    placeholder: "Description (optional)",
    value: "{description}",
    oninput: move |evt| description.set(evt.value().to_string()),
}
```

Replace it with:

```rust
MarkdownEditor {
    value: description.read().clone(),
    on_change: move |val: String| description.set(val),
    placeholder: "Description (optional)".to_string(),
    class: "min-h-[120px]".to_string(),
}
```

### 3. Add draft persistence via document::eval localStorage

Drafts save to `localStorage` with key `"lx-new-issue-draft"`. The draft stores all 5 fields as JSON.

Add this struct at the top of the file (after imports, before the component):

```rust
#[derive(serde::Serialize, serde::Deserialize, Default)]
struct IssueDraft {
    title: String,
    description: String,
    status: String,
    priority: String,
    assignee: Option<String>,
}
```

Inside the `NewIssueDialog` component, after the signal declarations and before `if !open`, add a `use_effect` to load the draft when the dialog opens:

```rust
use_effect(move || {
    if open {
        spawn(async move {
            let result = document::eval(
                r#"
                let d = localStorage.getItem("lx-new-issue-draft");
                return d || "";
                "#,
            )
            .await;
            if let Ok(val) = result {
                if let Ok(s) = val.as_str() {
                    if let Ok(draft) = serde_json::from_str::<IssueDraft>(s) {
                        title.set(draft.title);
                        description.set(draft.description);
                        status.set(draft.status);
                        priority.set(draft.priority);
                        assignee.set(draft.assignee);
                    }
                }
            }
        });
    }
});
```

Add a save-draft effect that runs whenever any field changes. Place it right after the load effect:

```rust
use_effect(move || {
    let draft = IssueDraft {
        title: title.read().clone(),
        description: description.read().clone(),
        status: status.read().clone(),
        priority: priority.read().clone(),
        assignee: assignee.read().clone(),
    };
    if let Ok(json) = serde_json::to_string(&draft) {
        let js = format!(r#"localStorage.setItem("lx-new-issue-draft", {})"#, serde_json::json!(json));
        let js = js.clone();
        spawn(async move { let _ = document::eval(&js).await; });
    }
});
```

Add a clear-draft call when the issue is successfully created. In the `on_create` button's `onclick` handler, before calling `on_create.call(NewIssuePayload)`, add:

```rust
spawn(async move { let _ = document::eval(r#"localStorage.removeItem("lx-new-issue-draft")"#).await; });
```

Drafts persist across cancel so the user can reopen and continue. Only clear on successful create.

### 4. Add Cmd+Enter keyboard shortcut to submit

Add an `onkeydown` handler to the outer dialog container div (the one with class `"fixed inset-0 z-50 ..."`). This enables Cmd+Enter from anywhere in the dialog to submit:

```rust
onkeydown: move |evt: KeyboardEvent| {
    if evt.modifiers().meta() && evt.key() == Key::Enter && !title.read().trim().is_empty() {
        spawn(async move { let _ = document::eval(r#"localStorage.removeItem("lx-new-issue-draft")"#).await; });
        on_create.call(NewIssuePayload);
    }
},
```

Place this right after the `onclick: move |_| on_close.call(())` on the backdrop div.

### 5. Verify serde dependency

The `IssueDraft` struct uses `serde::Serialize` and `serde::Deserialize`. Confirm that `serde` with `derive` feature is already in `crates/lx-desktop/Cargo.toml` dependencies. It should be, since `crates/lx-desktop/src/pages/issues/types.rs` already uses `#[derive(Serialize, Deserialize)]`.

### 7. Ensure file stays under 300 lines

After all changes, `new_issue.rs` should be approximately 160-180 lines. Well under the 300 line limit.

## Verification

1. Run `just diagnose` -- must compile with no warnings and no clippy errors.
2. Open the desktop app and navigate to the Issues page.
3. Open the New Issue dialog and verify:
   - The description field is now a MarkdownEditor with toolbar (bold, italic, code, link, heading buttons) and mode tabs (Edit/Preview/Split).
   - Typing in any field and closing the dialog, then reopening, restores the draft values.
   - Creating an issue clears the draft (reopen dialog to confirm fields are empty).
   - Pressing Cmd+Enter (or Ctrl+Enter on Linux) submits the form when the title is non-empty.
   - Pressing Cmd+Enter with an empty title does nothing.
   - The close button in the header still shows the Material Symbol "close" icon (this was already correct -- verify it was not regressed).
4. The file is under 300 lines.
