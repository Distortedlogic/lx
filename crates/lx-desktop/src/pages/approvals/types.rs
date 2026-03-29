use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Approval {
  pub id: String,
  pub approval_type: String,
  pub status: String,
  pub requested_by: Option<String>,
  pub payload: ApprovalPayload,
  pub decision_note: Option<String>,
  pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApprovalPayload {
  pub name: Option<String>,
  pub role: Option<String>,
  pub title: Option<String>,
  pub description: Option<String>,
  pub amount: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ApprovalComment {
  pub id: String,
  pub body: String,
  pub author: Option<String>,
  pub created_at: String,
}

pub const APPROVAL_TYPES: &[(&str, &str)] =
  &[("hire_agent", "Hire Agent"), ("approve_ceo_strategy", "CEO Strategy"), ("budget_override_required", "Budget Override")];

pub fn approval_type_label(t: &str) -> &str {
  APPROVAL_TYPES.iter().find(|(k, _)| *k == t).map(|(_, v)| *v).unwrap_or(t)
}

pub fn approval_type_icon(t: &str) -> &'static str {
  match t {
    "hire_agent" => "person_add",
    "approve_ceo_strategy" => "lightbulb",
    "budget_override_required" => "gpp_maybe",
    _ => "verified_user",
  }
}
