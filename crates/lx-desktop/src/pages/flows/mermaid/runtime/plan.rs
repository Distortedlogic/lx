use std::collections::{BTreeMap, VecDeque};

use crate::runtime::types::{DesktopAgentLaunchSpec, DesktopAgentStatus, DesktopRuntimeEventKind, payload_text};
use crate::runtime::{DesktopRuntimeController, DesktopRuntimeRegistry};

use super::super::types::{MermaidChart, MermaidSemanticKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MermaidExecutionNode {
  pub node_id: String,
  pub semantic_kind: MermaidSemanticKind,
  pub display_label: String,
  pub task_summary: String,
  pub prompt: String,
  pub dependencies: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MermaidExecutionPlan {
  pub node_order: Vec<String>,
  pub nodes: BTreeMap<String, MermaidExecutionNode>,
}

pub fn build_execution_plan(chart: &MermaidChart) -> Result<MermaidExecutionPlan, String> {
  let mut dependencies = BTreeMap::<String, Vec<String>>::new();
  for node in &chart.nodes {
    dependencies.entry(node.id.clone()).or_default();
  }
  for edge in &chart.edges {
    dependencies.entry(edge.to.clone()).or_default().push(edge.from.clone());
    dependencies.entry(edge.from.clone()).or_default();
  }

  let mut indegree = dependencies.iter().map(|(node_id, deps)| (node_id.clone(), deps.len())).collect::<BTreeMap<_, _>>();
  let mut ready = indegree.iter().filter(|(_, count)| **count == 0).map(|(node_id, _)| node_id.clone()).collect::<VecDeque<_>>();
  let mut order = Vec::new();
  while let Some(node_id) = ready.pop_front() {
    order.push(node_id.clone());
    for edge in chart.edges.iter().filter(|edge| edge.from == node_id) {
      if let Some(count) = indegree.get_mut(&edge.to) {
        *count = count.saturating_sub(1);
        if *count == 0 {
          ready.push_back(edge.to.clone());
        }
      }
    }
  }
  if order.len() != dependencies.len() {
    return Err("Mermaid execution requires an acyclic graph.".to_string());
  }

  Ok(MermaidExecutionPlan {
    node_order: order,
    nodes: chart
      .nodes
      .iter()
      .map(|node| {
        (
          node.id.clone(),
          MermaidExecutionNode {
            node_id: node.id.clone(),
            semantic_kind: node.semantic_kind,
            display_label: node.display_label.clone(),
            task_summary: node.metadata.task_summary.clone().filter(|value| !value.trim().is_empty()).unwrap_or_else(|| node.display_label.clone()),
            prompt: node.metadata.prompt.clone().filter(|value| !value.trim().is_empty()).unwrap_or_else(|| node.display_label.clone()),
            dependencies: dependencies.get(&node.id).cloned().unwrap_or_default(),
          },
        )
      })
      .collect(),
  })
}

pub fn launch_ready_nodes(
  runtime: &DesktopRuntimeController,
  registry: &DesktopRuntimeRegistry,
  chart: &MermaidChart,
  plan: &MermaidExecutionPlan,
  flow_id: &str,
  flow_run_id: &str,
) -> Vec<String> {
  let mut launched = Vec::new();
  for node_id in ready_nodes_to_launch(plan, registry, flow_run_id) {
    let Some(node) = plan.nodes.get(&node_id) else {
      continue;
    };
    let mut spec = DesktopAgentLaunchSpec::new(node.display_label.clone(), node.task_summary.clone(), shape_prompt(chart, plan, registry, flow_run_id, node));
    spec.flow_node_id = Some(node.node_id.clone());
    spec.parent_id = parent_agent_id(plan, registry, flow_run_id, &node.dependencies);
    launched.push(runtime.launch_pi_agent_for_flow_run(flow_id.to_string(), flow_run_id, &spec));
  }
  launched
}

fn ready_nodes_to_launch(plan: &MermaidExecutionPlan, registry: &DesktopRuntimeRegistry, flow_run_id: &str) -> Vec<String> {
  plan
    .node_order
    .iter()
    .filter(|node_id| {
      let Some(node) = plan.nodes.get(*node_id) else {
        return false;
      };
      registry.agent_for_flow_run_node(flow_run_id, node_id).is_none()
        && !has_blocking_dependency(registry, flow_run_id, node)
        && (node.dependencies.is_empty() || node.dependencies.iter().all(|dependency| registry.agent_for_flow_run_node(flow_run_id, dependency).is_some()))
    })
    .cloned()
    .collect()
}

fn has_blocking_dependency(registry: &DesktopRuntimeRegistry, flow_run_id: &str, node: &MermaidExecutionNode) -> bool {
  node.dependencies.iter().any(|dependency| {
    registry
      .agent_for_flow_run_node(flow_run_id, dependency)
      .map(|agent| matches!(agent.status, DesktopAgentStatus::Error | DesktopAgentStatus::Aborted))
      .unwrap_or(false)
  })
}

fn shape_prompt(
  chart: &MermaidChart,
  plan: &MermaidExecutionPlan,
  registry: &DesktopRuntimeRegistry,
  flow_run_id: &str,
  node: &MermaidExecutionNode,
) -> String {
  let predecessor_labels = ordered_dependencies(plan, &node.dependencies)
    .into_iter()
    .filter_map(|dependency| plan.nodes.get(&dependency).map(|entry| entry.display_label.clone()))
    .collect::<Vec<_>>();
  let predecessor_summaries = ordered_dependencies(plan, &node.dependencies)
    .into_iter()
    .filter_map(|dependency| {
      let predecessor = plan.nodes.get(&dependency)?;
      let agent = registry.agent_for_flow_run_node(flow_run_id, &dependency)?;
      let summary = predecessor_outcome_summary(registry, &agent.id)?;
      Some(format!("- {}: {}", predecessor.display_label, summary))
    })
    .collect::<Vec<_>>();

  let mut prompt = vec![
    format!("Flow chart: {}", chart.title),
    format!("Node: {}", node.display_label),
    format!("Semantic kind: {}", node.semantic_kind.class_name()),
    format!("Task summary: {}", node.task_summary),
    format!("Node instructions: {}", node.prompt),
  ];
  if !predecessor_labels.is_empty() {
    prompt.push(format!("Direct predecessors: {}", predecessor_labels.join(", ")));
  }
  if !predecessor_summaries.is_empty() {
    prompt.push("Predecessor outcomes:".to_string());
    prompt.extend(predecessor_summaries);
  }
  prompt.push("Work only on this node and report the concrete output for downstream steps.".to_string());
  prompt.join("\n")
}

fn parent_agent_id(plan: &MermaidExecutionPlan, registry: &DesktopRuntimeRegistry, flow_run_id: &str, dependencies: &[String]) -> Option<String> {
  ordered_dependencies(plan, dependencies).into_iter().find_map(|dependency| registry.agent_for_flow_run_node(flow_run_id, &dependency).map(|agent| agent.id))
}

fn ordered_dependencies(plan: &MermaidExecutionPlan, dependencies: &[String]) -> Vec<String> {
  let rank = plan.node_order.iter().enumerate().map(|(index, node_id)| (node_id.clone(), index)).collect::<BTreeMap<_, _>>();
  let mut ordered = dependencies.to_vec();
  ordered.sort_by_key(|dependency| rank.get(dependency).copied().unwrap_or(usize::MAX));
  ordered
}

fn predecessor_outcome_summary(registry: &DesktopRuntimeRegistry, agent_id: &str) -> Option<String> {
  let events = registry.events_for_agent(agent_id);
  events
    .iter()
    .rev()
    .find(|event| event.kind == DesktopRuntimeEventKind::MessageComplete && event.payload.get("role").and_then(serde_json::Value::as_str) == Some("assistant"))
    .and_then(|event| payload_text(&event.payload))
    .or_else(|| events.iter().rev().find(|event| event.kind == DesktopRuntimeEventKind::ToolResult).and_then(|event| payload_text(&event.payload)))
    .or_else(|| {
      events
        .iter()
        .rev()
        .find(|event| matches!(event.kind, DesktopRuntimeEventKind::ToolError | DesktopRuntimeEventKind::BackendError))
        .and_then(|event| payload_text(&event.payload))
    })
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::runtime::types::{DesktopAgentLaunchSpec, DesktopAgentRuntime};

  fn sample_chart() -> MermaidChart {
    MermaidChart {
      title: "Chart".to_string(),
      notes: None,
      direction: super::super::super::types::MermaidDirection::TopDown,
      nodes: vec![
        super::super::super::types::MermaidNode {
          id: "a".to_string(),
          semantic_kind: MermaidSemanticKind::Agent,
          display_label: "A".to_string(),
          subgraph_id: None,
          metadata: super::super::super::types::MermaidNodeMetadata::default(),
        },
        super::super::super::types::MermaidNode {
          id: "b".to_string(),
          semantic_kind: MermaidSemanticKind::Agent,
          display_label: "B".to_string(),
          subgraph_id: None,
          metadata: super::super::super::types::MermaidNodeMetadata::default(),
        },
        super::super::super::types::MermaidNode {
          id: "c".to_string(),
          semantic_kind: MermaidSemanticKind::Agent,
          display_label: "C".to_string(),
          subgraph_id: None,
          metadata: super::super::super::types::MermaidNodeMetadata::default(),
        },
      ],
      edges: vec![
        super::super::super::types::MermaidEdge { id: "e1".to_string(), from: "a".to_string(), to: "c".to_string(), label: None },
        super::super::super::types::MermaidEdge { id: "e2".to_string(), from: "b".to_string(), to: "c".to_string(), label: None },
      ],
      subgraphs: Vec::new(),
    }
  }

  #[test]
  fn plan_generation_is_topological() {
    let plan = build_execution_plan(&sample_chart()).expect("plan should build");
    assert_eq!(plan.node_order, vec!["a".to_string(), "b".to_string(), "c".to_string()]);
  }

  #[test]
  fn ready_nodes_wait_for_dependencies() {
    let registry = DesktopRuntimeRegistry::new();
    let plan = build_execution_plan(&sample_chart()).expect("plan should build");
    assert_eq!(ready_nodes_to_launch(&plan, &registry, "run-1"), vec!["a".to_string(), "b".to_string()]);
  }

  #[test]
  fn parent_selection_uses_first_predecessor_in_plan_order() {
    let registry = DesktopRuntimeRegistry::new();
    let mut alpha = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("A", "a", "a"));
    alpha.id = "agent-a".to_string();
    alpha.flow_run_id = Some("run-1".to_string());
    alpha.flow_node_id = Some("a".to_string());
    alpha.status = DesktopAgentStatus::Completed;
    registry.register_agent(alpha);
    let mut beta = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("B", "b", "b"));
    beta.id = "agent-b".to_string();
    beta.flow_run_id = Some("run-1".to_string());
    beta.flow_node_id = Some("b".to_string());
    beta.status = DesktopAgentStatus::Completed;
    registry.register_agent(beta);
    let plan = build_execution_plan(&sample_chart()).expect("plan should build");
    assert_eq!(parent_agent_id(&plan, &registry, "run-1", &["b".to_string(), "a".to_string()]), Some("agent-a".to_string()));
  }
}
