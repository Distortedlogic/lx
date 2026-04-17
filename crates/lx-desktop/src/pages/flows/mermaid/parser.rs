use std::collections::{BTreeMap, BTreeSet};

use dioxus::prelude::*;
use mmdflux::builtins::default_registry;
use mmdflux::graph::{Arrow, Direction, Stroke};
use mmdflux::payload::Diagram;
use serde::Deserialize;

use super::types::{MermaidChart, MermaidDirection, MermaidEdge, MermaidNode, MermaidNodeMetadata, MermaidSemanticKind, MermaidSubgraph};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MermaidParseResult {
  pub chart: Option<MermaidChart>,
  pub diagnostics: Vec<lx_graph_editor::protocol::GraphWidgetDiagnostic>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FlowMetadata {
  title: Option<String>,
  notes: Option<String>,
}

pub fn parse_chart(flow_id: &str, source: &str) -> MermaidParseResult {
  let scan = scan_source(source);
  let mut diagnostics = scan.diagnostics;
  let chart = parse_with_mmdflux(flow_id, source, &scan, &mut diagnostics);
  MermaidParseResult { chart: if diagnostics.iter().any(|diagnostic| diagnostic.severity == lx_graph_editor::protocol::GraphWidgetDiagnosticSeverity::Error) { None } else { chart }, diagnostics }
}

fn parse_with_mmdflux(
  flow_id: &str,
  source: &str,
  scan: &ScanResult,
  diagnostics: &mut Vec<lx_graph_editor::protocol::GraphWidgetDiagnostic>,
) -> Option<MermaidChart> {
  let registry = default_registry();
  let resolved = match registry.resolve(source) {
    Some(resolved) => resolved,
    None => {
      diagnostics.push(error_diagnostic("mermaid-missing-diagram", "The Mermaid file does not contain a supported diagram header.", None));
      return None;
    },
  };
  if resolved.diagram_id() != "flowchart" {
    diagnostics.push(error_diagnostic("mermaid-non-flowchart", "Only Mermaid flowcharts are supported for this product mode.", None));
    return None;
  }
  let instance = match registry.create(resolved.diagram_id()) {
    Some(instance) => instance,
    None => {
      diagnostics.push(error_diagnostic("mermaid-flowchart-instance", "The Mermaid flowchart parser could not be constructed.", None));
      return None;
    },
  };
  let payload = match instance.parse(source).and_then(|parsed| parsed.into_payload()) {
    Ok(payload) => payload,
    Err(error) => {
      diagnostics.push(error_diagnostic("mermaid-parse-error", format!("Failed to parse Mermaid flowchart: {error}"), None));
      return None;
    },
  };
  let Diagram::Flowchart(graph) = payload else {
    diagnostics.push(error_diagnostic("mermaid-unexpected-payload", "Expected a Mermaid flowchart payload.", None));
    return None;
  };
  if !matches!(graph.direction, Direction::TopDown | Direction::LeftRight) {
    diagnostics.push(error_diagnostic("mermaid-direction", "Only flowchart TD and flowchart LR are supported.", None));
  }

  let mut referenced_node_ids = BTreeSet::new();
  let mut nodes: Vec<_> = graph
    .nodes
    .values()
    .map(|node| {
      referenced_node_ids.insert(node.id.clone());
      let semantic_kind = scan.semantic_classes.get(&node.id).copied().or_else(|| match node.shape {
        mmdflux::graph::Shape::Diamond => Some(MermaidSemanticKind::Decision),
        _ => None,
      });
      if semantic_kind.is_none() {
        diagnostics.push(error_diagnostic(
          format!("mermaid-node-class-{}", node.id),
          format!("Node `{}` is missing a supported semantic class assignment.", node.id),
          Some(lx_graph_editor::model::GraphEntityRef::Node(node.id.clone())),
        ));
      }
      MermaidNode {
        id: node.id.clone(),
        semantic_kind: semantic_kind.unwrap_or(MermaidSemanticKind::Step),
        display_label: node.label.clone(),
        subgraph_id: node.parent.clone(),
        metadata: scan.node_metadata.get(&node.id).cloned().unwrap_or_default(),
      }
    })
    .collect();
  nodes.sort_by(|left, right| left.id.cmp(&right.id));

  for node_id in scan.semantic_classes.keys() {
    if !referenced_node_ids.contains(node_id) {
      diagnostics.push(error_diagnostic(
        format!("mermaid-class-target-{node_id}"),
        format!("Class assignment referenced unknown node `{node_id}`."),
        None,
      ));
    }
  }
  for node_id in scan.node_metadata.keys() {
    if !referenced_node_ids.contains(node_id) {
      diagnostics.push(error_diagnostic(
        format!("mermaid-node-meta-{node_id}"),
        format!("Node metadata referenced unknown node `{node_id}`."),
        None,
      ));
    }
  }

  let edges = graph
    .edges
    .iter()
    .enumerate()
    .map(|(index, edge)| {
      if edge.from_subgraph.is_some() || edge.to_subgraph.is_some() || edge.stroke != Stroke::Solid || edge.arrow_start != Arrow::None || edge.arrow_end != Arrow::Normal {
        diagnostics.push(error_diagnostic(
          format!("mermaid-edge-style-{index}"),
          "Only simple directed node-to-node edges using `-->` are supported.".to_string(),
          None,
        ));
      }
      MermaidEdge { id: format!("edge-{}-{}-{index}", edge.from, edge.to), from: edge.from.clone(), to: edge.to.clone(), label: edge.label.clone() }
    })
    .collect();

  let mut subgraphs = graph
    .subgraph_order
    .iter()
    .filter_map(|subgraph_id| graph.subgraphs.get(subgraph_id))
    .map(|subgraph| {
      if subgraph.dir.is_some() {
        diagnostics.push(error_diagnostic(
          format!("mermaid-subgraph-direction-{}", subgraph.id),
          format!("Subgraph `{}` uses an unsupported direction override.", subgraph.id),
          None,
        ));
      }
      MermaidSubgraph { id: subgraph.id.clone(), title: subgraph.title.clone(), parent_id: subgraph.parent.clone() }
    })
    .collect::<Vec<_>>();
  subgraphs.sort_by(|left, right| left.id.cmp(&right.id));

  Some(MermaidChart {
    title: scan.flow_metadata.title.clone().unwrap_or_else(|| humanize_flow_id(flow_id)),
    notes: scan.flow_metadata.notes.clone(),
    direction: if graph.direction == Direction::LeftRight { MermaidDirection::LeftRight } else { MermaidDirection::TopDown },
    nodes,
    edges,
    subgraphs,
  })
}

#[derive(Default)]
struct ScanResult {
  diagnostics: Vec<lx_graph_editor::protocol::GraphWidgetDiagnostic>,
  flow_metadata: FlowMetadata,
  semantic_classes: BTreeMap<String, MermaidSemanticKind>,
  node_metadata: BTreeMap<String, MermaidNodeMetadata>,
}

fn scan_source(source: &str) -> ScanResult {
  let mut result = ScanResult::default();
  let mut header_seen = false;
  let mut subgraph_depth = 0usize;

  for (index, line) in source.lines().enumerate() {
    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }
    if let Some(metadata) = trimmed.strip_prefix("%% lx-flow:") {
      match serde_json::from_str::<FlowMetadata>(metadata.trim()) {
        Ok(flow_metadata) => result.flow_metadata = flow_metadata,
        Err(error) => result.diagnostics.push(error_diagnostic(format!("mermaid-flow-meta-{index}"), format!("Invalid flow metadata: {error}"), None)),
      }
      continue;
    }
    if let Some(metadata) = trimmed.strip_prefix("%% lx-node:") {
      let mut parts = metadata.trim().splitn(2, char::is_whitespace);
      let Some(node_id) = parts.next().filter(|value| !value.is_empty()) else {
        result.diagnostics.push(error_diagnostic(format!("mermaid-node-meta-{index}"), "Node metadata must start with a node id.".to_string(), None));
        continue;
      };
      let Some(json) = parts.next() else {
        result.diagnostics.push(error_diagnostic(format!("mermaid-node-meta-json-{index}"), format!("Node metadata for `{node_id}` is missing a JSON payload."), None));
        continue;
      };
      match serde_json::from_str::<MermaidNodeMetadata>(json.trim()) {
        Ok(node_metadata) => {
          result.node_metadata.insert(node_id.to_string(), node_metadata);
        },
        Err(error) => result.diagnostics.push(error_diagnostic(format!("mermaid-node-meta-json-{index}"), format!("Invalid node metadata for `{node_id}`: {error}"), None)),
      }
      continue;
    }
    if trimmed.starts_with("%%") {
      continue;
    }
    if !header_seen {
      header_seen = true;
      if !matches!(trimmed.to_ascii_lowercase().as_str(), "flowchart td" | "flowchart lr") {
        result.diagnostics.push(error_diagnostic("mermaid-header", "Only `flowchart TD` and `flowchart LR` headers are supported.".to_string(), None));
      }
      continue;
    }
    if let Some(class_names) = trimmed.strip_prefix("classDef ") {
      for class_name in class_names.split_whitespace().next().unwrap_or_default().split(',').map(str::trim).filter(|value| !value.is_empty()) {
        if MermaidSemanticKind::from_class_name(class_name).is_none() {
          result.diagnostics.push(error_diagnostic(format!("mermaid-classdef-{class_name}-{index}"), format!("Unsupported Mermaid semantic class `{class_name}`."), None));
        }
      }
      continue;
    }
    if let Some(rest) = trimmed.strip_prefix("class ") {
      let mut parts = rest.splitn(2, char::is_whitespace);
      let node_ids = parts.next().unwrap_or_default();
      let class_name = parts.next().unwrap_or_default().trim();
      let Some(kind) = MermaidSemanticKind::from_class_name(class_name) else {
        result.diagnostics.push(error_diagnostic(format!("mermaid-class-{index}"), format!("Unsupported Mermaid semantic class `{class_name}`."), None));
        continue;
      };
      for node_id in node_ids.split(',').map(str::trim).filter(|value| !value.is_empty()) {
        if let Some(existing) = result.semantic_classes.insert(node_id.to_string(), kind) && existing != kind {
          result.diagnostics.push(error_diagnostic(format!("mermaid-class-duplicate-{node_id}"), format!("Node `{node_id}` was assigned multiple semantic classes."), None));
        }
      }
      continue;
    }
    if trimmed.eq("end") {
      if subgraph_depth == 0 {
        result.diagnostics.push(error_diagnostic(format!("mermaid-end-{index}"), "Encountered `end` without an open subgraph.".to_string(), None));
      } else {
        subgraph_depth -= 1;
      }
      continue;
    }
    if trimmed.starts_with("subgraph ") {
      subgraph_depth += 1;
      continue;
    }
    if is_supported_edge_statement(trimmed) || is_supported_node_statement(trimmed) {
      continue;
    }
    result.diagnostics.push(error_diagnostic(format!("mermaid-statement-{index}"), format!("Unsupported Mermaid statement: `{trimmed}`"), None));
  }

  if !header_seen {
    result.diagnostics.push(error_diagnostic("mermaid-header-missing", "The Mermaid file is missing a `flowchart TD` or `flowchart LR` header.".to_string(), None));
  }
  if subgraph_depth != 0 {
    result.diagnostics.push(error_diagnostic("mermaid-subgraph-balance", "Subgraph blocks must be closed with `end`.".to_string(), None));
  }
  result
}

