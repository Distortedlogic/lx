mod groups;
mod snapshot;
mod styles;
mod topology;

use dioxus::prelude::*;

use crate::runtime::types::{DesktopAgentRuntime, DesktopAgentStatus};
use crate::runtime::{status_label, use_desktop_runtime};
use crate::widgets::PiWidget;

use self::groups::{FlowRunGroup, flow_run_groups};
use self::snapshot::build_flow_run_snapshot;
use self::styles::{format_duration, run_snapshot_badge_style, run_snapshot_surface_style, run_status_label};
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
  let run_groups = flow_run_groups(&runtime.registry, &flow_id);
  let launch_prompt = flow_prompt(&document.title, document.metadata.notes.as_deref());
  let active_count =
    flow_agents.iter().filter(|agent| matches!(agent.status, DesktopAgentStatus::Starting | DesktopAgentStatus::Running | DesktopAgentStatus::Paused)).count();
  let completed_count = flow_agents.iter().filter(|agent| matches!(agent.status, DesktopAgentStatus::Completed)).count();
  let error_count = flow_agents.iter().filter(|agent| matches!(agent.status, DesktopAgentStatus::Error | DesktopAgentStatus::Aborted)).count();
  let selected_agent_id = state.active_run_agent_id.read().clone().filter(|agent_id| flow_agents.iter().any(|agent| agent.id == *agent_id));
  let selected_run = selected_run_group(&run_groups, selected_agent_id.as_deref());
  let selected_session = selected_session_agent(selected_run.as_ref(), selected_agent_id.as_deref());
  let selected_snapshot = selected_run
    .as_ref()
    .and_then(|run| runtime.registry.find_agent(&run.root_agent_id))
    .map(|agent| build_flow_run_snapshot(&document, &runtime.registry, &agent));
  let selected_run_id = selected_run.as_ref().map(|run| run.run_id.clone());
  let selected_session_id = selected_session.as_ref().map(|agent| agent.id.clone());

  {
    let selected_session_id = selected_session_id.clone();
    let selected_snapshot = selected_snapshot.clone();
    use_effect(move || {
      state.set_active_run_surface(selected_session_id.clone(), selected_snapshot.clone());
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
            "Project grouped runtime sessions onto the graph canvas and properties panel."
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
          button {
            class: "btn-outline-sm",
            onclick: {
                let document_for_run = document.clone();
                let flow_id_for_run = flow_id.clone();
                move |_| {
                    let templates = state.templates.read().clone();
                    let document = document_for_run.clone();
                    let flow_id = flow_id_for_run.clone();
                    spawn(async move {
                        let mut registry = crate::engine::default_registry();
                        let credentials = match crate::credentials::CredentialStore::file_backed() {
                            Ok(store) => store,
                            Err(_) => crate::credentials::CredentialStore::in_memory(),
                        };
                        let flow_persistence = crate::pages::flows::storage::FlowPersistence::file_backed();
                        crate::engine::register_sub_workflow_runner(&mut registry, flow_persistence, credentials.clone());
                        let (report, context) = crate::engine::execute_flow(&document, &templates, &registry, &credentials).await;
                        let persistence = crate::engine::FlowRunPersistence::file_backed();
                        let run_id = uuid::Uuid::new_v4().to_string();
                        let _ = persistence.save(&flow_id, &run_id, &report, &context);
                        let message = if report.aborted {
                            format!("Run aborted: {}", report.error.as_deref().unwrap_or("unknown error"))
                        } else {
                            format!(
                                "Run {run_id} complete: {} ok, {} failed, {} skipped",
                                report.success_count(),
                                report.failed_count(),
                                report.skipped_count()
                            )
                        };
                        state.set_status_message(message);
                    });
                }
            },
            "Run Locally"
          }
          button {
            class: "btn-outline-sm",
            onclick: {
                let document_for_activate = document.clone();
                let flow_id_for_activate = flow_id.clone();
                move |_| {
                    let document = document_for_activate.clone();
                    let flow_id = flow_id_for_activate.clone();
                    let templates = state.templates.read().clone();
                    let scheduler: Option<crate::engine::FlowScheduler> = try_consume_context();
                    spawn(async move {
                        let Some(scheduler) = scheduler else {
                            state.set_status_message("Scheduler not available".to_string());
                            return;
                        };
                        scheduler.register_flow(flow_id.clone(), crate::engine::FlowActivation { active: true, document, templates }).await;
                        let _ = crate::engine::set_flow_active(&flow_id, true);
                        state.set_status_message(format!("Flow `{flow_id}` activated (cron/webhook enabled)"));
                    });
                }
            },
            "Activate"
          }
          button {
            class: "btn-outline-sm",
            onclick: {
                let flow_id_for_deactivate = flow_id.clone();
                move |_| {
                    let flow_id = flow_id_for_deactivate.clone();
                    let scheduler: Option<crate::engine::FlowScheduler> = try_consume_context();
                    spawn(async move {
                        if let Some(scheduler) = scheduler {
                            scheduler.set_active(&flow_id, false).await;
                            let _ = crate::engine::set_flow_active(&flow_id, false);
                            state.set_status_message(format!("Flow `{flow_id}` deactivated"));
                        }
                    });
                }
            },
            "Deactivate"
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
        div { class: "mt-3 grid gap-2 md:grid-cols-2 xl:grid-cols-3",
          for run in run_groups.iter() {
            button {
              key: "{run.run_id}",
              class: if selected_run_id.as_deref() == Some(run.run_id.as_str()) { "rounded-2xl border px-4 py-3 text-left" } else { "rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-4 py-3 text-left transition-colors hover:bg-[var(--surface-container-highest)]" },
              style: if selected_run_id.as_deref() == Some(run.run_id.as_str()) { "border-color: color-mix(in srgb, var(--primary) 48%, transparent); background: color-mix(in srgb, var(--primary) 12%, transparent);" } else { "" },
              onclick: {
                  let agent_id = run.root_agent_id.clone();
                  move |_| state.set_active_run_surface(Some(agent_id.clone()), None)
              },
              div { class: "flex items-start justify-between gap-3",
                div { class: "min-w-0",
                  h3 { class: "text-sm font-semibold text-[var(--on-surface)] truncate",
                    "{run.title}"
                  }
                  p { class: "text-xs text-[var(--outline)]",
                    "{session_count_label(run.agents.len())}"
                  }
                }
                span {
                  class: "rounded-full border px-2.5 py-1 text-[11px] font-semibold",
                  style: if selected_run_id.as_deref() == Some(run.run_id.as_str()) { "border-color: color-mix(in srgb, var(--primary) 36%, transparent); background: color-mix(in srgb, var(--primary) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 84%, var(--primary) 16%);" } else { "border-color: color-mix(in srgb, var(--outline-variant) 70%, transparent); background: color-mix(in srgb, var(--surface-container-high) 76%, transparent); color: var(--on-surface-variant);" },
                  "{status_label(&run.root_agent_status)}"
                }
              }
              p { class: "mt-2 text-xs text-[var(--outline)] truncate",
                "{run.root_agent_name}"
              }
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
            if let Some(run) = selected_run.as_ref() {
              div { class: "text-xs text-[var(--outline)]",
                "{status_label(&run.root_agent_status)}"
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
      if let Some(run) = selected_run.clone() {
        div { class: "mt-4 rounded-2xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-low)] p-3",
          div { class: "text-xs font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
            "Sessions In Run"
          }
          div { class: "mt-3 flex flex-wrap gap-2",
            for agent in run.agents.iter() {
              button {
                key: "{agent.id}",
                class: if selected_session_id.as_deref() == Some(agent.id.as_str()) { "rounded-full border px-3 py-1.5 text-xs font-semibold" } else { "rounded-full border border-[var(--outline-variant)]/30 bg-[var(--surface-container-high)] px-3 py-1.5 text-xs text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]" },
                style: if selected_session_id.as_deref() == Some(agent.id.as_str()) { "border-color: color-mix(in srgb, var(--primary) 48%, transparent); background: color-mix(in srgb, var(--primary) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 84%, var(--primary) 16%);" } else { "" },
                onclick: {
                    let agent_id = agent.id.clone();
                    move |_| state.set_active_run_surface(Some(agent_id.clone()), None)
                },
                "{agent.name} · {status_label(&agent.status)}"
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

fn flow_prompt(title: &str, notes: Option<&str>) -> String {
  match notes {
    Some(notes) if !notes.trim().is_empty() => format!("Work on the flow \"{title}\". Flow notes: {notes}"),
    _ => format!("Work on the flow \"{title}\" and report the next execution steps."),
  }
}

fn selected_run_group(run_groups: &[FlowRunGroup], agent_id: Option<&str>) -> Option<FlowRunGroup> {
  match agent_id {
    Some(agent_id) => run_groups.iter().find(|run| run.agents.iter().any(|agent| agent.id == agent_id)).cloned().or_else(|| run_groups.first().cloned()),
    None => run_groups.first().cloned(),
  }
}

fn selected_session_agent(run_group: Option<&FlowRunGroup>, agent_id: Option<&str>) -> Option<DesktopAgentRuntime> {
  let run_group = run_group?;
  agent_id
    .and_then(|agent_id| run_group.agents.iter().find(|agent| agent.id == agent_id).cloned())
    .or_else(|| run_group.agents.iter().find(|agent| agent.id == run_group.root_agent_id).cloned())
    .or_else(|| run_group.agents.first().cloned())
}

fn session_count_label(count: usize) -> String {
  if count == 1 { "1 session".to_string() } else { format!("{count} sessions") }
}
