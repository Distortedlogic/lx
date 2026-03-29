use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
  pub id: String,
  pub name: String,
  pub role: String,
  pub title: Option<String>,
  pub status: Option<String>,
  pub adapter_type: Option<String>,
  pub company_id: Option<String>,
  pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Agent>, ApiError> {
  client.get(&format!("/companies/{company_id}/agents")).await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Agent, ApiError> {
  client.get(&format!("/agents/detail/{id}")).await
}

pub async fn create(client: &ApiClient, company_id: &str, input: &serde_json::Value) -> Result<Agent, ApiError> {
  client.post(&format!("/companies/{company_id}/agents"), input).await
}
