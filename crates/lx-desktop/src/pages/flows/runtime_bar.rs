use std::collections::HashMap;

use dioxus::prelude::*;

use crate::runtime::types::{DesktopAgentRuntime, DesktopAgentStatus, DesktopRuntimeEvent, DesktopRuntimeEventKind, DesktopToolStatus};
use crate::runtime::{DesktopRuntimeRegistry, status_label, use_desktop_runtime};
use crate::widgets::PiWidget;
use lx_graph_editor::model::GraphDocument;
use lx_graph_editor::protocol::{GraphEdgeRunState, GraphNodeRunState, GraphRunSnapshot, GraphRunStatus};

use super::controller::use_flow_editor_state;

#[component]
pub fn FlowRuntimeBar() -> Element {
  let state = use_flow_editor_state();
  let runtime = use_desktop_runtime();

  if !state.supports_runtime() {
    return rsx! {};
  }

  let flow_id = state.flow_id.read().clone();
  let document = state.document.read().clone();
  let flow_agents = runtime.registry.agents_for_flow(&flow_id);
  let launch_prompt = flow_prompt(&document.title, document.metadata.notes.as_deref());
  let active_count =
    flow_agents.iter().filter(|agent| matches!(agent.status, DesktopAgentStatus::Starting | DesktopAgentStatus::Running | DesktopAgentStatus::Paused)).count();
  let completed_count = flow_agents.iter().filter(|agent| matches!(agent.status, DesktopAgentStatus::Completed)).count();
  let error_count = flow_agents.iter().filter(|agent| matches!(agent.status, DesktopAgentStatus::Error | DesktopAgentStatus::Aborted)).count();
  let selected_agent_id = state.active_run_agent_id.read().clone().filter(|agent_id| flow_agents.iter().any(|agent| agent.id == *agent_id));
  let selected_agent = selected_agent_id.as_ref().and_then(|agent_id| flow_agents.iter().find(|agent| agent.id == *agent_id)).cloned();
  let selected_snapshot = selected_agent.as_ref().map(|agent| build_flow_run_snapshot(&document, &runtime.registry, agent));

  {
    let selected_agent_id = selected_agent_id.clone();
    let selected_snapshot = selected_snapshot.clone();
    use_effect(move || {
      state.set_active_run_surface(selected_agent_id.clone(), selected_snapshot.clone());
    });
  }

  rsx! {
    div { class: "mb-4 rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-4",
      div { class: "flex flex-wrap items-center justify-between gap-4",
        div {
          div { class: "text-xs font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
            "Runtime"
          }
          p { class: "text-sm text-[var(--on-surface-variant)]",
            "Project the selected Pi-backed flow run onto the graph canvas and properties panel."
          }
        }
        div { class: "flex flex-wrap items-center gap-2",
          if selected_agent_id.is_some() {
            button {
              class: "btn-outline-sm",
              onclick: move |_| state.clear_run_surface(),
              "Clear Run Surface"
            }
          }
          button {
            class: "btn-outline-sm",
            onclick: move |_| {
                let name = format!("Flow {}", document.title);
                let launched = runtime
                    .launch_flow_pi_agent(
                        flow_id.clone(),
                        &name,
                        launch_prompt.clone(),
                        std::env::current_dir().ok(),
                    );
                state.set_active_run_surface(Some(launched), None);
            },
            "Launch Pi Flow Run"
          }
        }
      }
      if !flow_agents.is_empty() {
        div { class: "mt-3 flex flex-wrap items-center gap-2 text-xs text-[var(--outline)]",
          span { class: "rounded-full border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-2.5 py-1",
            "{active_count} active"
          }
          span { class: "rounded-full border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-2.5 py-1",
            "{completed_count} completed"
          }
          span { class: "rounded-full border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-2.5 py-1",
            "{error_count} error"
          }
        }
        div { class: "mt-3 flex flex-wrap gap-2",
          for agent in flow_agents.iter() {
            button {
              key: "{agent.id}",
              class: if selected_agent_id.as_deref() == Some(agent.id.as_str()) { "rounded-full border px-3 py-1.5 text-xs font-semibold" } else { "rounded-full border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-3 py-1.5 text-xs text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]" },
              style: if selected_agent_id.as_deref() == Some(agent.id.as_str()) { "border-color: color-mix(in srgb, var(--primary) 48%, transparent); background: color-mix(in srgb, var(--primary) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 84%, var(--primary) 16%);" } else { "" },
              onclick: {
                  let agent_id = agent.id.clone();
                  move |_| state.set_active_run_surface(Some(agent_id.clone()), None)
              },
              "{agent.name}"
            }
          }
        }
      }
      if let Some(snapshot) = selected_snapshot.clone() {
        div {
          class: "mt-4 rounded-2xl border px-4 py-3",
          style: "{run_snapshot_surface_style(snapshot.status)}",
          div { class: "flex flex-wrap items-start justify-between gap-3",
            div {
              div { class: "text-[11px] font-mono uppercase tracking-[0.18em] text-[var(--outline)]",
                "Selected run"
              }
              div { class: "mt-1 flex flex-wrap items-center gap-2",
                h3 { class: "text-sm font-semibold text-[var(--on-surface)]",
                  {snapshot.label.clone().unwrap_or_else(|| "Flow run".to_string())}
                }
                span {
                  class: "rounded-full border px-2.5 py-1 text-[11px] font-semibold",
                  style: "{run_snapshot_badge_style(snapshot.status)}",
                  "{run_status_label(snapshot.status)}"
                }
                if let Some(duration_ms) = snapshot.duration_ms {
                  span { class: "text-xs text-[var(--outline)]",
                    "{format_duration(duration_ms)}"
                  }
                }
              }
            }
            if let Some(agent) = selected_agent.clone() {
              div { class: "text-xs text-[var(--outline)]",
                "{status_label(&agent.status)}"
              }
            }
          }
          if let Some(summary) = snapshot.summary.clone() {
            p { class: "mt-2 text-sm leading-6 text-[var(--on-surface-variant)]",
              "{summary}"
            }
          }
        }
      } else if !flow_agents.is_empty() {
        p { class: "mt-3 text-xs text-[var(--outline)]",
          "Select a flow run to project execution state onto the graph."
        }
      }
      if let Some(agent_id) = selected_agent_id {
        div { class: "mt-4",
          PiWidget { agent_id }
        }
      }
    }
  }
}

