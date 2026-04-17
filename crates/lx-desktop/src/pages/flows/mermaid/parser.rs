mod scan;

use std::collections::BTreeSet;

use mmdflux::builtins::default_registry;
use mmdflux::graph::{Arrow, Direction, Stroke};
use mmdflux::payload::Diagram;

use self::scan::{ScanResult, error_diagnostic, humanize_flow_id, scan_source};
use super::types::{MermaidChart, MermaidDirection, MermaidEdge, MermaidNode, MermaidSemanticKind, MermaidSubgraph};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MermaidParseResult {
  pub chart: Option<MermaidChart>,
  pub diagnostics: Vec<lx_graph_editor::protocol::GraphWidgetDiagnostic>,
}

pub fn parse_chart(flow_id: &str, source: &str) -> MermaidParseResult {
  let scan = scan_source(source);
  let mut diagnostics = scan.diagnostics.clone();
  let chart = parse_with_mmdflux(flow_id, source, &scan, &mut diagnostics);
  MermaidParseResult {
    chart: if diagnostics.iter().any(|diagnostic| diagnostic.severity == lx_graph_editor::protocol::GraphWidgetDiagnosticSeverity::Error) {
      None
    } else {
      chart
    },
    diagnostics,
  }
}

fn parse_with_mmdflux(
  flow_id: &str,
  source: &str,
  scan: &ScanResult,
  diagnostics: &mut Vec<lx_graph_editor::protocol::GraphWidgetDiagnostic>,
) -> Option<MermaidChart> {
  let registry = default_registry();
  let Some(resolved) = registry.resolve(source) else {
    diagnostics.push(error_diagnostic("mermaid-missing-diagram", "The Mermaid file does not contain a supported diagram header.", None));
    return None;
  };
  if resolved.diagram_id() != "flowchart" {
    diagnostics.push(error_diagnostic("mermaid-non-flowchart", "Only Mermaid flowcharts are supported for this product mode.", None));
    return None;
  }
  let Some(instance) = registry.create(resolved.diagram_id()) else {
    diagnostics.push(error_diagnostic("mermaid-flowchart-instance", "The Mermaid flowchart parser could not be constructed.", None));
    return None;
  };
  let parsed = match instance.parse(source) {
    Ok(parsed) => parsed,
    Err(error) => {
      diagnostics.push(error_diagnostic("mermaid-parse-error", format!("Failed to parse Mermaid flowchart: {error}"), None));
      return None;
    },
  };
  let payload = match parsed.into_payload() {
    Ok(payload) => payload,
    Err(error) => {
      diagnostics.push(error_diagnostic("mermaid-payload-error", format!("Failed to build Mermaid payload: {error}"), None));
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
      let semantic_kind = scan.semantic_classes.get(&node.id).copied().or(match node.shape {
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
      diagnostics.push(error_diagnostic(format!("mermaid-class-target-{node_id}"), format!("Class assignment referenced unknown node `{node_id}`."), None));
    }
  }
  for node_id in scan.node_metadata.keys() {
    if !referenced_node_ids.contains(node_id) {
      diagnostics.push(error_diagnostic(format!("mermaid-node-meta-{node_id}"), format!("Node metadata referenced unknown node `{node_id}`."), None));
    }
  }

  let edges = graph
    .edges
    .iter()
    .enumerate()
    .map(|(index, edge)| {
      if edge.from_subgraph.is_some()
        || edge.to_subgraph.is_some()
        || edge.stroke != Stroke::Solid
        || edge.arrow_start != Arrow::None
        || edge.arrow_end != Arrow::Normal
      {
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
    title: scan.flow_title.clone().unwrap_or_else(|| humanize_flow_id(flow_id)),
    notes: scan.flow_notes.clone(),
    direction: if graph.direction == Direction::LeftRight { MermaidDirection::LeftRight } else { MermaidDirection::TopDown },
    nodes,
    edges,
    subgraphs,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  const SAMPLE: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/flows/mermaid-mock-lx.mmd"));

  #[test]
  fn parses_supported_mermaid_sample() {
    let result = parse_chart("mermaid-mock-lx", SAMPLE);
    assert!(result.chart.is_some());
    assert!(result.diagnostics.is_empty());
  }

  #[test]
  fn rejects_unsupported_statements() {
    let result = parse_chart("bad", "flowchart TD\nclick A callback\nA[Start]\nclass A step");
    assert!(result.chart.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("Unsupported Mermaid statement")));
  }

  #[test]
  fn requires_supported_semantic_classes() {
    let result = parse_chart("bad", "flowchart TD\nA[Start]\nA --> B[End]");
    assert!(result.chart.is_none());
    assert!(result.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("missing a supported semantic class assignment")));
  }
}
