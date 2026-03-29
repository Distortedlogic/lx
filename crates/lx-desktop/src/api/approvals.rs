use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
  pub id: String,
  pub status: Option<String>,
  pub payload: Option<serde_json::Value>,
  pub company_id: Option<String>,
  pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Approval>, ApiError> {
  client.get(&format!("/companies/{company_id}/approvals")).await
}

pub async fn approve(client: &ApiClient, id: &str, note: Option<&str>) -> Result<Approval, ApiError> {
  client.post(&format!("/approvals/{id}/approve"), &serde_json::json!({ "decisionNote": note })).await
}

pub async fn reject(client: &ApiClient, id: &str, note: Option<&str>) -> Result<Approval, ApiError> {
  client.post(&format!("/approvals/{id}/reject"), &serde_json::json!({ "decisionNote": note })).await
}
