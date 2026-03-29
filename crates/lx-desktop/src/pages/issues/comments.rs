use dioxus::prelude::*;

use super::types::{AgentRef, IssueComment};
use crate::styles::BTN_PRIMARY_SM;

#[component]
pub fn CommentThread(comments: Vec<IssueComment>, agents: Vec<AgentRef>, on_add: EventHandler<String>) -> Element {
  let mut draft = use_signal(String::new);

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
        textarea {
          class: "w-full rounded border border-[var(--outline-variant)] px-3 py-2 bg-transparent outline-none text-sm min-h-[80px] resize-y placeholder:text-[var(--outline)]/40",
          placeholder: "Write a comment...",
          value: "{draft}",
          oninput: move |evt| draft.set(evt.value().to_string()),
        }
        div { class: "flex justify-end",
          button {
            class: BTN_PRIMARY_SM,
            disabled: draft.read().trim().is_empty(),
            onclick: move |_| {
                let body = draft.read().trim().to_string();
                if !body.is_empty() {
                    on_add.call(body);
                    draft.set(String::new());
                }
            },
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
      div { class: "text-sm text-[var(--on-surface-variant)] whitespace-pre-wrap",
        "{comment.body}"
      }
    }
  }
}
