use dioxus::prelude::*;

use super::types::IssueWorkspace;
use crate::styles::PROPERTY_LABEL;

#[component]
pub fn WorkspaceCard(workspace: IssueWorkspace) -> Element {
  let mode_label = match workspace.mode.as_deref() {
    Some("isolated_workspace") => "Isolated workspace",
    Some("operator_branch") => "Operator branch",
    Some("cloud_sandbox") => "Cloud sandbox",
    Some("adapter_managed") => "Adapter managed",
    _ => "Workspace",
  };

  rsx! {
    div { class: "border border-[var(--outline-variant)]/20 rounded-lg p-4 space-y-3",
      div { class: "flex items-center gap-2",
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
          "folder_open"
        }
        span { class: "text-sm font-medium text-[var(--on-surface)]", "{mode_label}" }
      }
      if let Some(branch) = &workspace.branch_name {
        div { class: "flex items-center gap-3 py-1",
          span { class: PROPERTY_LABEL, "Branch" }
          span { class: "text-sm font-mono text-[var(--on-surface)] break-all",
            "{branch}"
          }
        }
      }
      if let Some(path) = &workspace.worktree_path {
        div { class: "flex items-center gap-3 py-1",
          span { class: PROPERTY_LABEL, "Path" }
          span { class: "text-sm font-mono text-[var(--on-surface)] break-all",
            "{path}"
          }
        }
      }
    }
  }
}
