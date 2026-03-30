pub mod active_agents_panel;
pub mod activity_charts;

use dioxus::prelude::*;

use crate::components::empty_state::EmptyState;
use crate::components::metric_card::MetricCard;
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::breadcrumb::BreadcrumbEntry;

use self::active_agents_panel::ActiveAgentsPanel;
use self::activity_charts::{ActivitySummaryChart, ChartCard, EventBreakdownChart};

#[component]
pub fn Dashboard() -> Element {
  let breadcrumb_state = use_context::<crate::contexts::breadcrumb::BreadcrumbState>();
  use_effect(move || {
    breadcrumb_state.set(vec![BreadcrumbEntry { label: "Dashboard".into(), href: None }]);
  });

  let log = use_context::<ActivityLog>();
  let events = log.events.read();

  let total_events = events.len();
  let agent_events = events.iter().filter(|e| e.kind.contains("agent")).count();
  let tool_events = events.iter().filter(|e| e.kind.contains("tool")).count();
  let error_events = events.iter().filter(|e| e.kind.to_lowercase().contains("error") || e.message.to_lowercase().contains("error")).count();

  if total_events == 0 {
    return rsx! {
      EmptyState {
        icon: "dashboard",
        message: "No activity recorded yet. Run an agent to see metrics here.",
      }
    };
  }

  rsx! {
    div { class: "space-y-6",
      ActiveAgentsPanel {}

      div { class: "grid grid-cols-2 xl:grid-cols-4 gap-2",
        MetricCard {
          icon: "pulse_alert",
          value: "{total_events}",
          label: "Total Events",
        }
        MetricCard {
          icon: "smart_toy",
          value: "{agent_events}",
          label: "Agent Events",
        }
        MetricCard {
          icon: "build",
          value: "{tool_events}",
          label: "Tool Events",
        }
        MetricCard { icon: "error", value: "{error_events}", label: "Errors" }
      }

      div { class: "grid grid-cols-2 lg:grid-cols-4 gap-4",
        ChartCard { title: "Activity", subtitle: "Last 14 events".to_string(),
          ActivitySummaryChart {}
        }
        ChartCard { title: "Event Breakdown", subtitle: "By type".to_string(),
          EventBreakdownChart {}
        }
      }

      div { class: "min-w-0",
        h3 { class: "text-sm font-semibold text-[var(--on-surface-variant)] uppercase tracking-wide mb-3",
          "Recent Activity"
        }
        div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",
          for event in events.iter().take(10) {
            div { class: "px-4 py-2.5 text-sm hover:bg-[var(--on-surface)]/5 transition-colors",
              div { class: "flex gap-3",
                p { class: "flex-1 min-w-0 truncate",
                  span { class: "text-[var(--on-surface-variant)] font-mono text-xs",
                    "{event.kind}"
                  }
                  span { class: "ml-2", "{event.message}" }
                }
                span { class: "text-xs text-[var(--outline)] shrink-0",
                  "{event.timestamp}"
                }
              }
            }
          }
        }
      }
    }
  }
}

#[component]
pub fn DashboardAlt() -> Element {
  rsx! {
    Dashboard {}
  }
}
