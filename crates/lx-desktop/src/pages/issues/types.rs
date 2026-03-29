use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Issue {
  pub id: String,
  pub identifier: Option<String>,
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub priority: String,
  pub assignee_agent_id: Option<String>,
  pub assignee_user_id: Option<String>,
  pub project_id: Option<String>,
  pub parent_id: Option<String>,
  pub label_ids: Vec<String>,
  pub labels: Vec<IssueLabel>,
  pub created_at: String,
  pub updated_at: String,
  pub started_at: Option<String>,
  pub completed_at: Option<String>,
  pub created_by_agent_id: Option<String>,
  pub created_by_user_id: Option<String>,
  pub request_depth: u32,
  pub company_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueLabel {
  pub id: String,
  pub name: String,
  pub color: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueComment {
  pub id: String,
  pub body: String,
  pub author_agent_id: Option<String>,
  pub author_user_id: Option<String>,
  pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueDocument {
  pub key: String,
  pub title: Option<String>,
  pub body: String,
  pub format: String,
  pub latest_revision_id: Option<String>,
  pub updated_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IssueWorkspace {
  pub id: String,
  pub mode: Option<String>,
  pub branch_name: Option<String>,
  pub worktree_path: Option<String>,
  pub status: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AgentRef {
  pub id: String,
  pub name: String,
  pub icon: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectRef {
  pub id: String,
  pub name: String,
  pub color: Option<String>,
}

pub const STATUS_ORDER: &[&str] = &["in_progress", "todo", "backlog", "in_review", "blocked", "done", "cancelled"];

pub const PRIORITY_ORDER: &[&str] = &["critical", "high", "medium", "low"];

pub const QUICK_FILTER_PRESETS: &[(&str, &[&str])] =
  &[("All", &[]), ("Active", &["todo", "in_progress", "in_review", "blocked"]), ("Backlog", &["backlog"]), ("Done", &["done", "cancelled"])];

#[derive(Clone, Debug, PartialEq)]
pub enum IssueViewMode {
  List,
  Board,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IssueViewState {
  pub statuses: Vec<String>,
  pub priorities: Vec<String>,
  pub assignees: Vec<String>,
  pub sort_field: String,
  pub sort_dir: String,
  pub group_by: String,
  pub view_mode: IssueViewMode,
  pub search: String,
}

impl Default for IssueViewState {
  fn default() -> Self {
    Self {
      statuses: Vec::new(),
      priorities: Vec::new(),
      assignees: Vec::new(),
      sort_field: "updated".to_string(),
      sort_dir: "desc".to_string(),
      group_by: "none".to_string(),
      view_mode: IssueViewMode::List,
      search: String::new(),
    }
  }
}

pub fn status_label(status: &str) -> String {
  status
    .replace('_', " ")
    .split_whitespace()
    .map(|w| {
      let mut c = w.chars();
      match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}

pub fn status_icon_class(status: &str) -> &'static str {
  match status {
    "todo" => "text-blue-500",
    "in_progress" => "text-yellow-500",
    "in_review" => "text-purple-500",
    "blocked" => "text-red-500",
    "done" => "text-green-500",
    "cancelled" => "text-neutral-400",
    "backlog" => "text-neutral-500",
    _ => "text-neutral-400",
  }
}

pub fn priority_icon_class(priority: &str) -> &'static str {
  match priority {
    "critical" => "text-red-600",
    "high" => "text-orange-500",
    "medium" => "text-yellow-500",
    "low" => "text-blue-400",
    _ => "text-neutral-400",
  }
}

pub fn filter_issues(issues: &[Issue], state: &IssueViewState) -> Vec<Issue> {
  let mut result: Vec<Issue> = issues
    .iter()
    .filter(|i| {
      if !state.statuses.is_empty() && !state.statuses.contains(&i.status) {
        return false;
      }
      if !state.priorities.is_empty() && !state.priorities.contains(&i.priority) {
        return false;
      }
      if !state.search.is_empty() {
        let q = state.search.to_lowercase();
        let title_match = i.title.to_lowercase().contains(&q);
        let id_match = i.identifier.as_ref().map(|id| id.to_lowercase().contains(&q)).unwrap_or(false);
        if !title_match && !id_match {
          return false;
        }
      }
      true
    })
    .cloned()
    .collect();

  let dir: i32 = if state.sort_dir == "asc" { 1 } else { -1 };
  result.sort_by(|a, b| {
    let cmp = match state.sort_field.as_str() {
      "status" => {
        let ai = STATUS_ORDER.iter().position(|s| *s == a.status).unwrap_or(99);
        let bi = STATUS_ORDER.iter().position(|s| *s == b.status).unwrap_or(99);
        ai.cmp(&bi)
      },
      "priority" => {
        let ai = PRIORITY_ORDER.iter().position(|s| *s == a.priority).unwrap_or(99);
        let bi = PRIORITY_ORDER.iter().position(|s| *s == b.priority).unwrap_or(99);
        ai.cmp(&bi)
      },
      "title" => a.title.cmp(&b.title),
      "created" => a.created_at.cmp(&b.created_at),
      _ => a.updated_at.cmp(&b.updated_at),
    };
    if dir > 0 { cmp } else { cmp.reverse() }
  });
  result
}
