use lx_graph_editor::protocol::{GraphEdgeRunState, GraphNodeRunState, GraphRunSnapshot, GraphRunStatus};

use crate::runtime::DesktopRuntimeRegistry;
use crate::runtime::events::transcript_rows;
use crate::runtime::types::DesktopAgentStatus;

use super::super::types::MermaidChart;
use super::plan::MermaidExecutionPlan;

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
          (agent_status(&agent.status), detail, output_summary)
        },
        None => blocked_or_pending_detail(plan, registry, flow_run_id, &node.id),
      };
      GraphNodeRunState {
        node_id: node.id.clone(),
        status,
        label: Some(node.display_label.clone()),
        detail,
        output_summary,
        started_at: None,
        finished_at: None,
        duration_ms: None,
      }
    })
    .collect::<Vec<_>>();
  let edge_states = chart
    .edges
    .iter()
    .map(|edge| GraphEdgeRunState {
      edge_id: edge.id.clone(),
      status: edge_status(&node_states, &edge.from, &edge.to),
      label: edge.label.clone(),
      detail: None,
    })
    .collect::<Vec<_>>();
  let overall_status = node_states.iter().fold(GraphRunStatus::Succeeded, |status, node| merge_status(status, node.status));
  let summary = node_states
    .iter()
    .find(|node| node.status == GraphRunStatus::Failed)
    .and_then(|node| node.detail.clone())
    .or_else(|| node_states.iter().rev().find_map(|node| node.output_summary.clone()));
  GraphRunSnapshot {
    id: flow_run_id.to_string(),
    status: if node_states.is_empty() { GraphRunStatus::Idle } else { overall_status },
    label: Some(chart.title.clone()),
    summary,
    started_at: None,
    finished_at: None,
    duration_ms: None,
    node_states,
    edge_states,
  }
}

fn blocked_or_pending_detail(
  plan: &MermaidExecutionPlan,
  registry: &DesktopRuntimeRegistry,
  flow_run_id: &str,
  node_id: &str,
) -> (GraphRunStatus, Option<String>, Option<String>) {
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

fn agent_status(status: &DesktopAgentStatus) -> GraphRunStatus {
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

#[cfg(test)]
mod tests {
  use super::*;
  use crate::pages::flows::mermaid::build_execution_plan;
  use crate::pages::flows::mermaid::types::{MermaidDirection, MermaidEdge, MermaidNode, MermaidNodeMetadata, MermaidSemanticKind};
  use crate::runtime::types::{DesktopAgentLaunchSpec, DesktopAgentRuntime, DesktopRuntimeEvent, DesktopRuntimeEventKind, text_payload};

  fn sample_chart() -> MermaidChart {
    MermaidChart {
      title: "Chart".to_string(),
      notes: None,
      direction: MermaidDirection::TopDown,
      nodes: vec![
        MermaidNode {
          id: "a".to_string(),
          semantic_kind: MermaidSemanticKind::Agent,
          display_label: "A".to_string(),
          subgraph_id: None,
          metadata: MermaidNodeMetadata::default(),
        },
        MermaidNode {
          id: "b".to_string(),
          semantic_kind: MermaidSemanticKind::Agent,
          display_label: "B".to_string(),
          subgraph_id: None,
          metadata: MermaidNodeMetadata::default(),
        },
      ],
      edges: vec![MermaidEdge { id: "edge-a-b".to_string(), from: "a".to_string(), to: "b".to_string(), label: None }],
      subgraphs: Vec::new(),
    }
  }

  #[test]
  fn blocked_downstream_nodes_are_cancelled() {
    let registry = DesktopRuntimeRegistry::new();
    let plan = build_execution_plan(&sample_chart()).expect("plan should build");
    let mut agent = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("A", "a", "a"));
    agent.id = "agent-a".to_string();
    agent.flow_run_id = Some("run-1".to_string());
    agent.flow_node_id = Some("a".to_string());
    agent.status = DesktopAgentStatus::Error;
    registry.register_agent(agent);

    let snapshot = build_run_snapshot(&sample_chart(), &plan, &registry, "run-1");
    let downstream = snapshot.node_states.iter().find(|node| node.node_id == "b").expect("downstream node state");
    assert_eq!(downstream.status, GraphRunStatus::Cancelled);
  }

  #[test]
  fn snapshot_projects_runtime_state_to_graph_node_ids() {
    let registry = DesktopRuntimeRegistry::new();
    let plan = build_execution_plan(&sample_chart()).expect("plan should build");
    let mut agent = DesktopAgentRuntime::new(&DesktopAgentLaunchSpec::new("A", "a", "a"));
    agent.id = "agent-a".to_string();
    agent.flow_run_id = Some("run-1".to_string());
    agent.flow_node_id = Some("a".to_string());
    agent.status = DesktopAgentStatus::Completed;
    registry.register_agent(agent.clone());
    registry.append_event(DesktopRuntimeEvent {
      id: "event-a".to_string(),
      agent_id: agent.id.clone(),
      kind: DesktopRuntimeEventKind::MessageComplete,
      ts: "1".to_string(),
      payload: text_payload("assistant", "done"),
    });

    let snapshot = build_run_snapshot(&sample_chart(), &plan, &registry, "run-1");
    let upstream = snapshot.node_states.iter().find(|node| node.node_id == "a").expect("upstream node state");
    assert_eq!(upstream.status, GraphRunStatus::Succeeded);
    assert_eq!(upstream.output_summary.as_deref(), Some("done"));
  }
}
