use std::collections::{HashMap, HashSet, VecDeque};

use serde_json::Value;

use crate::graph_editor::catalog::{GraphFieldKind, GraphNodeTemplate, PortDirection, node_template, port_template};
use crate::graph_editor::model::{GraphDocument, GraphEntityRef};
use crate::graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

pub fn validate_workflow(document: &GraphDocument, templates: &[GraphNodeTemplate]) -> Vec<GraphWidgetDiagnostic> {
  let mut diagnostics = Vec::new();
  validate_duplicate_node_ids(document, &mut diagnostics);
  validate_required_fields(document, templates, &mut diagnostics);
  validate_required_inputs(document, templates, &mut diagnostics);
  validate_edges(document, templates, &mut diagnostics);
  validate_cycles(document, &mut diagnostics);
  diagnostics
}

fn validate_duplicate_node_ids(document: &GraphDocument, diagnostics: &mut Vec<GraphWidgetDiagnostic>) {
  let mut seen = HashSet::new();
  for node in &document.nodes {
    if seen.insert(node.id.clone()) {
      continue;
    }
    diagnostics.push(node_error(format!("duplicate-node-{}", node.id), format!("Duplicate node id `{}`.", node.id), &node.id));
  }
}

fn validate_required_fields(document: &GraphDocument, templates: &[GraphNodeTemplate], diagnostics: &mut Vec<GraphWidgetDiagnostic>) {
  for node in &document.nodes {
    let Some(template) = node_template(templates, &node.template_id) else {
      diagnostics.push(node_error(
        format!("unknown-template-{}", node.id),
        format!("Node `{}` references unknown template `{}`.", node.id, node.template_id),
        &node.id,
      ));
      continue;
    };
    let properties = node.properties.as_object();
    for field in &template.fields {
      if !field.required {
        continue;
      }
      let value = properties.and_then(|entry| entry.get(&field.id));
      if is_missing_required_value(value, &field.kind) {
        diagnostics.push(node_error(
          format!("missing-field-{}-{}", node.id, field.id),
          format!("`{}` is required for {}.", field.label, node.label.as_deref().unwrap_or(&node.id)),
          &node.id,
        ));
      }
    }
  }
}

fn validate_required_inputs(document: &GraphDocument, templates: &[GraphNodeTemplate], diagnostics: &mut Vec<GraphWidgetDiagnostic>) {
  for node in &document.nodes {
    let Some(template) = node_template(templates, &node.template_id) else {
      continue;
    };
    for port in template.ports.iter().filter(|port| port.direction == PortDirection::Input && port.required) {
      let has_connection = document.edges.iter().any(|edge| edge.to.node_id == node.id && edge.to.port_id == port.id);
      if has_connection {
        continue;
      }
      diagnostics.push(node_error(format!("missing-input-{}-{}", node.id, port.id), format!("Required input `{}` is not connected.", port.label), &node.id));
    }
  }
}

fn validate_edges(document: &GraphDocument, templates: &[GraphNodeTemplate], diagnostics: &mut Vec<GraphWidgetDiagnostic>) {
  for edge in &document.edges {
    let Some(from_node) = document.nodes.iter().find(|node| node.id == edge.from.node_id) else {
      diagnostics.push(edge_error(
        format!("missing-edge-from-node-{}", edge.id),
        format!("Edge `{}` references missing source node `{}`.", edge.id, edge.from.node_id),
        &edge.id,
      ));
      continue;
    };
    let Some(to_node) = document.nodes.iter().find(|node| node.id == edge.to.node_id) else {
      diagnostics.push(edge_error(
        format!("missing-edge-to-node-{}", edge.id),
        format!("Edge `{}` references missing target node `{}`.", edge.id, edge.to.node_id),
        &edge.id,
      ));
      continue;
    };

    let Some(from_port) = port_template(templates, &from_node.template_id, &edge.from.port_id) else {
      diagnostics.push(edge_error(
        format!("missing-edge-from-port-{}", edge.id),
        format!("Source port `{}` no longer exists on `{}`.", edge.from.port_id, from_node.id),
        &edge.id,
      ));
      continue;
    };
    let Some(to_port) = port_template(templates, &to_node.template_id, &edge.to.port_id) else {
      diagnostics.push(edge_error(
        format!("missing-edge-to-port-{}", edge.id),
        format!("Target port `{}` no longer exists on `{}`.", edge.to.port_id, to_node.id),
        &edge.id,
      ));
      continue;
    };

    if from_port.direction != PortDirection::Output || to_port.direction != PortDirection::Input {
      diagnostics.push(edge_error(
        format!("invalid-edge-direction-{}", edge.id),
        "Connections must flow from an output port into an input port.".to_string(),
        &edge.id,
      ));
    }

    if let (Some(from_type), Some(to_type)) = (&from_port.data_type, &to_port.data_type)
      && from_type != to_type
    {
      diagnostics.push(edge_error(
        format!("incompatible-edge-types-{}", edge.id),
        format!("`{from_type}` does not match `{to_type}` on this connection."),
        &edge.id,
      ));
    }
  }
}

