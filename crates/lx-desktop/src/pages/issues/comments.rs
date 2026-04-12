use dioxus::prelude::*;

use super::types::{AgentRef, IssueComment};
use crate::components::markdown_body::MarkdownBody;
use crate::components::markdown_editor::MarkdownEditor;
use crate::components::mention_popup::MentionCandidate;
#[component]
pub fn CommentThread(comments: Vec<IssueComment>, agents: Vec<AgentRef>, on_add: EventHandler<String>) -> Element {
  let mut draft = dioxus_storage::use_persistent("lx_issue_comment_draft", String::new);

  let mention_candidates: Vec<MentionCandidate> = agents.iter().map(|a| MentionCandidate { id: a.id.clone(), name: a.name.clone() }).collect();

  let mut submit = move |text: String| {
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
          placeholder: "Leave a comment (drag files here)...".to_string(),
          mention_candidates: mention_candidates.clone(),
        }
        div { class: "flex items-center justify-between",
          span { class: "text-[11px] text-[var(--outline)]", "Cmd+Enter to submit" }
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
