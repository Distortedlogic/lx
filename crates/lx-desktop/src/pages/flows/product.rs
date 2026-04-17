use lx_graph_editor::catalog::{GraphCredentialOption, GraphNodeTemplate};
use lx_graph_editor::model::GraphDocument;
use lx_graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

use crate::graph_editor::lowering::lower_lx_graph;
use crate::graph_editor::lx_semantics::LxNodeKind;

use super::registry::sample_workflow_registry;
use super::sample::is_lx_flow_id;
use super::validation::validate_workflow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowProductKind {
  Workflow,
  Lx,
}

impl FlowProductKind {
  pub fn label(self) -> &'static str {
    match self {
      Self::Workflow => "Workflow",
      Self::Lx => "LX Program",
    }
  }

  pub fn badge_label(self) -> &'static str {
    match self {
      Self::Workflow => "n8n-style automation",
      Self::Lx => "lx graphical programming",
    }
  }

  pub fn palette_title(self) -> &'static str {
    match self {
      Self::Workflow => "Node Palette",
      Self::Lx => "Program Palette",
    }
  }

  pub fn palette_description(self) -> &'static str {
    match self {
      Self::Workflow => "Insert workflow steps into the graph. New nodes appear at the current viewport center.",
      Self::Lx => "Insert lx semantic nodes into the graph. New nodes appear at the current viewport center.",
    }
  }

  pub fn empty_title(self) -> &'static str {
    match self {
      Self::Workflow => "Canvas",
      Self::Lx => "LX Program",
    }
  }

  pub fn empty_message(self) -> &'static str {
    match self {
      Self::Workflow => "Use the node palette to drop the first step into the graph. New nodes land in the current viewport center.",
      Self::Lx => "Use the program palette to add a goal, evidence, routing, and artifact path. New nodes land in the current viewport center.",
    }
  }

  pub fn diagnostics_title(self) -> &'static str {
    match self {
      Self::Workflow => "Validation",
      Self::Lx => "Compiler",
    }
  }

  pub fn diagnostics_description(self) -> &'static str {
    match self {
      Self::Workflow => "Workflow-specific graph checks run after each mutation.",
      Self::Lx => "LX semantic checks and lowering diagnostics run after each mutation.",
    }
  }

  pub fn supports_runtime(self) -> bool {
    matches!(self, Self::Workflow)
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

pub fn resolve_flow_product(document: &GraphDocument, flow_id: &str) -> FlowProductConfig {
  let kind = infer_flow_product_kind(document, flow_id);
  match kind {
    FlowProductKind::Workflow => {
      let registry = sample_workflow_registry();
      FlowProductConfig { kind, templates: registry.templates(), credential_options: registry.credential_options() }
    },
    FlowProductKind::Lx => FlowProductConfig { kind, templates: crate::graph_editor::lx_semantics::lx_node_templates(), credential_options: Vec::new() },
  }
}

pub fn evaluate_flow_document(kind: FlowProductKind, document: &GraphDocument, templates: &[GraphNodeTemplate]) -> FlowProductEvaluation {
  match kind {
    FlowProductKind::Workflow => FlowProductEvaluation { diagnostics: validate_workflow(document, templates), compile_state: None },
    FlowProductKind::Lx => {
      let outcome = lower_lx_graph(document);
      FlowProductEvaluation { compile_state: Some(lx_compile_state(&outcome)), diagnostics: outcome.diagnostics }
    },
  }
}

pub fn infer_flow_product_kind(document: &GraphDocument, flow_id: &str) -> FlowProductKind {
  if document.metadata.tags.iter().any(|tag| tag.eq_ignore_ascii_case("lx")) {
    return FlowProductKind::Lx;
  }
  if document.nodes.iter().any(|node| LxNodeKind::from_template_id(&node.template_id).is_some()) {
    return FlowProductKind::Lx;
  }
  if !document.nodes.is_empty() {
    return FlowProductKind::Workflow;
  }
  if is_lx_flow_id(flow_id) { FlowProductKind::Lx } else { FlowProductKind::Workflow }
}

fn lx_compile_state(outcome: &crate::graph_editor::lowering::LxLoweringOutcome) -> FlowCompileState {
  let error_count = outcome.diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error).count();
  let warning_count = outcome.diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Warning).count();

  if let Some(ir) = &outcome.ir {
    let label = if warning_count > 0 { "LX IR ready with warnings" } else { "LX IR ready" };
    let warning_suffix = if warning_count > 0 { format!(" {warning_count} warnings still need review.") } else { String::new() };
    FlowCompileState {
      status: FlowCompileStatus::Ready,
      label: label.to_string(),
      detail: format!("Lowered {} nodes and {} edges into lx IR in execution order.{}", ir.nodes.len(), ir.edges.len(), warning_suffix),
    }
  } else {
    FlowCompileState {
      status: FlowCompileStatus::Blocked,
      label: "LX compile blocked".to_string(),
      detail: format!("{error_count} errors and {warning_count} warnings are preventing lx lowering."),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{FlowCompileStatus, FlowProductKind, evaluate_flow_document, resolve_flow_product};
  use crate::pages::flows::sample::{DEFAULT_FLOW_ID, DEFAULT_LX_FLOW_ID, sample_document};

  #[test]
  fn resolves_workflow_product_for_newsfeed_sample() {
    let document = sample_document(DEFAULT_FLOW_ID);
    let config = resolve_flow_product(&document, DEFAULT_FLOW_ID);

    assert_eq!(config.kind, FlowProductKind::Workflow);
    assert!(config.templates.iter().any(|template| template.id == "web_fetch"));
    assert!(!config.credential_options.is_empty());
  }

  #[test]
  fn resolves_lx_product_for_lx_sample() {
    let document = sample_document(DEFAULT_LX_FLOW_ID);
    let config = resolve_flow_product(&document, DEFAULT_LX_FLOW_ID);

    assert_eq!(config.kind, FlowProductKind::Lx);
    assert!(config.templates.iter().any(|template| template.id == "lx_goal_input"));
    assert!(config.credential_options.is_empty());
  }

  #[test]
  fn lx_product_evaluation_reports_ready_compile_state() {
    let document = sample_document(DEFAULT_LX_FLOW_ID);
    let config = resolve_flow_product(&document, DEFAULT_LX_FLOW_ID);
    let evaluation = evaluate_flow_document(config.kind, &document, &config.templates);

    assert!(evaluation.diagnostics.is_empty());
    assert_eq!(evaluation.compile_state.expect("lx compile state").status, FlowCompileStatus::Ready);
  }
}
