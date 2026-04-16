use lx_graph_editor::catalog::{GraphCredentialOption, GraphNodeTemplate};

use super::catalog::{connector_node_templates, sample_workflow_pack_templates};
use super::connectors::{WorkflowConnector, sample_workflow_connectors};
use super::credentials::{WorkflowCredentialStore, sample_workflow_credentials};

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowNodeDescriptor {
  pub connector_id: String,
  pub template: GraphNodeTemplate,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorkflowNodeRegistry {
  pub connectors: Vec<WorkflowConnector>,
  pub nodes: Vec<WorkflowNodeDescriptor>,
  pub credentials: WorkflowCredentialStore,
}

impl WorkflowNodeRegistry {
  pub fn templates(&self) -> Vec<GraphNodeTemplate> {
    self.nodes.iter().map(|entry| entry.template.clone()).collect()
  }

  pub fn credential_options(&self) -> Vec<GraphCredentialOption> {
    self.credentials.graph_options()
  }
}

pub fn sample_workflow_registry() -> WorkflowNodeRegistry {
  let mut nodes = sample_workflow_pack_templates()
    .into_iter()
    .map(|template| WorkflowNodeDescriptor { connector_id: "sample_pack".to_string(), template })
    .collect::<Vec<_>>();

  nodes.extend(connector_node_templates().into_iter().map(|template| WorkflowNodeDescriptor {
    connector_id: match template.id.as_str() {
      "http_request" => "http".to_string(),
      "slack_post" => "slack".to_string(),
      _ => "sample_pack".to_string(),
    },
    template,
  }));

  WorkflowNodeRegistry { connectors: sample_workflow_connectors(), nodes, credentials: sample_workflow_credentials() }
}

#[cfg(test)]
mod tests {
  use super::sample_workflow_registry;

  #[test]
  fn registry_exposes_connector_templates_and_credentials() {
    let registry = sample_workflow_registry();
    let template_ids = registry.templates().into_iter().map(|template| template.id).collect::<Vec<_>>();
    let credential_ids = registry.credential_options().into_iter().map(|option| option.id).collect::<Vec<_>>();

    assert!(template_ids.iter().any(|id| id == "http_request"));
    assert!(template_ids.iter().any(|id| id == "slack_post"));
    assert!(credential_ids.iter().any(|id| id == "cred-http-news"));
    assert!(credential_ids.iter().any(|id| id == "cred-slack-briefing"));
  }
}
