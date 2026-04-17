use dioxus::prelude::*;
use lx_graph_editor::model::GraphDocument;
use lx_graph_editor::protocol::GraphRunStatus;

use crate::runtime::{status_label, use_desktop_runtime};
use crate::widgets::PiWidget;

use super::{build_execution_plan, build_run_snapshot, chart_from_graph_document, launch_ready_nodes};
use crate::pages::flows::controller::use_flow_editor_state;

#[derive(Clone, Debug, PartialEq, Eq)]
struct MermaidRunGroup {
  run_id: String,
  title: String,
  agents: Vec<crate::runtime::types::DesktopAgentRuntime>,
  root_status: crate::runtime::types::DesktopAgentStatus,
}

#[component]
pub fn MermaidRuntimeBar(flow_id: String, document: GraphDocument) -> Element {
  let mut state = use_flow_editor_state();
  let runtime = use_desktop_runtime();
  let chart = chart_from_graph_document(&document);
  let plan = build_execution_plan(&chart).ok();
  let run_groups = mermaid_run_groups(&runtime.registry, &flow_id);
  let flow_agents = runtime.registry.agents_for_flow(&flow_id);
  let selected_agent_id = state.active_run_agent_id.read().clone().filter(|agent_id| flow_agents.iter().any(|agent| agent.id == *agent_id));
  let selected_run = selected_agent_id
    .as_ref()
    .and_then(|agent_id| run_groups.iter().find(|run| run.agents.iter().any(|agent| agent.id == *agent_id)).cloned())
    .or_else(|| run_groups.first().cloned());
  let selected_session = selected_run.as_ref().and_then(|run| {
    selected_agent_id.as_ref().and_then(|agent_id| run.agents.iter().find(|agent| &agent.id == agent_id).cloned()).or_else(|| run.agents.first().cloned())
  });
  let selected_snapshot = selected_run.as_ref().and_then(|run| plan.as_ref().map(|plan| build_run_snapshot(&chart, plan, &runtime.registry, &run.run_id)));
  let selected_session_id = selected_session.as_ref().map(|agent| agent.id.clone());
  let selected_run_id = selected_run.as_ref().map(|run| run.run_id.clone());

  {
    let run_groups = run_groups.clone();
    let runtime = runtime.clone();
    let flow_id = flow_id.clone();
    let chart = chart.clone();
    let plan = plan.clone();
    let selected_session_id = selected_session_id.clone();
    let selected_snapshot = selected_snapshot.clone();
    use_effect(move || {
      if let Some(plan) = plan.clone() {
        for run in &run_groups {
          let _ = launch_ready_nodes(&runtime, &runtime.registry, &chart, &plan, &flow_id, &run.run_id);
        }
      }
      state.set_active_run_surface(selected_session_id.clone(), selected_snapshot.clone());
    });
  }

  let launch = {
    let runtime = runtime.clone();
    let flow_id = flow_id.clone();
    let chart = chart.clone();
    let plan = plan.clone();
    move |_| {
      let Some(plan) = plan.clone() else {
        state.status_message.set(Some("Mermaid execution is blocked by validation errors.".to_string()));
        return;
      };
      let run_id = runtime.create_flow_run(flow_id.clone(), chart.title.clone());
      let launched = launch_ready_nodes(&runtime, &runtime.registry, &chart, &plan, &flow_id, &run_id);
      state.set_active_run_surface(launched.first().cloned(), None);
    }
  };

  rsx! {
    div { class: "mb-4 rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container)] p-4",
      div { class: "flex flex-wrap items-center justify-between gap-4",
        div {
          div { class: "text-xs font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
            "Runtime"
          }
          p { class: "text-sm text-[var(--on-surface-variant)]",
            "Execute the Mermaid chart as grouped mock-lx Pi sessions."
          }
        }
        div { class: "flex flex-wrap items-center gap-2",
          if selected_session_id.is_some() {
            button {
              class: "btn-outline-sm",
              onclick: move |_| state.clear_run_surface(),
              "Clear Run Surface"
            }
          }
          button { class: "btn-outline-sm", onclick: launch, "Launch Mermaid Run" }
        }
      }
      if !run_groups.is_empty() {
        div { class: "mt-3 grid gap-2 md:grid-cols-2 xl:grid-cols-3",
          for run in run_groups.iter() {
            button {
              key: "{run.run_id}",
              class: if selected_run_id.as_deref() == Some(run.run_id.as_str()) { "rounded-2xl border px-4 py-3 text-left" } else { "rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-4 py-3 text-left transition-colors hover:bg-[var(--surface-container-highest)]" },
              style: if selected_run_id.as_deref() == Some(run.run_id.as_str()) { "border-color: color-mix(in srgb, var(--primary) 48%, transparent); background: color-mix(in srgb, var(--primary) 12%, transparent);" } else { "" },
              onclick: {
                  let agent_id = run.agents.first().map(|agent| agent.id.clone());
                  move |_| state.set_active_run_surface(agent_id.clone(), None)
              },
              div { class: "flex items-start justify-between gap-3",
                div { class: "min-w-0",
                  h3 { class: "text-sm font-semibold text-[var(--on-surface)] truncate",
                    "{run.title}"
                  }
                  p { class: "text-xs text-[var(--outline)]",
                    "{run.agents.len()} sessions"
                  }
                }
                span { class: "rounded-full border px-2.5 py-1 text-[11px] font-semibold",
                  "{status_label(&run.root_status)}"
                }
              }
            }
          }
        }
      }
      if let Some(snapshot) = selected_snapshot.clone() {
        div { class: "mt-4 rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-low)] px-4 py-3",
          div { class: "text-[11px] font-mono uppercase tracking-[0.18em] text-[var(--outline)]",
            "Selected run"
          }
          h3 { class: "mt-1 text-sm font-semibold text-[var(--on-surface)]",
            {snapshot.label.clone().unwrap_or_else(|| "Mermaid run".to_string())}
          }
          if let Some(summary) = snapshot.summary.clone() {
            p { class: "mt-2 text-sm leading-6 text-[var(--on-surface-variant)]",
              "{summary}"
            }
          }
        }
        div { class: "mt-4 rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-low)] p-3",
          div { class: "text-xs font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
            "Nodes In Run"
          }
          div { class: "mt-3 flex flex-wrap gap-2",
            for node_state in snapshot.node_states.iter() {
              button {
                key: "{node_state.node_id}",
                class: "rounded-full border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-3 py-1.5 text-xs text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
                onclick: {
                    let agent_id = selected_run
                        .as_ref()
                        .and_then(|run| {
                            runtime
                                .registry
                                .agent_for_flow_run_node(&run.run_id, &node_state.node_id)
                                .map(|agent| agent.id)
                        });
                    move |_| {
                        if let Some(agent_id) = agent_id.clone() {
                            state.set_active_run_surface(Some(agent_id), None);
                        }
                    }
                },
                "{node_state.label.clone().unwrap_or_else(|| node_state.node_id.clone())} · {run_status_label(node_state.status)}"
              }
            }
          }
        }
      }
      if let Some(agent_id) = selected_session_id {
        div { class: "mt-4",
          PiWidget { agent_id }
        }
      }
    }
  }
}

fn mermaid_run_groups(registry: &crate::runtime::DesktopRuntimeRegistry, flow_id: &str) -> Vec<MermaidRunGroup> {
  registry
    .flow_runs_for_flow(flow_id)
    .into_iter()
    .map(|summary| {
      let run_id = summary.id;
      MermaidRunGroup { run_id: run_id.clone(), title: summary.title, agents: registry.agents_for_flow_run(&run_id), root_status: summary.root_agent_status }
    })
    .collect()
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
