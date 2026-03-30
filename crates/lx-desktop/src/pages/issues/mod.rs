mod comments;
mod detail;
mod documents;
mod kanban;
mod kanban_card;
mod list;
mod new_issue;
mod properties;
pub mod types;
mod workspace_card;

use dioxus::prelude::*;

use self::detail::IssueDetailPage;
use self::list::IssuesList;
use self::new_issue::{NewIssueDialog, NewIssuePayload};
use self::types::{AgentRef, Issue};

#[component]
pub fn Issues() -> Element {
  let mut selected_issue_id = use_signal(|| Option::<String>::None);
  let mut show_new_dialog = use_signal(|| false);
  let issues: Vec<Issue> = Vec::new();
  let agents: Vec<AgentRef> = Vec::new();

  rsx! {
    match selected_issue_id.read().as_ref() {
        Some(_id) => rsx! {
          IssueDetailPage {
            issue: Issue {
                id: String::new(),
                identifier: None,
                title: "Loading...".to_string(),
                description: None,
                status: "todo".to_string(),
                priority: "medium".to_string(),
                assignee_agent_id: None,
                assignee_user_id: None,
                project_id: None,
                parent_id: None,
                label_ids: Vec::new(),
                labels: Vec::new(),
                created_at: String::new(),
                updated_at: String::new(),
                started_at: None,
                completed_at: None,
                created_by_agent_id: None,
                created_by_user_id: None,
                request_depth: 0,
                company_id: None,
            },
            comments: Vec::new(),
            documents: Vec::new(),
            workspace: None,
            agents: agents.clone(),
            on_back: move |_| selected_issue_id.set(None),
            on_update: move |_: (String, String)| {},
            on_add_comment: move |_: String| {},
          }
        },
        None => rsx! {
          IssuesList {
            issues,
            agents: agents.clone(),
            on_select: move |id: String| selected_issue_id.set(Some(id)),
            on_new_issue: move |_| show_new_dialog.set(true),
            on_update: move |_: (String, String, String)| {},
          }
        },
    }
    NewIssueDialog {
      open: *show_new_dialog.read(),
      agents: agents.clone(),
      on_close: move |_| show_new_dialog.set(false),
      on_create: move |_payload: NewIssuePayload| {
          show_new_dialog.set(false);
      },
    }
  }
}

#[component]
pub fn IssueDetail(issue_id: String) -> Element {
  let agents: Vec<AgentRef> = Vec::new();
  let nav = navigator();

  rsx! {
    IssueDetailPage {
      issue: Issue {
          id: issue_id.clone(),
          identifier: Some(issue_id),
          title: "Loading...".to_string(),
          description: None,
          status: "todo".to_string(),
          priority: "medium".to_string(),
          assignee_agent_id: None,
          assignee_user_id: None,
          project_id: None,
          parent_id: None,
          label_ids: Vec::new(),
          labels: Vec::new(),
          created_at: String::new(),
          updated_at: String::new(),
          started_at: None,
          completed_at: None,
          created_by_agent_id: None,
          created_by_user_id: None,
          request_depth: 0,
          company_id: None,
      },
      comments: Vec::new(),
      documents: Vec::new(),
      workspace: None,
      agents: agents.clone(),
      on_back: move |_| {
          nav.go_back();
      },
      on_update: move |_: (String, String)| {},
      on_add_comment: move |_: String| {},
    }
  }
}
