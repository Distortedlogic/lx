use dioxus::prelude::*;

use crate::components::empty_state::EmptyState;
use crate::components::ui::select::{Select, SelectOption};
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::breadcrumb::BreadcrumbEntry;

#[component]
pub fn Activity() -> Element {
  let breadcrumb_state = use_context::<crate::contexts::breadcrumb::BreadcrumbState>();
  use_effect(move || {
    breadcrumb_state.set(vec![BreadcrumbEntry { label: "Activity".into(), href: None }]);
  });

  let log = use_context::<ActivityLog>();
  let events = log.events.read();
  let mut filter = use_signal(|| "all".to_string());

  let mut entity_types: Vec<String> = events.iter().map(|e| e.kind.clone()).collect();
  entity_types.sort();
  entity_types.dedup();

  let current_filter = filter();
  let filtered: Vec<_> = if current_filter == "all" { events.iter().collect() } else { events.iter().filter(|e| e.kind == current_filter).collect() };

  rsx! {
    div { class: "space-y-4",
      div { class: "flex items-center justify-end",
        Select {
          class: "h-8 bg-[var(--surface-container)] px-2 py-1 text-xs".to_string(),
          value: current_filter.clone(),
          options: {
              let mut opts = vec![SelectOption::new("all", "All types")];
              opts.extend(entity_types.iter().map(|k| SelectOption::new(k.as_str(), k.as_str())));
              opts
          },
          onchange: move |val: String| filter.set(val),
        }
      }

      if filtered.is_empty() {
        EmptyState { icon: "history", message: "No activity recorded yet." }
      } else {
        div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",
          for event in filtered.iter() {
            div {
              key: "{event.timestamp}-{event.kind}-{event.message}",
              class: "flex items-center px-4 py-2.5 hover:bg-[var(--on-surface)]/5 transition-colors text-sm animate-activity-enter",
              span { class: "w-40 shrink-0 text-[var(--outline)] font-mono text-xs",
                "{event.timestamp}"
              }
              span { class: "w-28 shrink-0 text-[var(--primary)] uppercase font-semibold text-xs",
                "{event.kind}"
              }
              span { class: "flex-1 text-[var(--on-surface)] truncate",
                "{event.message}"
              }
            }
          }
        }
      }
    }
  }
}
