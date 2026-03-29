use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityEvent {
  pub id: Option<String>,
  pub entity_type: Option<String>,
  pub entity_id: Option<String>,
  pub action: Option<String>,
  pub actor_type: Option<String>,
  pub actor_id: Option<String>,
  pub details: Option<serde_json::Value>,
  pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str, entity_type: Option<&str>) -> Result<Vec<ActivityEvent>, ApiError> {
  let mut path = format!("/companies/{company_id}/activity");
  if let Some(et) = entity_type {
    path.push_str(&format!("?entityType={et}"));
  }
  client.get(&path).await
}
