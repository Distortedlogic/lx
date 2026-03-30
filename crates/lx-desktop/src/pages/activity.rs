use dioxus::prelude::*;

use crate::components::empty_state::EmptyState;
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
        select {
          class: "h-8 rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container)] px-2 py-1 text-xs focus:outline-none focus:ring-1 focus:ring-[var(--primary)]",
          value: "{current_filter}",
          onchange: move |evt: Event<FormData>| filter.set(evt.value()),
          option { value: "all", "All types" }
          for kind in entity_types.iter() {
            option { value: "{kind}", "{kind}" }
          }
        }
      }

      if filtered.is_empty() {
        EmptyState { icon: "history", message: "No activity recorded yet." }
      } else {
        div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)] overflow-hidden",
          for event in filtered.iter() {
            div { class: "flex items-center px-4 py-2.5 hover:bg-[var(--on-surface)]/5 transition-colors text-sm",
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
