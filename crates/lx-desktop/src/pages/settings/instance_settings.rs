use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct HeartbeatAgent {
  id: String,
  agent_name: String,
  company_id: String,
  company_name: String,
  title: String,
  interval_sec: u32,
  scheduler_active: bool,
  heartbeat_enabled: bool,
  last_heartbeat_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct HeartbeatGroup {
  company_name: String,
  agents: Vec<HeartbeatAgent>,
}

fn group_agents(agents: &[HeartbeatAgent]) -> Vec<HeartbeatGroup> {
  let mut map: std::collections::BTreeMap<String, Vec<HeartbeatAgent>> = std::collections::BTreeMap::new();
  for agent in agents {
    map.entry(agent.company_name.clone()).or_default().push(agent.clone());
  }
  map.into_iter().map(|(company_name, agents)| HeartbeatGroup { company_name, agents }).collect()
}

#[component]
pub fn InstanceHeartbeats() -> Element {
  let agents: Vec<HeartbeatAgent> = vec![];
  let grouped = group_agents(&agents);
  let active_count = agents.iter().filter(|a| a.scheduler_active).count();
  let disabled_count = agents.len() - active_count;
  let enabled_count = agents.iter().filter(|a| a.heartbeat_enabled).count();

  rsx! {
    div { class: "max-w-5xl space-y-6 p-4 overflow-auto",
      div { class: "space-y-2",
        div { class: "flex items-center gap-2",
          span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
            "settings"
          }
          h1 { class: "text-lg font-semibold text-[var(--on-surface)]",
            "Scheduler Heartbeats"
          }
        }
        p { class: "text-sm text-[var(--outline)]",
          "Agents with a timer heartbeat enabled across all companies."
        }
      }
      div { class: "flex items-center gap-4 text-sm text-[var(--outline)]",
        span {
          span { class: "font-semibold text-[var(--on-surface)]", "{active_count}" }
          " active"
        }
        span {
          span { class: "font-semibold text-[var(--on-surface)]", "{disabled_count}" }
          " disabled"
        }
        span {
          span { class: "font-semibold text-[var(--on-surface)]", "{grouped.len()}" }
          if grouped.len() == 1 {
            " company"
          } else {
            " companies"
          }
        }
        if enabled_count > 0 {
          button { class: "ml-auto bg-red-600 text-white rounded px-3 py-1 text-xs font-semibold",
            "Disable All"
          }
        }
      }
      if agents.is_empty() {
        div { class: "flex flex-col items-center justify-center py-16 text-[var(--outline)]",
          span { class: "material-symbols-outlined text-xl mb-4", "schedule" }
          p { class: "text-sm", "No scheduler heartbeats." }
        }
      } else {
        for group in grouped.iter() {
          div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)]",
            div { class: "border-b px-3 py-2 text-xs font-semibold uppercase tracking-wide text-[var(--outline)]",
              "{group.company_name}"
            }
            for agent in group.agents.iter() {
              div { class: "flex items-center gap-3 px-3 py-2 text-sm border-b border-[var(--outline-variant)]/30 last:border-b-0",
                span { class: if agent.scheduler_active { "shrink-0 text-[10px] px-1.5 py-0 rounded border border-[var(--primary)] text-[var(--primary)]" } else { "shrink-0 text-[10px] px-1.5 py-0 rounded border border-[var(--outline-variant)] text-[var(--outline)]" },
                  if agent.scheduler_active {
                    "On"
                  } else {
                    "Off"
                  }
                }
                span { class: "font-medium truncate", "{agent.agent_name}" }
                span { class: "text-[var(--outline)] truncate", "{agent.title}" }
                span { class: "text-[var(--outline)] tabular-nums shrink-0",
                  "{agent.interval_sec}s"
                }
                span { class: "text-[var(--outline)] truncate",
                  if let Some(ref ts) = agent.last_heartbeat_at {
                    "{ts}"
                  } else {
                    "never"
                  }
                }
                button { class: "ml-auto text-xs px-2 py-1 rounded hover:bg-[var(--surface-container)]",
                  if agent.heartbeat_enabled {
                    "Disable Timer Heartbeat"
                  } else {
                    "Enable Timer Heartbeat"
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
