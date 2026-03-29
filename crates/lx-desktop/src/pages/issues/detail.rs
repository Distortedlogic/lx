use dioxus::prelude::*;

use super::comments::CommentThread;
use super::documents::DocumentsSection;
use super::properties::IssuePropertiesPanel;
use super::types::{AgentRef, Issue, IssueComment, IssueDocument, IssueWorkspace};
use super::workspace_card::WorkspaceCard;
use crate::pages::agents::list::StatusBadge;

#[component]
pub fn IssueDetailPage(
  issue: Issue,
  comments: Vec<IssueComment>,
  documents: Vec<IssueDocument>,
  workspace: Option<IssueWorkspace>,
  agents: Vec<AgentRef>,
  on_back: EventHandler<()>,
  on_update: EventHandler<(String, String)>,
  on_add_comment: EventHandler<String>,
) -> Element {
  let mut editing_title = use_signal(|| false);
  let mut draft_title = use_signal(|| issue.title.clone());

  let id_display = issue.identifier.as_deref().unwrap_or(&issue.id[..8.min(issue.id.len())]);

  rsx! {
    div { class: "flex flex-col h-full overflow-auto",
      div { class: "flex items-center gap-2 px-4 py-3 border-b border-[var(--outline-variant)]/30",
        button {
          class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)]",
          onclick: move |_| on_back.call(()),
          "< Issues"
        }
        span { class: "text-xs font-mono text-[var(--outline)]", "{id_display}" }
        StatusBadge { status: issue.status.clone() }
      }
      div { class: "flex flex-1 min-h-0",
        div { class: "flex-1 p-4 overflow-auto space-y-6",
          if *editing_title.read() {
            input {
              class: "text-xl font-semibold w-full bg-transparent outline-none text-[var(--on-surface)] border-b border-[var(--primary)]",
              value: "{draft_title}",
              oninput: move |evt| draft_title.set(evt.value().to_string()),
              onkeydown: move |evt| {
                  if evt.key() == Key::Enter {
                      on_update.call(("title".to_string(), draft_title.read().clone()));
                      editing_title.set(false);
                  }
              },
            }
          } else {
            h1 {
              class: "text-xl font-semibold text-[var(--on-surface)] cursor-pointer hover:text-[var(--primary)] transition-colors",
              onclick: move |_| editing_title.set(true),
              "{issue.title}"
            }
          }
          if let Some(desc) = &issue.description {
            div { class: "text-sm text-[var(--on-surface-variant)] whitespace-pre-wrap",
              "{desc}"
            }
          }
          if let Some(ws) = &workspace {
            WorkspaceCard { workspace: ws.clone() }
          }
          if !documents.is_empty() {
            DocumentsSection { documents: documents.clone() }
          }
          CommentThread {
            comments: comments.clone(),
            agents: agents.clone(),
            on_add: on_add_comment,
          }
        }
        div { class: "w-64 shrink-0 border-l border-[var(--outline-variant)]/30 p-4 overflow-auto",
          IssuePropertiesPanel {
            issue: issue.clone(),
            agents: agents.clone(),
            on_update,
          }
        }
      }
    }
  }
}
