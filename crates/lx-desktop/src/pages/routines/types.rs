use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Routine {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub status: String,
  pub project_id: Option<String>,
  pub assignee_agent_id: Option<String>,
  pub priority: String,
  pub concurrency_policy: String,
  pub catch_up_policy: String,
  pub cron_expression: Option<String>,
  pub last_run_at: Option<String>,
  pub last_run_status: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub reports_to: Option<String>,
  #[serde(default)]
  pub connected_to: Vec<String>,
}

pub const CONCURRENCY_POLICIES: &[(&str, &str)] = &[
  ("coalesce_if_active", "If a run is already active, keep just one follow-up run queued"),
  ("always_enqueue", "Queue every trigger occurrence, even if already running"),
  ("skip_if_active", "Drop new trigger occurrences while a run is still active"),
];

pub const CATCH_UP_POLICIES: &[(&str, &str)] =
  &[("skip_missed", "Ignore windows that were missed while paused"), ("enqueue_missed_with_cap", "Catch up missed windows in capped batches")];

pub const PRIORITIES: &[&str] = &["low", "medium", "high", "critical"];
