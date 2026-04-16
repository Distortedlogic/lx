use lx_graph_editor::catalog::GraphCredentialOption;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowCredentialRecord {
  pub id: String,
  pub namespace: String,
  pub kind: String,
  pub label: String,
  pub detail: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WorkflowCredentialStore {
  pub records: Vec<WorkflowCredentialRecord>,
}

impl WorkflowCredentialStore {
  pub fn sample() -> Self {
    Self {
      records: vec![
        WorkflowCredentialRecord {
          id: "cred-http-news".to_string(),
          namespace: "workflow".to_string(),
          kind: "http_api".to_string(),
          label: "News API".to_string(),
          detail: Some("Shared read-only API credential".to_string()),
        },
        WorkflowCredentialRecord {
          id: "cred-feed-delivery".to_string(),
          namespace: "workflow".to_string(),
          kind: "feed_delivery".to_string(),
          label: "Feed Delivery".to_string(),
          detail: Some("Publishes into the internal reader feed".to_string()),
        },
        WorkflowCredentialRecord {
          id: "cred-slack-briefing".to_string(),
          namespace: "workflow".to_string(),
          kind: "slack_bot".to_string(),
          label: "Slack Briefing Bot".to_string(),
          detail: Some("Posts to #research-intel".to_string()),
        },
      ],
    }
  }

  pub fn graph_options(&self) -> Vec<GraphCredentialOption> {
    self
      .records
      .iter()
      .map(|record| GraphCredentialOption {
        id: record.id.clone(),
        namespace: record.namespace.clone(),
        kind: record.kind.clone(),
        label: record.label.clone(),
        detail: record.detail.clone(),
      })
      .collect()
  }
}

pub fn sample_workflow_credentials() -> WorkflowCredentialStore {
  WorkflowCredentialStore::sample()
}
