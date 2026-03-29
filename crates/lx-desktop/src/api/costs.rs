use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CostSummary {
  pub total_cents: Option<i64>,
  pub currency: Option<String>,
}

pub async fn summary(client: &ApiClient, company_id: &str, from: Option<&str>, to: Option<&str>) -> Result<CostSummary, ApiError> {
  let mut path = format!("/companies/{company_id}/costs/summary");
  let mut params = Vec::new();
  if let Some(f) = from {
    params.push(format!("from={f}"));
  }
  if let Some(t) = to {
    params.push(format!("to={t}"));
  }
  if !params.is_empty() {
    path.push('?');
    path.push_str(&params.join("&"));
  }
  client.get(&path).await
}