fn validate_cycles(document: &GraphDocument, diagnostics: &mut Vec<GraphWidgetDiagnostic>) {
  let node_ids: HashSet<String> = document.nodes.iter().map(|node| node.id.clone()).collect();
  let mut indegree = HashMap::<String, usize>::new();
  let mut outgoing = HashMap::<String, Vec<String>>::new();

  for node_id in &node_ids {
    indegree.insert(node_id.clone(), 0);
    outgoing.insert(node_id.clone(), Vec::new());
  }

  for edge in &document.edges {
    if !node_ids.contains(&edge.from.node_id) || !node_ids.contains(&edge.to.node_id) {
      continue;
    }
    outgoing.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    *indegree.entry(edge.to.node_id.clone()).or_default() += 1;
  }

  let mut queue = VecDeque::from_iter(indegree.iter().filter(|(_, count)| **count == 0).map(|(node_id, _)| node_id.clone()));
  let mut visited = 0usize;

  while let Some(node_id) = queue.pop_front() {
    visited += 1;
    if let Some(targets) = outgoing.get(&node_id) {
      for target in targets {
        if let Some(count) = indegree.get_mut(target) {
          *count -= 1;
          if *count == 0 {
            queue.push_back(target.clone());
          }
        }
      }
    }
  }

  if visited == node_ids.len() {
    return;
  }

  for (node_id, count) in indegree {
    if count == 0 {
      continue;
    }
    diagnostics.push(node_error(
      format!("cycle-node-{node_id}"),
      "This node participates in a cycle. Workflow execution should remain acyclic.".to_string(),
      &node_id,
    ));
  }
}

fn is_missing_required_value(value: Option<&Value>, kind: &GraphFieldKind) -> bool {
  let Some(value) = value else {
    return true;
  };
  if value.is_null() {
    return true;
  }
  match kind {
    GraphFieldKind::Text | GraphFieldKind::TextArea | GraphFieldKind::Select { .. } => value.as_str().is_none_or(str::is_empty),
    GraphFieldKind::StringList => value.as_array().is_none_or(|items| items.is_empty()),
    GraphFieldKind::Number | GraphFieldKind::Integer => !value.is_number(),
    GraphFieldKind::Boolean => !value.is_boolean(),
  }
}

fn node_error(id: String, message: String, node_id: &str) -> GraphWidgetDiagnostic {
  GraphWidgetDiagnostic { id, severity: GraphWidgetDiagnosticSeverity::Error, message, target: Some(GraphEntityRef::Node(node_id.to_string())) }
}

fn edge_error(id: String, message: String, edge_id: &str) -> GraphWidgetDiagnostic {
  GraphWidgetDiagnostic { id, severity: GraphWidgetDiagnosticSeverity::Error, message, target: Some(GraphEntityRef::Edge(edge_id.to_string())) }
}

#[cfg(test)]
mod tests {
  use super::validate_workflow;
  use crate::pages::flows::catalog::workflow_node_templates;
  use crate::pages::flows::sample::sample_document;

  #[test]
  fn sample_flow_is_initially_valid() {
    let document = sample_document("newsfeed-research");
    let diagnostics = validate_workflow(&document, &workflow_node_templates());
    assert!(diagnostics.is_empty());
  }

  #[test]
  fn detects_cycles_and_missing_required_inputs() {
    let templates = workflow_node_templates();
    let mut document = sample_document("newsfeed-research");
    document.edges.retain(|edge| edge.id != "edge-sources-fetch");
    document.edges.push(crate::graph_editor::model::GraphEdge {
      id: "edge-score-fetch".to_string(),
      label: None,
      metadata: Default::default(),
      from: crate::graph_editor::model::GraphPortRef { node_id: "score".to_string(), port_id: "ranked".to_string() },
      to: crate::graph_editor::model::GraphPortRef { node_id: "fetch".to_string(), port_id: "topics".to_string() },
    });

    let diagnostics = validate_workflow(&document, &templates);
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.id.starts_with("cycle-node-")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.id == "missing-input-fetch-sources"));
  }
}
