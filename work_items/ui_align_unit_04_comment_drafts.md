# Unit 04: CommentThread Drafts + Editor Wiring

## Goal

Replace the plain `<textarea>` in both CommentThread components with the `MarkdownEditor` component, add draft persistence via `dioxus_storage::use_persistent`, and wire Cmd+Enter to submit.

## Preconditions

- Unit 03 (MarkdownEditor cursor insertion) should be complete first, so the MarkdownEditor toolbar works with cursor-position insertion.
- The `MarkdownEditor` component exists at `crates/lx-desktop/src/components/markdown_editor.rs`.
- The `dioxus_storage::use_persistent` API is already used throughout the codebase (e.g., `pages/costs/overview.rs:62`, `pages/settings/state.rs:47`, `pages/approvals/list.rs:60`).

## Files to Modify

- `crates/lx-desktop/src/components/comment_thread.rs` (currently 79 lines)
- `crates/lx-desktop/src/pages/issues/comments.rs` (currently 65 lines)

## Context: dioxus_storage::use_persistent API

From existing usage across the codebase, the API is:

```rust
let signal = dioxus_storage::use_persistent("storage_key", || default_value);
```

This returns a `Signal<T>` that auto-persists to localStorage. The key must be unique per storage location.

## Context: MarkdownEditor Props

From `markdown_editor.rs` lines 13-19:

```rust
pub fn MarkdownEditor(
  value: String,
  on_change: EventHandler<String>,
  #[props(optional)] on_submit: Option<EventHandler<String>>,
  #[props(optional)] placeholder: Option<String>,
  #[props(optional)] class: Option<String>,
) -> Element
```

The `on_submit` handler fires on Cmd+Enter (already implemented in `EditorTextarea`, line 84-88).

## Steps

### Step 1: Update components/comment_thread.rs

Replace the entire file content. The key changes are:
1. Replace `use_signal(String::new)` with `dioxus_storage::use_persistent` for draft persistence.
2. Replace the `<textarea>` with `MarkdownEditor`.
3. Wire the `on_submit` prop for Cmd+Enter.

**Current code to replace** (lines 1-78, the entire file):

Replace with the following. Note: no code comments per CLAUDE.md rules.

```rust
use dioxus::prelude::*;

use super::identity::Identity;
use super::markdown_body::MarkdownBody;
use super::markdown_editor::MarkdownEditor;

#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
  pub id: String,
  pub author_name: String,
  pub body: String,
  pub created_at: String,
}

#[component]
pub fn CommentThread(comments: Vec<Comment>, on_add: EventHandler<String>) -> Element {
  let mut body = dioxus_storage::use_persistent("lx_comment_draft", String::new);
  let mut submitting = use_signal(|| false);
  let count = comments.len();

  let submit = move |text: String| {
    let text = text.trim().to_string();
    if text.is_empty() {
      return;
    }
    submitting.set(true);
    on_add.call(text);
    body.set(String::new());
    submitting.set(false);
  };

  rsx! {
    div { class: "space-y-4",
      h3 { class: "text-sm font-semibold", "Comments ({count})" }
      if comments.is_empty() {
        p { class: "text-sm text-[var(--on-surface-variant)]", "No comments yet." }
      }
      div { class: "space-y-3",
        for comment in comments.iter() {
          div { class: "border border-[var(--outline-variant)] p-3 overflow-hidden min-w-0 rounded-sm",
            div { class: "flex items-center justify-between mb-1",
              Identity {
                name: comment.author_name.clone(),
                size: "sm".to_string(),
              }
              span { class: "text-xs text-[var(--on-surface-variant)]",
                "{comment.created_at}"
              }
            }
            MarkdownBody {
              content: comment.body.clone(),
              class: "text-sm".to_string(),
            }
          }
        }
      }
      div { class: "space-y-2",
        MarkdownEditor {
          value: body(),
          on_change: move |v: String| body.set(v),
          on_submit: move |v: String| submit(v),
          placeholder: "Leave a comment...".to_string(),
        }
        div { class: "flex items-center justify-between",
          span { class: "text-[11px] text-[var(--outline)]",
            "Cmd+Enter to submit"
          }
          button {
            class: "px-3 py-1.5 bg-[var(--primary)] hover:brightness-110 text-[var(--on-primary)] text-sm rounded transition-colors disabled:opacity-50",
            disabled: body().trim().is_empty() || submitting(),
            onclick: move |_| submit(body()),
            if submitting() {
              "Posting..."
            } else {
              "Comment"
            }
          }
        }
      }
    }
  }
}
```

