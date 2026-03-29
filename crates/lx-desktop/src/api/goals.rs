use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Goal {
  pub id: String,
  pub title: String,
  pub description: Option<String>,
  pub level: Option<String>,
  pub status: Option<String>,
  pub company_id: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Goal>, ApiError> {
  client.get(&format!("/companies/{company_id}/goals")).await
}

pub async fn create(client: &ApiClient, company_id: &str, input: &serde_json::Value) -> Result<Goal, ApiError> {
  client.post(&format!("/companies/{company_id}/goals"), input).await
}
