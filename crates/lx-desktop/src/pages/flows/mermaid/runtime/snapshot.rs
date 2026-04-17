use lx_graph_editor::protocol::{GraphEdgeRunState, GraphNodeRunState, GraphRunSnapshot, GraphRunStatus};

use crate::runtime::DesktopRuntimeRegistry;
use crate::runtime::events::transcript_rows;
use crate::runtime::types::{DesktopAgentStatus, DesktopRuntimeEventKind};

use super::plan::MermaidExecutionPlan;
use super::super::types::MermaidChart;

pub fn build_run_snapshot(chart: &MermaidChart, plan: &MermaidExecutionPlan, registry: &DesktopRuntimeRegistry, flow_run_id: &str) -> GraphRunSnapshot {
  let node_states = chart
    .nodes
    .iter()
    .map(|node| {
      let agent = registry.agent_for_flow_run_node(flow_run_id, &node.id);
      let (status, detail, output_summary) = match agent {
        Some(agent) => {
          let events = registry.events_for_agent(&agent.id);
          let transcript = transcript_rows(&events);
          let output_summary = transcript.iter().rev().find(|row| row.role == "assistant").map(|row| row.text.clone());
          let detail = output_summary.clone().or_else(|| transcript.iter().rev().find(|row| row.role == "error").map(|row| row.text.clone()));
          (agent_status(agent.status), detail, output_summary)
        },
        None => blocked_or_pending_detail(plan, registry, flow_run_id, &node.id),
      };
      GraphNodeRunState { node_id: node.id.clone(), status, label: Some(node.display_label.clone()), detail, output_summary, started_at: None, finished_at: None, duration_ms: None }
    })
    .collect::<Vec<_>>();
  let edge_states = chart
    .edges
    .iter()
    .map(|edge| GraphEdgeRunState { edge_id: edge.id.clone(), status: edge_status(&node_states, &edge.from, &edge.to), label: edge.label.clone(), detail: None })
    .collect::<Vec<_>>();
  let overall_status = node_states.iter().fold(GraphRunStatus::Succeeded, |status, node| merge_status(status, node.status));
  let summary = node_states.iter().find(|node| node.status == GraphRunStatus::Failed).and_then(|node| node.detail.clone()).or_else(|| {
    node_states.iter().rev().find_map(|node| node.output_summary.clone())
  });
  GraphRunSnapshot { id: flow_run_id.to_string(), status: if node_states.is_empty() { GraphRunStatus::Idle } else { overall_status }, label: Some(chart.title.clone()), summary, started_at: None, finished_at: None, duration_ms: None, node_states, edge_states }
}

fn blocked_or_pending_detail(plan: &MermaidExecutionPlan, registry: &DesktopRuntimeRegistry, flow_run_id: &str, node_id: &str) -> (GraphRunStatus, Option<String>, Option<String>) {
  let Some(node) = plan.nodes.get(node_id) else {
    return (GraphRunStatus::Idle, None, None);
  };
  for dependency in &node.dependencies {
    let Some(agent) = registry.agent_for_flow_run_node(flow_run_id, dependency) else {
      return (GraphRunStatus::Pending, Some("Waiting on upstream steps.".to_string()), None);
    };
    if matches!(agent.status, DesktopAgentStatus::Error | DesktopAgentStatus::Aborted) {
      return (GraphRunStatus::Cancelled, Some(format!("Blocked by predecessor `{dependency}`.")), None);
    }
    if agent.status != DesktopAgentStatus::Completed {
      return (GraphRunStatus::Pending, Some("Waiting on upstream steps.".to_string()), None);
    }
  }
  if node.dependencies.is_empty() {
    (GraphRunStatus::Pending, Some("Waiting to launch.".to_string()), None)
  } else {
    (GraphRunStatus::Pending, Some("Ready to launch.".to_string()), None)
  }
}

fn edge_status(node_states: &[GraphNodeRunState], from: &str, to: &str) -> GraphRunStatus {
  let from_status = node_states.iter().find(|node| node.node_id == from).map(|node| node.status).unwrap_or(GraphRunStatus::Idle);
  let to_status = node_states.iter().find(|node| node.node_id == to).map(|node| node.status).unwrap_or(GraphRunStatus::Idle);
  match (from_status, to_status) {
    (_, GraphRunStatus::Failed) => GraphRunStatus::Failed,
    (_, GraphRunStatus::Cancelled) => GraphRunStatus::Cancelled,
    (_, GraphRunStatus::Running) => GraphRunStatus::Running,
    (GraphRunStatus::Succeeded, GraphRunStatus::Succeeded) => GraphRunStatus::Succeeded,
    (GraphRunStatus::Succeeded, GraphRunStatus::Pending) => GraphRunStatus::Pending,
    (GraphRunStatus::Running, _) => GraphRunStatus::Running,
    _ => GraphRunStatus::Idle,
  }
}

fn agent_status(status: DesktopAgentStatus) -> GraphRunStatus {
  match status {
    DesktopAgentStatus::Idle => GraphRunStatus::Idle,
    DesktopAgentStatus::Starting => GraphRunStatus::Pending,
    DesktopAgentStatus::Running => GraphRunStatus::Running,
    DesktopAgentStatus::Paused => GraphRunStatus::Warning,
    DesktopAgentStatus::Completed => GraphRunStatus::Succeeded,
    DesktopAgentStatus::Error => GraphRunStatus::Failed,
    DesktopAgentStatus::Aborted => GraphRunStatus::Cancelled,
  }
}

fn merge_status(left: GraphRunStatus, right: GraphRunStatus) -> GraphRunStatus {
  use GraphRunStatus::{Cancelled, Failed, Idle, Pending, Running, Succeeded, Warning};
  match (left, right) {
    (Failed, _) | (_, Failed) => Failed,
    (Cancelled, _) | (_, Cancelled) => Cancelled,
    (Running, _) | (_, Running) => Running,
    (Warning, _) | (_, Warning) => Warning,
    (Pending, _) | (_, Pending) => Pending,
    (Idle, status) => status,
    (status, Idle) => status,
    (Succeeded, Succeeded) => Succeeded,
  }
}