fn is_supported_edge_statement(line: &str) -> bool {
  let Some((left, right)) = line.split_once("-->") else {
    return false;
  };
  if left.contains("&") || right.contains("&") || left.contains("<") || right.contains("<") || line.matches("-->").count() != 1 {
    return false;
  }
  let right = if let Some(after_arrow) = right.trim().strip_prefix('|') {
    let Some((_, target)) = after_arrow.split_once('|') else {
      return false;
    };
    target.trim()
  } else {
    right.trim()
  };
  parse_endpoint_id(left.trim()).is_some() && parse_endpoint_id(right).is_some()
}

fn is_supported_node_statement(line: &str) -> bool {
  parse_endpoint_id(line).is_some()
}

fn parse_endpoint_id(value: &str) -> Option<&str> {
  let trimmed = value.trim();
  let end = trimmed.find(['[', '{', '(', '/']).unwrap_or(trimmed.len());
  let node_id = trimmed[..end].trim();
  if node_id.is_empty() || !node_id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')) {
    return None;
  }
  Some(node_id)
}

fn error_diagnostic(
  id: impl Into<String>,
  message: impl Into<String>,
  target: Option<lx_graph_editor::model::GraphEntityRef>,
) -> lx_graph_editor::protocol::GraphWidgetDiagnostic {
  lx_graph_editor::protocol::GraphWidgetDiagnostic {
    id: id.into(),
    severity: lx_graph_editor::protocol::GraphWidgetDiagnosticSeverity::Error,
    message: message.into(),
    source: Some("mermaid".to_string()),
    detail: None,
    target,
  }
}

fn humanize_flow_id(flow_id: &str) -> String {
  flow_id
    .split(['-', '_'])
    .filter(|segment| !segment.is_empty())
    .map(|segment| {
      let mut chars = segment.chars();
      match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}
