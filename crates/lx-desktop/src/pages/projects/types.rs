use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Project {
  pub id: String,
  pub name: String,
  pub description: Option<String>,
  pub status: String,
  pub color: String,
  pub target_date: Option<String>,
  pub goal_ids: Vec<String>,
  pub archived_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Goal {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub level: String,
  pub parent_id: Option<String>,
  pub owner_agent_id: Option<String>,
  pub created_at: String,
  pub updated_at: String,
}

pub const PROJECT_STATUSES: &[&str] = &["backlog", "planned", "in_progress", "completed", "cancelled"];

pub const PROJECT_COLORS: &[&str] = &["#6366f1", "#8b5cf6", "#ec4899", "#f43f5e", "#ef4444", "#f97316", "#eab308", "#22c55e", "#14b8a6", "#06b6d4"];

pub const GOAL_STATUSES: &[&str] = &["planned", "in_progress", "completed", "cancelled"];

pub const GOAL_LEVELS: &[&str] = &["company", "team", "agent", "task"];
