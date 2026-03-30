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

  let mut submit = move |text: String| {
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
