use super::state::{SettingsDataStoreExt, SettingsState};
use dioxus::prelude::*;

#[component]
pub fn TaskPriorityPanel() -> Element {
  let settings = use_context::<SettingsState>();
  let priority = settings.data.task_priority().cloned();
  let auto_scale = settings.data.auto_scale().cloned();
  let redundant_verify = settings.data.redundant_verify().cloned();
  let priority_display = format!("{priority:.2}");

  rsx! {
    div { class: "bg-[var(--surface-container-low)] border-2 border-[var(--outline-variant)] p-6",
      span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--warning)] mb-4",
        "TASK_PRIORITY"
      }
      div { class: "flex items-center justify-between mb-2",
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]",
          "WEIGHTING_INDEX"
        }
        span { class: "text-sm font-semibold text-[var(--on-surface)]", "{priority_display}" }
      }
      input {
        r#type: "range",
        min: "0",
        max: "1",
        step: "0.01",
        value: "{priority}",
        class: "w-full accent-[var(--warning)] mb-3",
        oninput: move |evt| {
            if let Ok(v) = evt.value().parse::<f64>() {
                settings.data.task_priority().set(v);
            }
        },
      }
      div { class: "flex justify-between text-[10px] text-[var(--outline)] mb-4",
        span { "LOW_LATENCY" }
        span { "HIGH_THROUGHPUT" }
      }
      div { class: "flex flex-col gap-2",
        label { class: "flex items-center gap-2 text-xs text-[var(--on-surface-variant)] cursor-pointer",
          input {
            r#type: "checkbox",
            checked: auto_scale,
            class: "w-4 h-4 accent-[var(--warning)]",
            onchange: move |_| {
                let current = settings.data.auto_scale().cloned();
                settings.data.auto_scale().set(!current);
            },
          }
          "AUTO-SCALE_RESOURCES"
        }
        label { class: "flex items-center gap-2 text-xs text-[var(--on-surface-variant)] cursor-pointer",
          input {
            r#type: "checkbox",
            checked: redundant_verify,
            class: "w-4 h-4 accent-[var(--warning)]",
            onchange: move |_| {
                let current = settings.data.redundant_verify().cloned();
                settings.data.redundant_verify().set(!current);
            },
          }
          "REDUNDANT_VERIFICATION"
        }
      }
    }
  }
}
