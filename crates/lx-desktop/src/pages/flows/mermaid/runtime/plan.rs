use std::collections::{BTreeMap, VecDeque};

use crate::runtime::types::DesktopAgentLaunchSpec;
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
) {
  for node_id in &plan.node_order {
    let Some(node) = plan.nodes.get(node_id) else {
      continue;
    };
    if registry.agent_for_flow_run_node(flow_run_id, node_id).is_some() {
      continue;
    }
    if node.dependencies.iter().any(|dependency| {
      registry
        .agent_for_flow_run_node(flow_run_id, dependency)
        .map(|agent| !matches!(agent.status, crate::runtime::types::DesktopAgentStatus::Completed))
        .unwrap_or(false)
    }) {
      continue;
    }
    if node.dependencies.iter().all(|dependency| registry.agent_for_flow_run_node(flow_run_id, dependency).is_some()) || node.dependencies.is_empty() {
      let mut spec = DesktopAgentLaunchSpec::new(node.display_label.clone(), node.task_summary.clone(), node.prompt.clone());
      spec.flow_node_id = Some(node.node_id.clone());
      spec.parent_id = node.dependencies.iter().find_map(|dependency| registry.agent_for_flow_run_node(flow_run_id, dependency).map(|agent| agent.id));
      runtime.launch_pi_agent_for_flow_run(flow_id.to_string(), flow_run_id.to_string(), &spec);
    }
  }
  let _ = chart;
}
