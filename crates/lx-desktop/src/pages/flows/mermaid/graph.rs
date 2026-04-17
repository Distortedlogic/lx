use lx_graph_editor::catalog::{GraphFieldCapabilities, GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate, GraphPortType, PortDirection};
use lx_graph_editor::model::{GraphDocument, GraphEdge, GraphNode, GraphPoint, GraphPortRef};
use serde_json::json;

use super::types::{MermaidChart, MermaidDirection, MermaidEdge, MermaidNode, MermaidNodeMetadata, MermaidSemanticKind};

const DISPLAY_LABEL_FIELD: &str = "display_label";
const TASK_SUMMARY_FIELD: &str = "task_summary";
const PROMPT_FIELD: &str = "prompt";

pub fn mermaid_templates() -> Vec<GraphNodeTemplate> {
  [
    MermaidSemanticKind::Step,
    MermaidSemanticKind::Agent,
    MermaidSemanticKind::Decision,
    MermaidSemanticKind::Tool,
    MermaidSemanticKind::Io,
  ]
  .into_iter()
  .map(template_for_kind)
  .collect()
}

pub fn chart_graph_document(flow_id: &str, chart: &MermaidChart) -> GraphDocument {
  let mut document = GraphDocument::new(flow_id, chart.title.clone());
  document.metadata.notes = chart.notes.clone();
  document.metadata.tags.push("mermaid".to_string());
  document.metadata.tags.push(format!("mermaid-direction:{}", chart.direction.keyword().to_lowercase()));
  document.nodes = chart
    .nodes
    .iter()
    .enumerate()
    .map(|(index, node)| GraphNode {
      id: node.id.clone(),
      template_id: node.semantic_kind.template_id().to_string(),
      label: Some(node.display_label.clone()),
      metadata: Default::default(),
      position: GraphPoint { x: 160.0 + (index % 4) as f64 * 320.0, y: 140.0 + (index / 4) as f64 * 220.0 },
      properties: json!({
        DISPLAY_LABEL_FIELD: node.display_label,
        TASK_SUMMARY_FIELD: node.metadata.task_summary.clone().unwrap_or_default(),
        PROMPT_FIELD: node.metadata.prompt.clone().unwrap_or_default(),
      }),
    })
    .collect();
  document.edges = chart
    .edges
    .iter()
    .map(|edge| GraphEdge {
      id: edge.id.clone(),
      label: edge.label.clone(),
      metadata: Default::default(),
      from: GraphPortRef { node_id: edge.from.clone(), port_id: "out".to_string() },
      to: GraphPortRef { node_id: edge.to.clone(), port_id: "in".to_string() },
    })
    .collect();
  document
}

pub fn chart_from_graph_document(document: &GraphDocument, previous_chart: Option<&MermaidChart>) -> MermaidChart {
  let direction = previous_chart
    .map(|chart| chart.direction)
    .or_else(|| {
      document.metadata.tags.iter().find_map(|tag| match tag.as_str() {
        "mermaid-direction:td" => Some(MermaidDirection::TopDown),
        "mermaid-direction:lr" => Some(MermaidDirection::LeftRight),
        _ => None,
      })
    })
    .unwrap_or_default();

  let nodes = document
    .nodes
    .iter()
    .filter_map(|node| {
      let semantic_kind = MermaidSemanticKind::from_template_id(&node.template_id)?;
      let display_label = field_text(node, DISPLAY_LABEL_FIELD).filter(|value| !value.is_empty()).or_else(|| node.label.clone()).unwrap_or_else(|| node.id.clone());
      let previous_node = previous_chart.and_then(|chart| chart.node(&node.id));
      Some(MermaidNode {
        id: node.id.clone(),
        semantic_kind,
        display_label,
        subgraph_id: previous_node.and_then(|entry| entry.subgraph_id.clone()),
        metadata: MermaidNodeMetadata {
          task_summary: normalize_optional_field(node, TASK_SUMMARY_FIELD).or_else(|| previous_node.and_then(|entry| entry.metadata.task_summary.clone())),
          prompt: normalize_optional_field(node, PROMPT_FIELD).or_else(|| previous_node.and_then(|entry| entry.metadata.prompt.clone())),
        },
      })
    })
    .collect::<Vec<_>>();

  let node_ids: std::collections::BTreeSet<_> = nodes.iter().map(|node| node.id.as_str()).collect();
  let edges = document
    .edges
    .iter()
    .enumerate()
    .map(|(index, edge)| MermaidEdge {
      id: if edge.id.is_empty() { edge_id(&edge.from.node_id, &edge.to.node_id, index) } else { edge.id.clone() },
      from: edge.from.node_id.clone(),
      to: edge.to.node_id.clone(),
      label: edge.label.clone(),
    })
    .collect::<Vec<_>>();

  let subgraphs = previous_chart
    .map(|chart| {
      chart
        .subgraphs
        .iter()
        .filter(|subgraph| nodes.iter().any(|node| node.subgraph_id.as_deref() == Some(subgraph.id.as_str())) || chart_has_child_subgraph(chart, &subgraph.id, &node_ids))
        .cloned()
        .collect()
    })
    .unwrap_or_default();

  MermaidChart { title: document.title.clone(), notes: document.metadata.notes.clone(), direction, nodes, edges, subgraphs }
}