### Step 2: Update pages/issues/comments.rs

Replace the entire file content. The key changes are:
1. Replace `use_signal(String::new)` with `dioxus_storage::use_persistent` using a key that includes context to avoid collisions with the generic CommentThread.
2. Replace the `<textarea>` with `MarkdownEditor`.
3. Wire `on_submit` for Cmd+Enter.
4. Replace the existing `CommentBubble` to use `MarkdownBody` for rendering comment bodies.

**Current code to replace** (lines 1-65, entire file):

Replace with:

```rust
use dioxus::prelude::*;

use super::types::{AgentRef, IssueComment};
use crate::components::markdown_body::MarkdownBody;
use crate::components::markdown_editor::MarkdownEditor;
#[component]
pub fn CommentThread(comments: Vec<IssueComment>, agents: Vec<AgentRef>, on_add: EventHandler<String>) -> Element {
  let mut draft = dioxus_storage::use_persistent("lx_issue_comment_draft", String::new);

  let submit = move |text: String| {
    let body = text.trim().to_string();
    if !body.is_empty() {
      on_add.call(body);
      draft.set(String::new());
    }
  };

  rsx! {
    div { class: "space-y-4",
      h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Comments" }
      if comments.is_empty() {
        p { class: "text-sm text-[var(--outline)]", "No comments yet." }
      }
      for comment in comments.iter() {
        CommentBubble { comment: comment.clone(), agents: agents.clone() }
      }
      div { class: "space-y-2",
        MarkdownEditor {
          value: draft(),
          on_change: move |v: String| draft.set(v),
          on_submit: move |v: String| submit(v),
          placeholder: "Write a comment...".to_string(),
        }
        div { class: "flex items-center justify-between",
          span { class: "text-[11px] text-[var(--outline)]",
            "Cmd+Enter to submit"
          }
          button {
            class: "btn-primary-sm",
            disabled: draft.read().trim().is_empty(),
            onclick: move |_| submit(draft()),
            "Comment"
          }
        }
      }
    }
  }
}

#[component]
fn CommentBubble(comment: IssueComment, agents: Vec<AgentRef>) -> Element {
  let author = comment
    .author_agent_id
    .as_ref()
    .and_then(|aid| agents.iter().find(|a| &a.id == aid).map(|a| a.name.clone()))
    .unwrap_or_else(|| if comment.author_user_id.is_some() { "User".to_string() } else { "System".to_string() });

  rsx! {
    div { class: "border border-[var(--outline-variant)]/20 rounded-lg p-3 space-y-1",
      div { class: "flex items-center justify-between",
        span { class: "text-xs font-medium text-[var(--on-surface)]", "{author}" }
        span { class: "text-xs text-[var(--outline)]", "{comment.created_at}" }
      }
      MarkdownBody { content: comment.body.clone() }
    }
  }
}
```

### Step 3: Storage key design

The two CommentThread components use different storage keys to avoid overwriting each other:
- Generic CommentThread: `"lx_comment_draft"`
- Issue-specific CommentThread: `"lx_issue_comment_draft"`

## Verification

1. Run `just diagnose` to confirm no compilation errors or warnings.
2. Confirm both files are under 300 lines.
3. Test the generic CommentThread (used in agent detail or similar):
   - Type text in the editor. Navigate away from the page, then navigate back. Confirm the draft is preserved.
   - Click "Comment" to submit. Confirm the editor clears and the draft is removed from storage.
   - Type text, press Cmd+Enter. Confirm the comment submits and the editor clears.
   - Verify the toolbar buttons (Bold, Italic, Code, Link, Heading) work inside the comment editor.
   - Verify the Preview and Split modes work.
4. Test the issue-specific CommentThread (on the issue detail page):
   - Type text in the editor. Navigate away and back. Confirm the draft persists.
   - Submit via button. Confirm clear.
   - Submit via Cmd+Enter. Confirm clear.
   - Verify comment bodies now render markdown (bold, links, code blocks).
5. Verify that drafts in the generic CommentThread do not interfere with drafts in the issue CommentThread (different storage keys).
6. Check the "Cmd+Enter to submit" hint text appears below the editor in both locations.
