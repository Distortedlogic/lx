use std::collections::{HashMap, HashSet, VecDeque};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use lx_graph_editor::catalog::{PortDirection, node_template};
use lx_graph_editor::model::{GraphDocument, GraphEntityRef, GraphNode};
use lx_graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

use super::lx_semantics::{LxNodeKind, lx_node_templates};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LxGraphIr {
  pub graph_id: String,
  pub title: String,
  pub goal_node_id: String,
  pub node_order: Vec<String>,
  pub nodes: Vec<LxIrNode>,
  pub edges: Vec<LxIrEdge>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LxIrNode {
  pub node_id: String,
  pub kind: LxNodeKind,
  pub label: String,
  pub properties: Value,
  pub inputs: Vec<LxIrPortBinding>,
  pub outputs: Vec<LxIrPortBinding>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxIrPortBinding {
  pub port_id: String,
  #[serde(default)]
  pub edge_ids: Vec<String>,
  #[serde(default)]
  pub peer_node_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LxIrEdge {
  pub edge_id: String,
  pub from_node_id: String,
  pub from_port_id: String,
  pub to_node_id: String,
  pub to_port_id: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct LxLoweringOutcome {
  pub ir: Option<LxGraphIr>,
  #[serde(default)]
  pub diagnostics: Vec<GraphWidgetDiagnostic>,
}

pub fn lower_lx_graph(document: &GraphDocument) -> LxLoweringOutcome {
  let templates = lx_node_templates();
  let mut diagnostics = Vec::new();
  let mut node_kinds = HashMap::<String, LxNodeKind>::new();

  for node in &document.nodes {
    match LxNodeKind::from_template_id(&node.template_id) {
      Some(kind) => {
        node_kinds.insert(node.id.clone(), kind);
      },
      None => diagnostics.push(node_error(
        format!("lx-unknown-template-{}", node.id),
        format!("`{}` is not part of the lx graph registry.", node.template_id),
        Some("Use lx graph node templates when compiling into lx IR.".to_string()),
        &node.id,
      )),
    }
  }

  let mut incoming_edges = HashMap::<String, Vec<&lx_graph_editor::model::GraphEdge>>::new();
  let mut outgoing_edges = HashMap::<String, Vec<&lx_graph_editor::model::GraphEdge>>::new();

  for edge in &document.edges {
    let Some(_) = document.node(&edge.from.node_id) else {
      diagnostics.push(edge_error(
        format!("lx-missing-source-node-{}", edge.id),
        format!("Connection source node `{}` is missing.", edge.from.node_id),
        Some("Repair or remove this broken connection before compiling the lx graph.".to_string()),
        &edge.id,
      ));
      continue;
    };
    let Some(_) = document.node(&edge.to.node_id) else {
      diagnostics.push(edge_error(
        format!("lx-missing-target-node-{}", edge.id),
        format!("Connection target node `{}` is missing.", edge.to.node_id),
        Some("Repair or remove this broken connection before compiling the lx graph.".to_string()),
        &edge.id,
      ));
      continue;
    };
    incoming_edges.entry(edge.to.node_id.clone()).or_default().push(edge);
    outgoing_edges.entry(edge.from.node_id.clone()).or_default().push(edge);
  }

  let goal_nodes: Vec<_> = document.nodes.iter().filter(|node| node_kinds.get(&node.id) == Some(&LxNodeKind::GoalInput)).collect();
  let output_nodes: Vec<_> = document.nodes.iter().filter(|node| node_kinds.get(&node.id) == Some(&LxNodeKind::ArtifactOutput)).collect();

  if goal_nodes.is_empty() {
    diagnostics.push(graph_error(
      "lx-missing-goal".to_string(),
      "lx graphs require exactly one Goal Input node.".to_string(),
      Some("Add a Goal Input node to define the graph mission and success criteria.".to_string()),
    ));
  } else if goal_nodes.len() > 1 {
    for node in goal_nodes.iter().skip(1) {
      diagnostics.push(node_error(
        format!("lx-extra-goal-{}", node.id),
        "lx graphs require exactly one Goal Input node.".to_string(),
        Some("Merge competing goal frames into one root goal node.".to_string()),
        &node.id,
      ));
    }
  }

  if output_nodes.is_empty() {
    diagnostics.push(graph_error(
      "lx-missing-output".to_string(),
      "lx graphs need at least one Artifact Output node.".to_string(),
      Some("Add an Artifact Output node so compiled artifacts have a publication sink.".to_string()),
    ));
  }

  for node in &document.nodes {
    let Some(kind) = node_kinds.get(&node.id).copied() else {
      continue;
    };
    let Some(template) = node_template(&templates, &node.template_id) else {
      continue;
    };

    for port in template.ports.iter().filter(|port| port.direction == PortDirection::Input) {
      let sources = incoming_for_port(&incoming_edges, &node.id, &port.id);
      if port.required && sources.is_empty() {
        diagnostics.push(node_error(
          format!("lx-missing-input-{}-{}", node.id, port.id),
          format!("`{}` is missing the required `{}` input.", node.label.as_deref().unwrap_or(&node.id), port.label),
          Some("Wire the required upstream lx value into this input before compiling.".to_string()),
          &node.id,
        ));
      }
      if !port.allow_multiple && sources.len() > 1 {
        diagnostics.push(node_error(
          format!("lx-multi-input-{}-{}", node.id, port.id),
          format!("`{}` receives multiple connections on `{}`.", node.label.as_deref().unwrap_or(&node.id), port.label),
          Some("Keep a single upstream source for this input or change the semantic model.".to_string()),
          &node.id,
        ));
      }
    }

    match kind {
      LxNodeKind::SensemakingPass => {
        if outgoing_for_port(&outgoing_edges, &node.id, "artifact").is_empty() {
          diagnostics.push(node_warning(
            format!("lx-unconsumed-sensemaking-{}", node.id),
            format!("`{}` produces an artifact that nothing consumes.", node.label.as_deref().unwrap_or(&node.id)),
            Some("Connect the artifact output to a router or artifact output node.".to_string()),
            &node.id,
          ));
        }
      },
      LxNodeKind::DecisionRouter => {
        if outgoing_for_port(&outgoing_edges, &node.id, "actionable").is_empty() {
          diagnostics.push(node_warning(
            format!("lx-dropped-actionable-{}", node.id),
            format!("`{}` drops its actionable branch.", node.label.as_deref().unwrap_or(&node.id)),
            Some("Connect the actionable output to an Agent Task node.".to_string()),
            &node.id,
          ));
        }
        if outgoing_for_port(&outgoing_edges, &node.id, "archive").is_empty() {
          diagnostics.push(node_warning(
            format!("lx-dropped-archive-{}", node.id),
            format!("`{}` drops its archive branch.", node.label.as_deref().unwrap_or(&node.id)),
            Some("Connect the archive output to an Artifact Output node or another archival path.".to_string()),
            &node.id,
          ));
        }
      },
      LxNodeKind::AgentTask => {
        if outgoing_for_port(&outgoing_edges, &node.id, "artifact").is_empty() {
          diagnostics.push(node_warning(
            format!("lx-unpublished-task-{}", node.id),
            format!("`{}` produces an execution artifact that is never published.", node.label.as_deref().unwrap_or(&node.id)),
            Some("Connect the artifact output to an Artifact Output node or a downstream review stage.".to_string()),
            &node.id,
          ));
        }
      },
      LxNodeKind::GoalInput | LxNodeKind::EvidenceIngest | LxNodeKind::ArtifactOutput => {},
    }
  }

  for edge in &document.edges {
    let Some(source_kind) = node_kinds.get(&edge.from.node_id).copied() else {
      continue;
    };
    let Some(target_kind) = node_kinds.get(&edge.to.node_id).copied() else {
      continue;
    };

    if source_kind == LxNodeKind::DecisionRouter && edge.from.port_id == "actionable" && target_kind != LxNodeKind::AgentTask {
      diagnostics.push(edge_error(
        format!("lx-actionable-target-{}", edge.id),
        "Actionable packets must flow into an Agent Task node.".to_string(),
        Some("Route this edge into an Agent Task so actionable work becomes executable.".to_string()),
        &edge.id,
      ));
    }
  }

  if !goal_nodes.is_empty() {
    let reachable = reachable_from_roots(&goal_nodes, &outgoing_edges);
    for node in &document.nodes {
      if node_kinds.contains_key(&node.id) && !reachable.contains(&node.id) {
        diagnostics.push(node_warning(
          format!("lx-unreachable-node-{}", node.id),
          format!("`{}` is not reachable from the root goal.", node.label.as_deref().unwrap_or(&node.id)),
          Some("Connect this node into the goal-driven flow or remove it from the lx program.".to_string()),
          &node.id,
        ));
      }
    }
  }

  let node_order = topological_order(document, &node_kinds);
  if node_order.len() != node_kinds.len() {
    diagnostics.push(graph_error(
      "lx-cycle-detected".to_string(),
      "The lx graph contains a cycle and cannot be lowered into execution order.".to_string(),
      Some("Remove or reroute the back-edge so the graph stays acyclic.".to_string()),
    ));
  }

  let has_errors = diagnostics.iter().any(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error);
  if has_errors {
    return LxLoweringOutcome { ir: None, diagnostics };
  }

  let goal_node_id = goal_nodes.first().map(|node| node.id.clone()).or_else(|| node_order.first().cloned()).unwrap_or_default();
  let nodes = node_order
    .iter()
    .filter_map(|node_id| {
      let node = document.node(node_id)?;
      let kind = node_kinds.get(node_id).copied()?;
      let template = node_template(&templates, &node.template_id)?;
      Some(LxIrNode {
        node_id: node.id.clone(),
        kind,
        label: node.label.clone().unwrap_or_else(|| node.id.clone()),
        properties: node.properties.clone(),
        inputs: template
          .ports
          .iter()
          .filter(|port| port.direction == PortDirection::Input)
          .map(|port| port_binding_from_incoming(&incoming_edges, node, &port.id))
          .collect(),
        outputs: template
          .ports
          .iter()
          .filter(|port| port.direction == PortDirection::Output)
          .map(|port| port_binding_from_outgoing(&outgoing_edges, node, &port.id))
          .collect(),
      })
    })
    .collect();
  let edges = document
    .edges
    .iter()
    .filter(|edge| node_kinds.contains_key(&edge.from.node_id) && node_kinds.contains_key(&edge.to.node_id))
    .map(|edge| LxIrEdge {
      edge_id: edge.id.clone(),
      from_node_id: edge.from.node_id.clone(),
      from_port_id: edge.from.port_id.clone(),
      to_node_id: edge.to.node_id.clone(),
      to_port_id: edge.to.port_id.clone(),
    })
    .collect();

  LxLoweringOutcome {
    ir: Some(LxGraphIr { graph_id: document.id.clone(), title: document.title.clone(), goal_node_id, node_order, nodes, edges }),
    diagnostics,
  }
}

fn incoming_for_port<'a>(
  incoming_edges: &'a HashMap<String, Vec<&'a lx_graph_editor::model::GraphEdge>>,
  node_id: &str,
  port_id: &str,
) -> Vec<&'a lx_graph_editor::model::GraphEdge> {
  incoming_edges.get(node_id).into_iter().flatten().copied().filter(|edge| edge.to.port_id == port_id).collect()
}

fn outgoing_for_port<'a>(
  outgoing_edges: &'a HashMap<String, Vec<&'a lx_graph_editor::model::GraphEdge>>,
  node_id: &str,
  port_id: &str,
) -> Vec<&'a lx_graph_editor::model::GraphEdge> {
  outgoing_edges.get(node_id).into_iter().flatten().copied().filter(|edge| edge.from.port_id == port_id).collect()
}

