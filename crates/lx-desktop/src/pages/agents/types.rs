use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentSummary {
  pub id: String,
  pub name: String,
  pub role: String,
  pub title: Option<String>,
  pub status: String,
  pub adapter_type: String,
  pub icon: Option<String>,
  pub last_heartbeat_at: Option<String>,
  pub reports_to: Option<String>,
  pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentDetail {
  pub id: String,
  pub name: String,
  pub role: String,
  pub title: Option<String>,
  pub status: String,
  pub adapter_type: String,
  pub icon: Option<String>,
  pub last_heartbeat_at: Option<String>,
  pub reports_to: Option<String>,
  pub created_at: String,
  pub budget_monthly_cents: i64,
  pub spent_monthly_cents: i64,
  pub adapter_config: serde_json::Value,
  pub runtime_config: serde_json::Value,
  pub pause_reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FilterTab {
  All,
  Active,
  Paused,
  Error,
}

impl FilterTab {
  pub fn label(&self) -> &'static str {
    match self {
      Self::All => "All",
      Self::Active => "Active",
      Self::Paused => "Paused",
      Self::Error => "Error",
    }
  }

  pub fn matches(&self, status: &str) -> bool {
    match self {
      Self::All => status != "terminated",
      Self::Active => matches!(status, "active" | "running" | "idle"),
      Self::Paused => status == "paused",
      Self::Error => status == "error",
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AgentDetailTab {
  Overview,
  Runs,
  Config,
  Skills,
  Budget,
}

impl AgentDetailTab {
  pub fn label(&self) -> &'static str {
    match self {
      Self::Overview => "Overview",
      Self::Runs => "Runs",
      Self::Config => "Configuration",
      Self::Skills => "Skills",
      Self::Budget => "Budget",
    }
  }

  pub fn all() -> &'static [AgentDetailTab] {
    &[Self::Overview, Self::Runs, Self::Config, Self::Skills, Self::Budget]
  }
}

pub const ADAPTER_LABELS: &[(&str, &str)] = &[
  ("claude_local", "Claude"),
  ("codex_local", "Codex"),
  ("gemini_local", "Gemini"),
  ("opencode_local", "OpenCode"),
  ("cursor", "Cursor"),
  ("hermes_local", "Hermes"),
  ("openclaw_gateway", "OpenClaw Gateway"),
  ("process", "Process"),
  ("http", "HTTP"),
];

pub fn adapter_label(adapter_type: &str) -> &str {
  ADAPTER_LABELS.iter().find(|(k, _)| *k == adapter_type).map(|(_, v)| *v).unwrap_or(adapter_type)
}

pub const ROLE_LABELS: &[(&str, &str)] =
  &[("ceo", "CEO"), ("executive", "Executive"), ("manager", "Manager"), ("general", "General"), ("specialist", "Specialist")];

pub fn role_label(role: &str) -> &str {
  ROLE_LABELS.iter().find(|(k, _)| *k == role).map(|(_, v)| *v).unwrap_or(role)
}

pub fn status_dot_class(status: &str) -> &'static str {
  match status {
    "running" => "status-dot-running",
    "active" | "idle" => "status-dot-active",
    "paused" => "status-dot-paused",
    "error" => "status-dot-error",
    _ => "status-dot-default",
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LxAgentConfig {
  pub name: String,
  pub source_text: String,
  pub adapter_type: String,
  pub model: String,
  pub tools: Vec<LxToolDecl>,
  pub channels: Vec<String>,
  pub fields: Vec<LxAgentField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LxToolDecl {
  pub path: String,
  pub alias: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LxAgentField {
  pub name: String,
  pub value: String,
}
