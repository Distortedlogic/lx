use lx_graph_editor::catalog::{GraphCredentialOption, GraphNodeTemplate};
use lx_graph_editor::model::GraphDocument;
use lx_graph_editor::protocol::GraphWidgetDiagnostic;

use super::registry::sample_workflow_registry;
use super::validation::validate_workflow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowProductKind {
  Workflow,
}

impl FlowProductKind {
  pub fn label(self) -> &'static str {
    "Workflow"
  }

  pub fn badge_label(self) -> &'static str {
    "n8n-style automation"
  }

  pub fn palette_title(self) -> &'static str {
    "Node Palette"
  }

  pub fn palette_description(self) -> &'static str {
    "Insert workflow steps into the graph. New nodes appear at the current viewport center."
  }

  pub fn empty_title(self) -> &'static str {
    "Canvas"
  }

  pub fn empty_message(self) -> &'static str {
    "Use the node palette to drop the first step into the graph. New nodes land in the current viewport center."
  }

  pub fn diagnostics_title(self) -> &'static str {
    "Validation"
  }

  pub fn diagnostics_description(self) -> &'static str {
    "Workflow-specific graph checks run after each mutation."
  }

  pub fn supports_runtime(self) -> bool {
    true
  }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FlowProductConfig {
  pub kind: FlowProductKind,
  pub templates: Vec<GraphNodeTemplate>,
  pub credential_options: Vec<GraphCredentialOption>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowCompileStatus {
  Ready,
  Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FlowCompileState {
  pub status: FlowCompileStatus,
  pub label: String,
  pub detail: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlowProductEvaluation {
  pub diagnostics: Vec<GraphWidgetDiagnostic>,
  pub compile_state: Option<FlowCompileState>,
}

pub fn resolve_flow_product(_document: &GraphDocument, _flow_id: &str) -> FlowProductConfig {
  let registry = sample_workflow_registry();
  FlowProductConfig { kind: FlowProductKind::Workflow, templates: registry.templates(), credential_options: registry.credential_options() }
}

pub fn evaluate_flow_document(_kind: FlowProductKind, document: &GraphDocument, templates: &[GraphNodeTemplate]) -> FlowProductEvaluation {
  FlowProductEvaluation { diagnostics: validate_workflow(document, templates), compile_state: None }
}

#[cfg(test)]
mod tests {
  use super::{FlowProductKind, resolve_flow_product};
  use crate::pages::flows::sample::{DEFAULT_FLOW_ID, sample_document};

  #[test]
  fn resolves_workflow_product_for_newsfeed_sample() {
    let document = sample_document(DEFAULT_FLOW_ID);
    let config = resolve_flow_product(&document, DEFAULT_FLOW_ID);

    assert_eq!(config.kind, FlowProductKind::Workflow);
    assert!(config.templates.iter().any(|template| template.id == "web_fetch"));
    assert!(!config.credential_options.is_empty());
  }
}
