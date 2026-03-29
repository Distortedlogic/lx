use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeartbeatRun {
  pub id: String,
  pub status: Option<String>,
  pub agent_id: Option<String>,
  pub started_at: Option<String>,
  pub finished_at: Option<String>,
  pub created_at: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str, agent_id: Option<&str>) -> Result<Vec<HeartbeatRun>, ApiError> {
  let mut path = format!("/companies/{company_id}/heartbeat-runs");
  if let Some(aid) = agent_id {
    path.push_str(&format!("?agentId={aid}"));
  }
  client.get(&path).await
}

pub async fn get(client: &ApiClient, run_id: &str) -> Result<HeartbeatRun, ApiError> {
  client.get(&format!("/heartbeat-runs/{run_id}")).await
}
