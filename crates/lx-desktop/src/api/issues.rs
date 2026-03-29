use serde::{Deserialize, Serialize};

use super::client::{ApiClient, ApiError};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Issue {
  pub id: String,
  pub title: String,
  pub identifier: Option<String>,
  pub status: Option<String>,
  pub priority: Option<String>,
  pub assignee_agent_id: Option<String>,
  pub assignee_user_id: Option<String>,
  pub project_id: Option<String>,
  pub description: Option<String>,
  pub created_at: Option<String>,
  pub updated_at: Option<String>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueFilters {
  pub status: Option<String>,
  pub project_id: Option<String>,
  pub assignee_agent_id: Option<String>,
  pub q: Option<String>,
}

pub async fn list(client: &ApiClient, company_id: &str, filters: Option<&IssueFilters>) -> Result<Vec<Issue>, ApiError> {
  let mut path = format!("/companies/{company_id}/issues");
  if let Some(f) = filters {
    let mut params = Vec::new();
    if let Some(s) = &f.status {
      params.push(format!("status={s}"));
    }
    if let Some(p) = &f.project_id {
      params.push(format!("projectId={p}"));
    }
    if let Some(a) = &f.assignee_agent_id {
      params.push(format!("assigneeAgentId={a}"));
    }
    if let Some(q) = &f.q {
      params.push(format!("q={q}"));
    }
    if !params.is_empty() {
      path.push('?');
      path.push_str(&params.join("&"));
    }
  }
  client.get(&path).await
}

pub async fn get(client: &ApiClient, id: &str) -> Result<Issue, ApiError> {
  client.get(&format!("/issues/detail/{id}")).await
}

pub async fn create(client: &ApiClient, company_id: &str, input: &serde_json::Value) -> Result<Issue, ApiError> {
  client.post(&format!("/companies/{company_id}/issues"), input).await
}