fn flow_prompt(title: &str, notes: Option<&str>) -> String {
  match notes {
    Some(notes) if !notes.trim().is_empty() => format!("Work on the flow \"{title}\". Flow notes: {notes}"),
    _ => format!("Work on the flow \"{title}\" and report the next execution steps."),
  }
}

fn build_flow_run_snapshot(document: &GraphDocument, registry: &DesktopRuntimeRegistry, agent: &DesktopAgentRuntime) -> GraphRunSnapshot {
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

fn topological_node_ids(document: &GraphDocument) -> Vec<String> {
  let mut indegree: HashMap<String, usize> = document.nodes.iter().map(|node| (node.id.clone(), 0usize)).collect();
  let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
  let positions: HashMap<String, (f64, f64)> = document.nodes.iter().map(|node| (node.id.clone(), (node.position.x, node.position.y))).collect();

  for edge in &document.edges {
    if indegree.contains_key(&edge.from.node_id) && indegree.contains_key(&edge.to.node_id) {
      *indegree.entry(edge.to.node_id.clone()).or_insert(0) += 1;
      outgoing.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    }
  }

  let mut ready: Vec<String> = indegree.iter().filter(|(_, degree)| **degree == 0).map(|(node_id, _)| node_id.clone()).collect();
  sort_node_ids(&mut ready, &positions);

  let mut ordered = Vec::with_capacity(document.nodes.len());
  while let Some(node_id) = ready.first().cloned() {
    ready.remove(0);
    ordered.push(node_id.clone());
    if let Some(targets) = outgoing.get(&node_id) {
      for target in targets {
        if let Some(degree) = indegree.get_mut(target) {
          *degree = degree.saturating_sub(1);
          if *degree == 0 {
            ready.push(target.clone());
          }
        }
      }
      sort_node_ids(&mut ready, &positions);
    }
  }

  if ordered.len() < document.nodes.len() {
    let mut remaining: Vec<_> =
      document.nodes.iter().map(|node| node.id.clone()).filter(|node_id| !ordered.iter().any(|ordered_id| ordered_id == node_id)).collect();
    sort_node_ids(&mut remaining, &positions);
    ordered.extend(remaining);
  }

  ordered
}

fn sort_node_ids(node_ids: &mut [String], positions: &HashMap<String, (f64, f64)>) {
  node_ids.sort_by(|left, right| {
    let left_position = positions.get(left).copied().unwrap_or((0.0, 0.0));
    let right_position = positions.get(right).copied().unwrap_or((0.0, 0.0));
    left_position
      .0
      .partial_cmp(&right_position.0)
      .unwrap_or(std::cmp::Ordering::Equal)
      .then_with(|| left_position.1.partial_cmp(&right_position.1).unwrap_or(std::cmp::Ordering::Equal))
      .then_with(|| left.cmp(right))
  });
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

fn run_status_label(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => "idle",
    GraphRunStatus::Pending => "pending",
    GraphRunStatus::Running => "running",
    GraphRunStatus::Succeeded => "succeeded",
    GraphRunStatus::Warning => "warning",
    GraphRunStatus::Failed => "failed",
    GraphRunStatus::Cancelled => "cancelled",
  }
}

fn run_snapshot_badge_style(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => {
      "border-color: color-mix(in srgb, var(--outline-variant) 70%, transparent); background: color-mix(in srgb, var(--surface-container-high) 76%, transparent); color: var(--on-surface-variant);"
    },
    GraphRunStatus::Pending => {
      "border-color: color-mix(in srgb, var(--warning) 32%, transparent); background: color-mix(in srgb, var(--warning) 14%, transparent); color: color-mix(in srgb, var(--on-surface) 80%, var(--warning) 20%);"
    },
    GraphRunStatus::Running => {
      "border-color: color-mix(in srgb, var(--primary) 34%, transparent); background: color-mix(in srgb, var(--primary) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--primary) 18%);"
    },
    GraphRunStatus::Succeeded => {
      "border-color: color-mix(in srgb, var(--success) 34%, transparent); background: color-mix(in srgb, var(--success) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--success) 18%);"
    },
    GraphRunStatus::Warning => {
      "border-color: color-mix(in srgb, var(--warning) 34%, transparent); background: color-mix(in srgb, var(--warning) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--warning) 18%);"
    },
    GraphRunStatus::Failed => {
      "border-color: color-mix(in srgb, var(--error) 34%, transparent); background: color-mix(in srgb, var(--error) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--error) 18%);"
    },
    GraphRunStatus::Cancelled => {
      "border-color: color-mix(in srgb, var(--outline) 34%, transparent); background: color-mix(in srgb, var(--surface-container-high) 74%, transparent); color: var(--on-surface-variant);"
    },
  }
}