fn reachable_from_roots<'a>(roots: &[&'a GraphNode], outgoing_edges: &HashMap<String, Vec<&lx_graph_editor::model::GraphEdge>>) -> HashSet<String> {
  let mut reachable = HashSet::new();
  let mut queue = VecDeque::from_iter(roots.iter().map(|node| node.id.clone()));

  while let Some(node_id) = queue.pop_front() {
    if !reachable.insert(node_id.clone()) {
      continue;
    }
    if let Some(edges) = outgoing_edges.get(&node_id) {
      for edge in edges {
        if !reachable.contains(&edge.to.node_id) {
          queue.push_back(edge.to.node_id.clone());
        }
      }
    }
  }

  reachable
}

fn topological_order(document: &GraphDocument, node_kinds: &HashMap<String, LxNodeKind>) -> Vec<String> {
  let mut indegree = HashMap::<String, usize>::new();
  let mut outgoing = HashMap::<String, Vec<String>>::new();

  for node in &document.nodes {
    if !node_kinds.contains_key(&node.id) {
      continue;
    }
    indegree.insert(node.id.clone(), 0);
    outgoing.insert(node.id.clone(), Vec::new());
  }

  for edge in &document.edges {
    if !node_kinds.contains_key(&edge.from.node_id) || !node_kinds.contains_key(&edge.to.node_id) {
      continue;
    }
    outgoing.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    *indegree.entry(edge.to.node_id.clone()).or_default() += 1;
  }

  let mut queue = VecDeque::new();
  for node in &document.nodes {
    if indegree.get(&node.id) == Some(&0) {
      queue.push_back(node.id.clone());
    }
  }

  let mut ordered = Vec::new();
  while let Some(node_id) = queue.pop_front() {
    if !node_kinds.contains_key(&node_id) || ordered.iter().any(|current| current == &node_id) {
      continue;
    }
    ordered.push(node_id.clone());
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

  ordered
}

fn port_binding_from_incoming(incoming_edges: &HashMap<String, Vec<&lx_graph_editor::model::GraphEdge>>, node: &GraphNode, port_id: &str) -> LxIrPortBinding {
  let edges = incoming_for_port(incoming_edges, &node.id, port_id);
  LxIrPortBinding {
    port_id: port_id.to_string(),
    edge_ids: edges.iter().map(|edge| edge.id.clone()).collect(),
    peer_node_ids: edges.iter().map(|edge| edge.from.node_id.clone()).collect(),
  }
}

fn port_binding_from_outgoing(outgoing_edges: &HashMap<String, Vec<&lx_graph_editor::model::GraphEdge>>, node: &GraphNode, port_id: &str) -> LxIrPortBinding {
  let edges = outgoing_for_port(outgoing_edges, &node.id, port_id);
  LxIrPortBinding {
    port_id: port_id.to_string(),
    edge_ids: edges.iter().map(|edge| edge.id.clone()).collect(),
    peer_node_ids: edges.iter().map(|edge| edge.to.node_id.clone()).collect(),
  }
}

fn graph_error(id: String, message: String, detail: Option<String>) -> GraphWidgetDiagnostic {
  GraphWidgetDiagnostic { id, severity: GraphWidgetDiagnosticSeverity::Error, message, source: Some("lx compiler".to_string()), detail, target: None }
}

fn node_error(id: String, message: String, detail: Option<String>, node_id: &str) -> GraphWidgetDiagnostic {
  GraphWidgetDiagnostic {
    id,
    severity: GraphWidgetDiagnosticSeverity::Error,
    message,
    source: Some("lx compiler".to_string()),
    detail,
    target: Some(GraphEntityRef::Node(node_id.to_string())),
  }
}

fn edge_error(id: String, message: String, detail: Option<String>, edge_id: &str) -> GraphWidgetDiagnostic {
  GraphWidgetDiagnostic {
    id,
    severity: GraphWidgetDiagnosticSeverity::Error,
    message,
    source: Some("lx compiler".to_string()),
    detail,
    target: Some(GraphEntityRef::Edge(edge_id.to_string())),
  }
}

fn node_warning(id: String, message: String, detail: Option<String>, node_id: &str) -> GraphWidgetDiagnostic {
  GraphWidgetDiagnostic {
    id,
    severity: GraphWidgetDiagnosticSeverity::Warning,
    message,
    source: Some("lx compiler".to_string()),
    detail,
    target: Some(GraphEntityRef::Node(node_id.to_string())),
  }
}

#[cfg(test)]
mod tests {
  use lx_graph_editor::commands::{GraphCommand, apply_graph_command};
  use lx_graph_editor::model::{GraphDocument, GraphPoint, GraphPortRef};

  use super::lower_lx_graph;
  use crate::graph_editor::lx_semantics::{LxNodeKind, lx_node_templates};

  #[test]
  fn lowers_valid_lx_graph_into_ir() {
    let document = valid_lx_graph();
    let outcome = lower_lx_graph(&document);

    assert!(outcome.diagnostics.is_empty());
    let ir = outcome.ir.expect("valid lx graph should lower");
    assert_eq!(ir.goal_node_id, "goal");
    assert_eq!(ir.node_order, vec!["goal", "evidence", "sense", "out"]);
    assert_eq!(ir.edges.len(), 4);
    assert_eq!(ir.nodes[2].node_id, "sense");
    assert_eq!(ir.nodes[2].inputs[0].peer_node_ids, vec!["goal"]);
    assert_eq!(ir.nodes[2].inputs[1].peer_node_ids, vec!["evidence"]);
  }

  #[test]
  fn rejects_multiple_goal_inputs() {
    let templates = lx_node_templates();
    let mut document = valid_lx_graph();
    apply_graph_command(
      &mut document,
      &templates,
      GraphCommand::AddNode {
        node_id: "goal-2".to_string(),
        template_id: LxNodeKind::GoalInput.template_id().to_string(),
        position: GraphPoint { x: 0.0, y: 240.0 },
        label: None,
      },
    )
    .expect("add second goal");

    let outcome = lower_lx_graph(&document);

    assert!(outcome.ir.is_none());
    assert!(outcome.diagnostics.iter().any(|diagnostic| diagnostic.id == "lx-extra-goal-goal-2"));
  }

  #[test]
  fn rejects_actionable_edges_that_skip_agent_tasks() {
    let templates = lx_node_templates();
    let mut document = GraphDocument::new("lx-graph", "LX Graph");

    add_node(&mut document, &templates, "goal", LxNodeKind::GoalInput, 0.0, 0.0);
    add_node(&mut document, &templates, "evidence", LxNodeKind::EvidenceIngest, 220.0, 160.0);
    add_node(&mut document, &templates, "sense", LxNodeKind::SensemakingPass, 460.0, 80.0);
    add_node(&mut document, &templates, "router", LxNodeKind::DecisionRouter, 700.0, 80.0);
    add_node(&mut document, &templates, "out", LxNodeKind::ArtifactOutput, 940.0, 80.0);

    connect(&mut document, &templates, "goal-evidence", "goal", "goal", "evidence", "goal");
    connect(&mut document, &templates, "goal-sense", "goal", "goal", "sense", "goal");
    connect(&mut document, &templates, "evidence-sense", "evidence", "evidence", "sense", "evidence");
    connect(&mut document, &templates, "sense-router", "sense", "artifact", "router", "artifact");
    connect(&mut document, &templates, "router-out", "router", "actionable", "out", "artifact");

    let outcome = lower_lx_graph(&document);

    assert!(outcome.ir.is_none());
    assert!(outcome.diagnostics.iter().any(|diagnostic| diagnostic.id == "lx-actionable-target-router-out"));
  }

  #[test]
  fn warns_when_router_drops_a_branch() {
    let templates = lx_node_templates();
    let mut document = GraphDocument::new("lx-graph", "LX Graph");

    add_node(&mut document, &templates, "goal", LxNodeKind::GoalInput, 0.0, 0.0);
    add_node(&mut document, &templates, "evidence", LxNodeKind::EvidenceIngest, 220.0, 160.0);
    add_node(&mut document, &templates, "sense", LxNodeKind::SensemakingPass, 460.0, 80.0);
    add_node(&mut document, &templates, "router", LxNodeKind::DecisionRouter, 700.0, 80.0);
    add_node(&mut document, &templates, "task", LxNodeKind::AgentTask, 940.0, 40.0);
    add_node(&mut document, &templates, "out", LxNodeKind::ArtifactOutput, 1180.0, 40.0);

    connect(&mut document, &templates, "goal-evidence", "goal", "goal", "evidence", "goal");
    connect(&mut document, &templates, "goal-sense", "goal", "goal", "sense", "goal");
    connect(&mut document, &templates, "evidence-sense", "evidence", "evidence", "sense", "evidence");
    connect(&mut document, &templates, "sense-router", "sense", "artifact", "router", "artifact");
    connect(&mut document, &templates, "router-task", "router", "actionable", "task", "actionable");
    connect(&mut document, &templates, "task-out", "task", "artifact", "out", "artifact");

    let outcome = lower_lx_graph(&document);

    assert!(outcome.ir.is_some());
    assert!(outcome.diagnostics.iter().any(|diagnostic| diagnostic.id == "lx-dropped-archive-router"));
    assert!(outcome.diagnostics.iter().all(|diagnostic| diagnostic.severity != lx_graph_editor::protocol::GraphWidgetDiagnosticSeverity::Error));
  }

  fn valid_lx_graph() -> GraphDocument {
    let templates = lx_node_templates();
    let mut document = GraphDocument::new("lx-graph", "LX Graph");

    add_node(&mut document, &templates, "goal", LxNodeKind::GoalInput, 0.0, 0.0);
    add_node(&mut document, &templates, "evidence", LxNodeKind::EvidenceIngest, 220.0, 160.0);
    add_node(&mut document, &templates, "sense", LxNodeKind::SensemakingPass, 460.0, 80.0);
    add_node(&mut document, &templates, "out", LxNodeKind::ArtifactOutput, 720.0, 80.0);

    connect(&mut document, &templates, "goal-evidence", "goal", "goal", "evidence", "goal");
    connect(&mut document, &templates, "goal-sense", "goal", "goal", "sense", "goal");
    connect(&mut document, &templates, "evidence-sense", "evidence", "evidence", "sense", "evidence");
    connect(&mut document, &templates, "sense-out", "sense", "artifact", "out", "artifact");

    document
  }

  fn add_node(document: &mut GraphDocument, templates: &[lx_graph_editor::catalog::GraphNodeTemplate], node_id: &str, kind: LxNodeKind, x: f64, y: f64) {
    apply_graph_command(
      document,
      templates,
      GraphCommand::AddNode { node_id: node_id.to_string(), template_id: kind.template_id().to_string(), position: GraphPoint { x, y }, label: None },
    )
    .expect("add lx node");
  }

  fn connect(
    document: &mut GraphDocument,
    templates: &[lx_graph_editor::catalog::GraphNodeTemplate],
    edge_id: &str,
    from_node: &str,
    from_port: &str,
    to_node: &str,
    to_port: &str,
  ) {
    apply_graph_command(
      document,
      templates,
      GraphCommand::ConnectPorts {
        edge_id: edge_id.to_string(),
        from: GraphPortRef { node_id: from_node.to_string(), port_id: from_port.to_string() },
        to: GraphPortRef { node_id: to_node.to_string(), port_id: to_port.to_string() },
        label: None,
      },
    )
    .expect("connect lx graph");
  }
}
