use std::collections::HashMap;

use lx_graph_editor::model::GraphDocument;
use lx_graph_editor::protocol::{GraphEdgeRunState, GraphNodeRunState, GraphRunSnapshot, GraphRunStatus};

use crate::runtime::DesktopRuntimeRegistry;
use crate::runtime::types::{DesktopAgentRuntime, DesktopAgentStatus, DesktopRuntimeEvent, DesktopRuntimeEventKind, DesktopToolStatus};

use super::topology::topological_node_ids;

pub fn build_flow_run_snapshot(document: &GraphDocument, registry: &DesktopRuntimeRegistry, agent: &DesktopAgentRuntime) -> GraphRunSnapshot {
  let ordered_node_ids = topological_node_ids(document);
  let events = registry.events_for_agent(&agent.id);
  let running_tool = registry.tools_for_agent(&agent.id).into_iter().find(|tool| tool.status == DesktopToolStatus::Running);
  let completed_tool_events: Vec<_> = events.iter().filter(|event| matches!(event.kind, DesktopRuntimeEventKind::ToolResult)).cloned().collect();
  let failed_tool_events: Vec<_> =
    events.iter().filter(|event| matches!(event.kind, DesktopRuntimeEventKind::ToolError | DesktopRuntimeEventKind::BackendError)).cloned().collect();
  let completed_steps = completed_tool_events.len().min(ordered_node_ids.len());
  let overall_status = overall_run_status(&agent.status, !failed_tool_events.is_empty());
  let duration_ms = duration_between(&agent.created_at, &agent.last_event_at);
  let latest_summary = events.iter().rev().find_map(event_summary);
  let last_failure = failed_tool_events.iter().rev().find_map(event_summary);

  let node_states = ordered_node_ids
    .iter()
    .enumerate()
    .map(|(index, node_id)| {
      let status = node_run_status(index, completed_steps, ordered_node_ids.len(), &agent.status, !failed_tool_events.is_empty());
      let completed_event = completed_tool_events.get(index);
      let label = match status {
        GraphRunStatus::Running => running_tool.as_ref().map(|tool| tool.tool_name.clone()),
        GraphRunStatus::Succeeded => completed_event.and_then(tool_name_for_event),
        GraphRunStatus::Failed => failed_tool_events.last().and_then(tool_name_for_event),
        _ => None,
      };
      let detail = match status {
        GraphRunStatus::Running => running_tool
          .as_ref()
          .and_then(|tool| tool.result_preview.clone())
          .or_else(|| latest_summary.clone())
          .or_else(|| Some("Execution is in progress.".to_string())),
        GraphRunStatus::Succeeded => completed_event.and_then(event_summary),
        GraphRunStatus::Failed => last_failure.clone().or_else(|| Some("The run ended with an error.".to_string())),
        GraphRunStatus::Warning => latest_summary.clone(),
        GraphRunStatus::Pending => Some("Waiting on upstream steps.".to_string()),
        GraphRunStatus::Cancelled => Some("Execution was cancelled before this step finished.".to_string()),
        GraphRunStatus::Idle => None,
      };
      let output_summary = if status == GraphRunStatus::Succeeded { completed_event.and_then(event_summary) } else { None };

      GraphNodeRunState {
        node_id: node_id.clone(),
        status,
        label,
        detail,
        output_summary,
        started_at: None,
        finished_at: None,
        duration_ms: if status == GraphRunStatus::Running { duration_ms } else { None },
      }
    })
    .collect();

  let node_index: HashMap<_, _> = ordered_node_ids.iter().enumerate().map(|(index, node_id)| (node_id.clone(), index)).collect();
  let edge_states = document
    .edges
    .iter()
    .filter_map(|edge| {
      let from_index = node_index.get(&edge.from.node_id).copied()?;
      let to_index = node_index.get(&edge.to.node_id).copied()?;
      let status = edge_run_status(from_index, to_index, completed_steps, ordered_node_ids.len(), &agent.status, !failed_tool_events.is_empty());
      Some(GraphEdgeRunState { edge_id: edge.id.clone(), status, label: None, detail: edge_run_detail(status) })
    })
    .collect();

  GraphRunSnapshot {
    id: agent.id.clone(),
    status: overall_status,
    label: Some(agent.name.clone()),
    summary: last_failure.or(latest_summary).or_else(|| Some(format!("Mapped {} graph steps to the selected Pi flow run.", ordered_node_ids.len()))),
    started_at: None,
    finished_at: None,
    duration_ms,
    node_states,
    edge_states,
  }
}

fn overall_run_status(status: &DesktopAgentStatus, has_failures: bool) -> GraphRunStatus {
  match status {
    DesktopAgentStatus::Idle => GraphRunStatus::Idle,
    DesktopAgentStatus::Starting => GraphRunStatus::Pending,
    DesktopAgentStatus::Running => GraphRunStatus::Running,
    DesktopAgentStatus::Paused => GraphRunStatus::Warning,
    DesktopAgentStatus::Completed if has_failures => GraphRunStatus::Warning,
    DesktopAgentStatus::Completed => GraphRunStatus::Succeeded,
    DesktopAgentStatus::Error => GraphRunStatus::Failed,
    DesktopAgentStatus::Aborted => GraphRunStatus::Cancelled,
  }
}