fn run_snapshot_surface_style(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => {
      "border-color: color-mix(in srgb, var(--outline-variant) 58%, transparent); background: color-mix(in srgb, var(--surface-container-high) 44%, transparent);"
    },
    GraphRunStatus::Pending => {
      "border-color: color-mix(in srgb, var(--warning) 22%, transparent); background: color-mix(in srgb, var(--warning) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Running => {
      "border-color: color-mix(in srgb, var(--primary) 22%, transparent); background: color-mix(in srgb, var(--primary) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Succeeded => {
      "border-color: color-mix(in srgb, var(--success) 22%, transparent); background: color-mix(in srgb, var(--success) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Warning => {
      "border-color: color-mix(in srgb, var(--warning) 22%, transparent); background: color-mix(in srgb, var(--warning) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Failed => {
      "border-color: color-mix(in srgb, var(--error) 22%, transparent); background: color-mix(in srgb, var(--error) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Cancelled => {
      "border-color: color-mix(in srgb, var(--outline) 22%, transparent); background: color-mix(in srgb, var(--surface-container-high) 48%, transparent);"
    },
  }
}

fn format_duration(duration_ms: u64) -> String {
  if duration_ms < 1_000 {
    return format!("{duration_ms} ms");
  }
  if duration_ms < 60_000 {
    return format!("{:.1} s", duration_ms as f64 / 1_000.0);
  }
  let minutes = duration_ms / 60_000;
  let seconds = (duration_ms % 60_000) / 1_000;
  format!("{minutes}m {seconds}s")
}
