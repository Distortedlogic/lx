use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Company {
  pub id: String,
  pub name: String,
  pub description: Option<String>,
  pub status: Option<String>,
  pub issue_prefix: Option<String>,
  pub budget_monthly_cents: Option<i64>,
  pub brand_color: Option<String>,
  pub created_at: Option<String>,
  pub updated_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCompanyInput {
  pub name: String,
  pub description: Option<String>,
}

pub async fn list(client: &ApiClient) -> Result<Vec<Company>, ApiError> {
  client.get("/companies").await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Company, ApiError> {
  client.get(&format!("/companies/{id}")).await
}

pub async fn create(client: &ApiClient, input: &CreateCompanyInput) -> Result<Company, ApiError> {
  client.post("/companies", input).await
}
