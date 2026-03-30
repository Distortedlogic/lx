use super::budget_tab::BudgetTab;
use super::config_form::AgentConfigPanel;
use super::list::StatusBadge;
use super::overview::AgentOverview;
use super::run_types::{BudgetSummary, HeartbeatRun, SkillSnapshot};
use super::runs_tab::RunsTab;
use super::skills_tab::SkillsTab;
use super::types::{AgentDetail as AgentDetailData, AgentDetailTab, role_label};
use crate::contexts::activity_log::ActivityLog;
use crate::styles::{BTN_OUTLINE_SM, TAB_ACTIVE, TAB_INACTIVE};
use dioxus::prelude::*;

#[component]
pub fn AgentDetailShell(
  agent: AgentDetailData,
  on_back: EventHandler<()>,
  on_run: EventHandler<()>,
  on_pause: EventHandler<()>,
  on_resume: EventHandler<()>,
  on_terminate: EventHandler<()>,
) -> Element {
  let mut active_tab = use_signal(|| AgentDetailTab::Overview);

  let log = use_context::<ActivityLog>();
  let all_events = log.events.read();

  let agent_events: Vec<_> = all_events.iter().filter(|e| e.message.contains(&agent.name) || e.kind.contains("agent")).cloned().collect();

  let runs: Vec<HeartbeatRun> = {
    let mut run_list = Vec::new();
    for event in agent_events.iter() {
      if event.kind == "agent_start" || event.kind == "agent_running" {
        let status = if event.kind == "agent_running" { "running" } else { "queued" };
        let already = run_list.iter().any(|r: &HeartbeatRun| r.id == event.timestamp);
        if !already {
          run_list.push(HeartbeatRun {
            id: event.timestamp.clone(),
            agent_id: agent.id.clone(),
            company_id: String::new(),
            status: status.to_string(),
            invocation_source: "on_demand".to_string(),
            trigger_detail: None,
            started_at: Some(event.timestamp.clone()),
            finished_at: None,
            created_at: event.timestamp.clone(),
            error: None,
            error_code: None,
            usage_json: None,
            result_json: None,
            context_snapshot: None,
          });
        }
      }
    }
    run_list
  };

  let role_text = role_label(&agent.role);
  let subtitle = match &agent.title {
    Some(t) => format!("{role_text} - {t}"),
    None => role_text.to_string(),
  };
  let is_paused = agent.status == "paused";

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-6",
      div { class: "flex items-center justify-between gap-2",
        div { class: "flex items-center gap-3 min-w-0",
          button {
            class: "shrink-0 text-xs text-[var(--outline)] hover:text-[var(--on-surface)]",
            onclick: move |_| on_back.call(()),
            "< Back"
          }
          AgentIconDisplay { icon: agent.icon.clone() }
          div { class: "min-w-0",
            h2 { class: "text-2xl font-bold text-[var(--on-surface)] truncate",
              "{agent.name}"
            }
            p { class: "text-sm text-[var(--outline)] truncate", "{subtitle}" }
          }
        }
        div { class: "flex items-center gap-2 shrink-0",
          button {
            class: BTN_OUTLINE_SM,
            onclick: move |_| on_run.call(()),
            "Run"
          }
          if is_paused {
            button {
              class: BTN_OUTLINE_SM,
              onclick: move |_| on_resume.call(()),
              "Resume"
            }
          } else {
            button {
              class: BTN_OUTLINE_SM,
              onclick: move |_| on_pause.call(()),
              "Pause"
            }
          }
          StatusBadge { status: agent.status.clone() }
        }
      }
      div { class: "flex gap-1 border-b border-[var(--outline-variant)]/30",
        for tab in AgentDetailTab::all() {
          button {
            class: if *active_tab.read() == *tab { TAB_ACTIVE } else { TAB_INACTIVE },
            onclick: {
                let tab = tab.clone();
                move |_| active_tab.set(tab.clone())
            },
            "{tab.label()}"
          }
        }
      }
      match *active_tab.read() {
          AgentDetailTab::Overview => rsx! {
            AgentOverview { agent: agent.clone() }
          },
          AgentDetailTab::Config => rsx! {
            AgentConfigPanel { agent: agent.clone() }
          },
          AgentDetailTab::Runs => rsx! {
            RunsTab { runs: runs.clone(), agent_route_id: agent.id.clone() }
          },
          AgentDetailTab::Skills => rsx! {
            SkillsTab {
              snapshot: SkillSnapshot {
                  entries: Vec::new(),
                  desired_skills: Vec::new(),
              },
            }
          },
          AgentDetailTab::Budget => rsx! {
            BudgetTab {
              summary: BudgetSummary {
                  amount: agent.budget_monthly_cents,
                  observed_amount: agent.spent_monthly_cents,
                  remaining_amount: (agent.budget_monthly_cents - agent.spent_monthly_cents)
                      .max(0),
                  utilization_percent: if agent.budget_monthly_cents > 0 {
                      agent.spent_monthly_cents as f64 / agent.budget_monthly_cents as f64 * 100.0
                  } else {
                      0.0
                  },
                  warn_percent: 80,
                  hard_stop_enabled: true,
                  status: "ok".to_string(),
                  is_active: agent.budget_monthly_cents > 0,
              },
              on_save: move |_cents: i64| {},
            }
          },
      }
    }
  }
}

#[component]
fn AgentIconDisplay(icon: Option<String>) -> Element {
  let icon_char = icon.as_deref().unwrap_or("smart_toy");
  rsx! {
    div { class: "shrink-0 flex items-center justify-center h-12 w-12 rounded-lg bg-[var(--surface-container-high)]",
      span { class: "material-symbols-outlined text-xl", "{icon_char}" }
    }
  }
}
