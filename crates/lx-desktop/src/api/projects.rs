use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
  pub id: String,
  pub name: String,
  pub description: Option<String>,
  pub status: Option<String>,
  pub archived_at: Option<String>,
  pub company_id: Option<String>,
  pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str) -> Result<Vec<Project>, ApiError> {
  client.get(&format!("/companies/{company_id}/projects")).await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Project, ApiError> {
  client.get(&format!("/projects/{id}")).await
}

pub async fn create(client: &ApiClient, company_id: &str, input: &serde_json::Value) -> Result<Project, ApiError> {
  client.post(&format!("/companies/{company_id}/projects"), input).await
}
