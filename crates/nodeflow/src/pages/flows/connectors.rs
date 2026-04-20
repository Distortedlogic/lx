#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkflowConnector {
  pub id: String,
  pub label: String,
  pub description: String,
  pub category: String,
}

pub fn sample_workflow_connectors() -> Vec<WorkflowConnector> {
  vec![
    WorkflowConnector {
      id: "sample_pack".to_string(),
      label: "Sample Pack".to_string(),
      description: "Local development pack for the bundled newsfeed workflow.".to_string(),
      category: "starter".to_string(),
    },
    WorkflowConnector {
      id: "http".to_string(),
      label: "HTTP".to_string(),
      description: "Generic HTTP connector with expression-backed URLs and credential references.".to_string(),
      category: "connector".to_string(),
    },
    WorkflowConnector {
      id: "slack".to_string(),
      label: "Slack".to_string(),
      description: "Slack delivery connector using bot credentials.".to_string(),
      category: "connector".to_string(),
    },
  ]
}
