use dioxus::prelude::*;

use super::identity::Identity;
use super::markdown_body::MarkdownBody;

#[derive(Clone, Debug, PartialEq)]
pub struct Comment {
  pub id: String,
  pub author_name: String,
  pub body: String,
  pub created_at: String,
}

#[component]
pub fn CommentThread(comments: Vec<Comment>, on_add: EventHandler<String>) -> Element {
  let mut body = use_signal(String::new);
  let mut submitting = use_signal(|| false);
  let count = comments.len();

  let handle_submit = move |_| {
    let text = body().trim().to_string();
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
        p { class: "text-sm text-gray-400", "No comments yet." }
      }
      div { class: "space-y-3",
        for comment in comments.iter() {
          div { class: "border border-gray-700 p-3 overflow-hidden min-w-0 rounded-sm",
            div { class: "flex items-center justify-between mb-1",
              Identity {
                name: comment.author_name.clone(),
                size: "sm".to_string(),
              }
              span { class: "text-xs text-gray-400", "{comment.created_at}" }
            }
            MarkdownBody {
              content: comment.body.clone(),
              class: "text-sm".to_string(),
            }
          }
        }
      }
      div { class: "space-y-2",
        textarea {
          class: "w-full bg-gray-800 border border-gray-600 rounded p-2 text-sm outline-none resize-none min-h-[60px] placeholder:text-gray-500",
          placeholder: "Leave a comment...",
          value: "{body}",
          oninput: move |evt: Event<FormData>| body.set(evt.value()),
        }
        div { class: "flex items-center justify-end",
          button {
            class: "px-3 py-1.5 bg-blue-600 hover:bg-blue-500 text-white text-sm rounded transition-colors disabled:opacity-50",
            disabled: body().trim().is_empty() || submitting(),
            onclick: handle_submit,
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