fn node_run_status(index: usize, completed_steps: usize, node_count: usize, agent_status: &DesktopAgentStatus, has_failures: bool) -> GraphRunStatus {
  match agent_status {
    DesktopAgentStatus::Idle => GraphRunStatus::Idle,
    DesktopAgentStatus::Starting => {
      if index == 0 {
        GraphRunStatus::Pending
      } else {
        GraphRunStatus::Idle
      }
    },
    DesktopAgentStatus::Running => {
      if index < completed_steps {
        GraphRunStatus::Succeeded
      } else if index == completed_steps.min(node_count.saturating_sub(1)) {
        GraphRunStatus::Running
      } else {
        GraphRunStatus::Pending
      }
    },
    DesktopAgentStatus::Paused => {
      if index < completed_steps {
        GraphRunStatus::Succeeded
      } else if index == completed_steps.min(node_count.saturating_sub(1)) {
        GraphRunStatus::Warning
      } else {
        GraphRunStatus::Pending
      }
    },
    DesktopAgentStatus::Completed if has_failures => {
      if index < completed_steps {
        GraphRunStatus::Succeeded
      } else if index == completed_steps.min(node_count.saturating_sub(1)) {
        GraphRunStatus::Warning
      } else {
        GraphRunStatus::Pending
      }
    },
    DesktopAgentStatus::Completed => GraphRunStatus::Succeeded,
    DesktopAgentStatus::Error => {
      if index < completed_steps {
        GraphRunStatus::Succeeded
      } else if index == completed_steps.min(node_count.saturating_sub(1)) {
        GraphRunStatus::Failed
      } else {
        GraphRunStatus::Idle
      }
    },
    DesktopAgentStatus::Aborted => {
      if index < completed_steps {
        GraphRunStatus::Succeeded
      } else if index == completed_steps.min(node_count.saturating_sub(1)) {
        GraphRunStatus::Cancelled
      } else {
        GraphRunStatus::Idle
      }
    },
  }
}

fn edge_run_status(
  from_index: usize,
  to_index: usize,
  completed_steps: usize,
  node_count: usize,
  agent_status: &DesktopAgentStatus,
  has_failures: bool,
) -> GraphRunStatus {
  let current_index = completed_steps.min(node_count.saturating_sub(1));
  match agent_status {
    DesktopAgentStatus::Idle => GraphRunStatus::Idle,
    DesktopAgentStatus::Starting => {
      if to_index == 0 || from_index == 0 {
        GraphRunStatus::Pending
      } else {
        GraphRunStatus::Idle
      }
    },
    DesktopAgentStatus::Running => {
      if to_index < completed_steps {
        GraphRunStatus::Succeeded
      } else if to_index == current_index {
        GraphRunStatus::Running
      } else if from_index <= current_index {
        GraphRunStatus::Pending
      } else {
        GraphRunStatus::Idle
      }
    },
    DesktopAgentStatus::Paused => {
      if to_index < completed_steps {
        GraphRunStatus::Succeeded
      } else if to_index == current_index {
        GraphRunStatus::Warning
      } else if from_index <= current_index {
        GraphRunStatus::Pending
      } else {
        GraphRunStatus::Idle
      }
    },
    DesktopAgentStatus::Completed if has_failures => {
      if to_index < completed_steps {
        GraphRunStatus::Succeeded
      } else if to_index == current_index {
        GraphRunStatus::Warning
      } else {
        GraphRunStatus::Pending
      }
    },
    DesktopAgentStatus::Completed => GraphRunStatus::Succeeded,
    DesktopAgentStatus::Error => {
      if to_index < completed_steps {
        GraphRunStatus::Succeeded
      } else if to_index == current_index {
        GraphRunStatus::Failed
      } else {
        GraphRunStatus::Idle
      }
    },
    DesktopAgentStatus::Aborted => {
      if to_index < completed_steps {
        GraphRunStatus::Succeeded
      } else if to_index == current_index {
        GraphRunStatus::Cancelled
      } else {
        GraphRunStatus::Idle
      }
    },
  }
}

fn edge_run_detail(status: GraphRunStatus) -> Option<String> {
  match status {
    GraphRunStatus::Running => Some("in flight".to_string()),
    GraphRunStatus::Succeeded => Some("delivered".to_string()),
    GraphRunStatus::Pending => Some("queued".to_string()),
    GraphRunStatus::Failed => Some("blocked".to_string()),
    GraphRunStatus::Cancelled => Some("cancelled".to_string()),
    GraphRunStatus::Warning => Some("paused".to_string()),
    GraphRunStatus::Idle => None,
  }
}

fn tool_name_for_event(event: &DesktopRuntimeEvent) -> Option<String> {
  event.payload.get("tool_name").and_then(serde_json::Value::as_str).map(ToOwned::to_owned)
}

fn event_summary(event: &DesktopRuntimeEvent) -> Option<String> {
  event
    .payload
    .get("text")
    .and_then(serde_json::Value::as_str)
    .map(ToOwned::to_owned)
    .or_else(|| event.payload.get("tool_name").and_then(serde_json::Value::as_str).map(|tool_name| format!("{tool_name} finished")))
}

fn duration_between(start_ms: &str, end_ms: &str) -> Option<u64> {
  let start = start_ms.parse::<u64>().ok()?;
  let end = end_ms.parse::<u64>().ok()?;
  Some(end.saturating_sub(start))
}