fn template_for_kind(kind: MermaidSemanticKind) -> GraphNodeTemplate {
  GraphNodeTemplate {
    id: kind.template_id().to_string(),
    label: template_label(kind).to_string(),
    description: Some(template_description(kind).to_string()),
    category: Some("mermaid".to_string()),
    default_label: Some(template_default_label(kind).to_string()),
    ports: vec![
      GraphPortTemplate {
        id: "in".to_string(),
        label: "In".to_string(),
        description: Some("Incoming Mermaid control flow".to_string()),
        direction: PortDirection::Input,
        data_type: Some(GraphPortType::new("mermaid", "flow")),
        required: false,
        allow_multiple: true,
      },
      GraphPortTemplate {
        id: "out".to_string(),
        label: "Out".to_string(),
        description: Some("Outgoing Mermaid control flow".to_string()),
        direction: PortDirection::Output,
        data_type: Some(GraphPortType::new("mermaid", "flow")),
        required: false,
        allow_multiple: true,
      },
    ],
    fields: vec![
      GraphFieldSchema {
        id: DISPLAY_LABEL_FIELD.to_string(),
        label: "Display Label".to_string(),
        description: Some("Visible Mermaid node label".to_string()),
        kind: GraphFieldKind::Text,
        required: true,
        default_value: Some(json!(template_default_label(kind))),
        capabilities: GraphFieldCapabilities::default(),
      },
      GraphFieldSchema {
        id: TASK_SUMMARY_FIELD.to_string(),
        label: "Task Summary".to_string(),
        description: Some("Runtime summary used when shaping a mock lx task".to_string()),
        kind: GraphFieldKind::TextArea,
        required: false,
        default_value: Some(json!("")),
        capabilities: GraphFieldCapabilities::default(),
      },
      GraphFieldSchema {
        id: PROMPT_FIELD.to_string(),
        label: "Prompt".to_string(),
        description: Some("Additional runtime prompt instructions for this node".to_string()),
        kind: GraphFieldKind::TextArea,
        required: false,
        default_value: Some(json!("")),
        capabilities: GraphFieldCapabilities::default(),
      },
    ],
  }
}

fn field_text(node: &GraphNode, field_id: &str) -> Option<String> {
  node.properties.as_object()?.get(field_id)?.as_str().map(ToOwned::to_owned)
}

fn normalize_optional_field(node: &GraphNode, field_id: &str) -> Option<String> {
  let value = field_text(node, field_id)?;
  let trimmed = value.trim();
  if trimmed.is_empty() { None } else { Some(trimmed.to_string()) }
}

fn edge_id(from: &str, to: &str, index: usize) -> String {
  format!("edge-{from}-{to}-{index}")
}

fn chart_has_child_subgraph(chart: &MermaidChart, subgraph_id: &str, node_ids: &std::collections::BTreeSet<&str>) -> bool {
  chart.subgraphs.iter().any(|subgraph| {
    subgraph.parent_id.as_deref() == Some(subgraph_id)
      && (chart.nodes.iter().any(|node| node.subgraph_id.as_deref() == Some(subgraph.id.as_str()) && node_ids.contains(node.id.as_str()))
        || chart_has_child_subgraph(chart, &subgraph.id, node_ids))
  })
}

fn template_label(kind: MermaidSemanticKind) -> &'static str {
  match kind {
    MermaidSemanticKind::Step => "Step",
    MermaidSemanticKind::Agent => "Agent",
    MermaidSemanticKind::Decision => "Decision",
    MermaidSemanticKind::Tool => "Tool",
    MermaidSemanticKind::Io => "I/O",
  }
}

fn template_default_label(kind: MermaidSemanticKind) -> &'static str {
  match kind {
    MermaidSemanticKind::Step => "Step",
    MermaidSemanticKind::Agent => "Agent",
    MermaidSemanticKind::Decision => "Decision",
    MermaidSemanticKind::Tool => "Tool",
    MermaidSemanticKind::Io => "I/O",
  }
}

fn template_description(kind: MermaidSemanticKind) -> &'static str {
  match kind {
    MermaidSemanticKind::Step => "Generic Mermaid program step.",
    MermaidSemanticKind::Agent => "Pi-backed mock agent node.",
    MermaidSemanticKind::Decision => "Branching decision node.",
    MermaidSemanticKind::Tool => "Tool activity node.",
    MermaidSemanticKind::Io => "Input or output boundary node.",
  }
}
