use lx_graph_editor::catalog::{GraphCredentialOption, GraphNodeTemplate};
use lx_graph_editor::model::GraphDocument;
use lx_graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

use crate::graph_editor::lowering::lower_lx_graph;
use crate::graph_editor::lx_semantics::LxNodeKind;

use super::mermaid::{build_execution_plan, chart_from_graph_document, mermaid_templates};
use super::registry::sample_workflow_registry;
use super::sample::{is_lx_flow_id, is_mermaid_flow_id};
use super::validation::validate_workflow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlowProductKind {
  Workflow,
  Lx,
  Mermaid,
}

impl FlowProductKind {
  pub fn label(self) -> &'static str {
    match self {
      Self::Workflow => "Workflow",
      Self::Lx => "LX Program",
      Self::Mermaid => "Mermaid Program",
    }
  }

  pub fn badge_label(self) -> &'static str {
    match self {
      Self::Workflow => "n8n-style automation",
      Self::Lx => "lx graphical programming",
      Self::Mermaid => "mermaid mock lx runtime",
    }
  }

  pub fn palette_title(self) -> &'static str {
    match self {
      Self::Workflow => "Node Palette",
      Self::Lx => "Program Palette",
      Self::Mermaid => "Mermaid Palette",
    }
  }

  pub fn palette_description(self) -> &'static str {
    match self {
      Self::Workflow => "Insert workflow steps into the graph. New nodes appear at the current viewport center.",
      Self::Lx => "Insert lx semantic nodes into the graph. New nodes appear at the current viewport center.",
      Self::Mermaid => "Insert Mermaid mock-lx nodes into the graph. New nodes appear at the current viewport center.",
    }
  }

  pub fn empty_title(self) -> &'static str {
    match self {
      Self::Workflow => "Canvas",
      Self::Lx => "LX Program",
      Self::Mermaid => "Mermaid Program",
    }
  }

  pub fn empty_message(self) -> &'static str {
    match self {
      Self::Workflow => "Use the node palette to drop the first step into the graph. New nodes land in the current viewport center.",
      Self::Lx => "Use the program palette to add a goal, evidence, routing, and artifact path. New nodes land in the current viewport center.",
      Self::Mermaid => "Use the Mermaid palette to add mock-lx steps, agents, decisions, tools, and I/O boundaries.",
    }
  }

  pub fn diagnostics_title(self) -> &'static str {
    match self {
      Self::Workflow => "Validation",
      Self::Lx => "Compiler",
      Self::Mermaid => "Mermaid Validation",
    }
  }

  pub fn diagnostics_description(self) -> &'static str {
    match self {
      Self::Workflow => "Workflow-specific graph checks run after each mutation.",
      Self::Lx => "LX semantic checks and lowering diagnostics run after each mutation.",
      Self::Mermaid => "Mermaid roundtrip and execution-plan checks run after each mutation.",
    }
  }

  pub fn supports_runtime(self) -> bool {
    matches!(self, Self::Workflow | Self::Mermaid)
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
    FlowProductKind::Mermaid => FlowProductConfig { kind, templates: mermaid_templates(), credential_options: Vec::new() },
  }
}

pub fn evaluate_flow_document(kind: FlowProductKind, document: &GraphDocument, templates: &[GraphNodeTemplate]) -> FlowProductEvaluation {
  match kind {
    FlowProductKind::Workflow => FlowProductEvaluation { diagnostics: validate_workflow(document, templates), compile_state: None },
    FlowProductKind::Lx => {
      let outcome = lower_lx_graph(document);
      FlowProductEvaluation { compile_state: Some(lx_compile_state(&outcome)), diagnostics: outcome.diagnostics }
    },
    FlowProductKind::Mermaid => mermaid_evaluation(document),
  }
}

pub fn infer_flow_product_kind(document: &GraphDocument, flow_id: &str) -> FlowProductKind {
  if document.metadata.tags.iter().any(|tag| tag.eq_ignore_ascii_case("mermaid")) {
    return FlowProductKind::Mermaid;
  }
  if document.metadata.tags.iter().any(|tag| tag.eq_ignore_ascii_case("lx")) {
    return FlowProductKind::Lx;
  }
  if document.nodes.iter().any(|node| node.template_id.starts_with("mermaid_")) {
    return FlowProductKind::Mermaid;
  }
  if document.nodes.iter().any(|node| LxNodeKind::from_template_id(&node.template_id).is_some()) {
    return FlowProductKind::Lx;
  }
  if !document.nodes.is_empty() {
    return FlowProductKind::Workflow;
  }
  if is_mermaid_flow_id(flow_id) {
    FlowProductKind::Mermaid
  } else if is_lx_flow_id(flow_id) {
    FlowProductKind::Lx
  } else {
    FlowProductKind::Workflow
  }
}

fn mermaid_evaluation(document: &GraphDocument) -> FlowProductEvaluation {
  let chart = chart_from_graph_document(document);
  let mut diagnostics = Vec::new();
  for node in &chart.nodes {
    if let Some(subgraph_id) = node.subgraph_id.as_deref()
      && chart.subgraph(subgraph_id).is_none()
    {
      diagnostics.push(GraphWidgetDiagnostic {
        id: format!("mermaid-subgraph-node-{}", node.id),
        severity: GraphWidgetDiagnosticSeverity::Error,
        message: format!("Node `{}` references missing subgraph `{subgraph_id}`.", node.id),
        source: Some("mermaid".to_string()),
        detail: None,
        target: Some(lx_graph_editor::model::GraphEntityRef::Node(node.id.clone())),
      });
    }
  }
  let compile_state = match build_execution_plan(&chart) {
    Ok(plan) => Some(FlowCompileState {
      status: FlowCompileStatus::Ready,
      label: "Mermaid run plan ready".to_string(),
      detail: format!("Planned {} mock-lx runtime steps in DAG order.", plan.node_order.len()),
    }),
    Err(error) => {
      diagnostics.push(GraphWidgetDiagnostic {
        id: "mermaid-plan".to_string(),
        severity: GraphWidgetDiagnosticSeverity::Error,
        message: error.clone(),
        source: Some("mermaid".to_string()),
        detail: None,
        target: None,
      });
      Some(FlowCompileState { status: FlowCompileStatus::Blocked, label: "Mermaid run plan blocked".to_string(), detail: error })
    },
  };
  FlowProductEvaluation { diagnostics, compile_state }
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
  use crate::pages::flows::sample::{DEFAULT_FLOW_ID, DEFAULT_LX_FLOW_ID, DEFAULT_MERMAID_FLOW_ID, sample_document};

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

  #[test]
  fn resolves_mermaid_product_for_mermaid_sample() {
    let document = sample_document(DEFAULT_MERMAID_FLOW_ID);
    let config = resolve_flow_product(&document, DEFAULT_MERMAID_FLOW_ID);

    assert_eq!(config.kind, FlowProductKind::Mermaid);
    assert!(config.templates.iter().any(|template| template.id == "mermaid_agent"));
  }

  #[test]
  fn mermaid_product_evaluation_reports_ready_run_plan() {
    let document = sample_document(DEFAULT_MERMAID_FLOW_ID);
    let config = resolve_flow_product(&document, DEFAULT_MERMAID_FLOW_ID);
    let evaluation = evaluate_flow_document(config.kind, &document, &config.templates);

    assert!(evaluation.diagnostics.is_empty());
    assert_eq!(evaluation.compile_state.expect("mermaid compile state").status, FlowCompileStatus::Ready);
  }
}
