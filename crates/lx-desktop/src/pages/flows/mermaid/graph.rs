use lx_graph_editor::catalog::{GraphFieldCapabilities, GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, GraphPortTemplate, GraphPortType, PortDirection};
use lx_graph_editor::model::{GraphDocument, GraphEdge, GraphNode, GraphPoint, GraphPortRef};
use serde_json::json;

use super::types::{MermaidChart, MermaidDirection, MermaidEdge, MermaidNode, MermaidNodeMetadata, MermaidSemanticKind};

const DISPLAY_LABEL_FIELD: &str = "display_label";
const SUBGRAPH_ID_FIELD: &str = "subgraph_id";
const TASK_SUMMARY_FIELD: &str = "task_summary";
const PROMPT_FIELD: &str = "prompt";
const MERMAID_TAG: &str = "mermaid";
const MERMAID_DIRECTION_PREFIX: &str = "mermaid-direction:";
const MERMAID_SUBGRAPH_PREFIX: &str = "mermaid-subgraph:";

pub fn mermaid_templates() -> Vec<GraphNodeTemplate> {
  [MermaidSemanticKind::Step, MermaidSemanticKind::Agent, MermaidSemanticKind::Decision, MermaidSemanticKind::Tool, MermaidSemanticKind::Io]
    .into_iter()
    .map(template_for_kind)
    .collect()
}

pub fn chart_graph_document(flow_id: &str, chart: &MermaidChart) -> GraphDocument {
  let mut document = GraphDocument::new(flow_id, chart.title.clone());
  document.metadata.notes = chart.notes.clone();
  document.metadata.tags.push(MERMAID_TAG.to_string());
  document.metadata.tags.push(format!("{MERMAID_DIRECTION_PREFIX}{}", chart.direction.keyword().to_lowercase()));
  document.metadata.tags.extend(
    chart
      .subgraphs
      .iter()
      .map(|subgraph| format!("{MERMAID_SUBGRAPH_PREFIX}{}", serde_json::to_string(subgraph).expect("mermaid subgraph metadata should serialize"))),
  );
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
        SUBGRAPH_ID_FIELD: node.subgraph_id.clone().unwrap_or_default(),
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

pub fn chart_from_graph_document(document: &GraphDocument) -> MermaidChart {
  let direction = document
    .metadata
    .tags
    .iter()
    .find_map(|tag| match tag.as_str() {
      "mermaid-direction:td" => Some(MermaidDirection::TopDown),
      "mermaid-direction:lr" => Some(MermaidDirection::LeftRight),
      _ => None,
    })
    .unwrap_or_default();

  let subgraphs = mermaid_subgraphs(document);
  let nodes = document
    .nodes
    .iter()
    .filter_map(|node| {
      let semantic_kind = MermaidSemanticKind::from_template_id(&node.template_id)?;
      let display_label =
        field_text(node, DISPLAY_LABEL_FIELD).filter(|value| !value.is_empty()).or_else(|| node.label.clone()).unwrap_or_else(|| node.id.clone());
      Some(MermaidNode {
        id: node.id.clone(),
        semantic_kind,
        display_label,
        subgraph_id: normalize_optional_field(node, SUBGRAPH_ID_FIELD),
        metadata: MermaidNodeMetadata {
          task_summary: normalize_optional_field(node, TASK_SUMMARY_FIELD),
          prompt: normalize_optional_field(node, PROMPT_FIELD),
        },
      })
    })
    .collect::<Vec<_>>();

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
        id: SUBGRAPH_ID_FIELD.to_string(),
        label: "Subgraph Id".to_string(),
        description: Some("Optional Mermaid subgraph membership".to_string()),
        kind: GraphFieldKind::Text,
        required: false,
        default_value: Some(json!("")),
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

fn mermaid_subgraphs(document: &GraphDocument) -> Vec<super::types::MermaidSubgraph> {
  let mut subgraphs = document
    .metadata
    .tags
    .iter()
    .filter_map(|tag| tag.strip_prefix(MERMAID_SUBGRAPH_PREFIX))
    .filter_map(|payload| serde_json::from_str(payload).ok())
    .collect::<Vec<_>>();
  subgraphs.sort_by(|left: &super::types::MermaidSubgraph, right| left.id.cmp(&right.id));
  subgraphs
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn mermaid_graph_roundtrip_preserves_subgraphs_and_edge_labels() {
    let chart = MermaidChart {
      title: "Chart".to_string(),
      notes: Some("notes".to_string()),
      direction: MermaidDirection::LeftRight,
      nodes: vec![
        MermaidNode {
          id: "a".to_string(),
          semantic_kind: MermaidSemanticKind::Agent,
          display_label: "Agent".to_string(),
          subgraph_id: Some("sg-1".to_string()),
          metadata: MermaidNodeMetadata { task_summary: Some("summary".to_string()), prompt: Some("prompt".to_string()) },
        },
        MermaidNode {
          id: "b".to_string(),
          semantic_kind: MermaidSemanticKind::Io,
          display_label: "Output".to_string(),
          subgraph_id: None,
          metadata: MermaidNodeMetadata::default(),
        },
      ],
      edges: vec![MermaidEdge { id: "edge-a-b".to_string(), from: "a".to_string(), to: "b".to_string(), label: Some("next".to_string()) }],
      subgraphs: vec![super::super::types::MermaidSubgraph { id: "sg-1".to_string(), title: "Group".to_string(), parent_id: None }],
    };

    let document = chart_graph_document("chart", &chart);
    let roundtrip = chart_from_graph_document(&document);

    assert_eq!(roundtrip.direction, MermaidDirection::LeftRight);
    assert_eq!(roundtrip.subgraphs, chart.subgraphs);
    assert_eq!(roundtrip.edges[0].label.as_deref(), Some("next"));
    assert_eq!(roundtrip.node("a").and_then(|node| node.subgraph_id.as_deref()), Some("sg-1"));
  }
}
