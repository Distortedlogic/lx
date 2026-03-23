use dioxus::prelude::*;

struct Quota {
  label: &'static str,
  percent: u8,
  color: &'static str,
  overload: bool,
  min_label: &'static str,
  max_label: &'static str,
}

const QUOTAS: &[Quota] = &[
  Quota { label: "COMPUTE_CORE", percent: 85, color: "bg-[var(--success)]", overload: false, min_label: "0.0 GHz/s", max_label: "12.4 GHz/s" },
  Quota { label: "MEMORY_BUFFER", percent: 32, color: "bg-[var(--warning)]", overload: false, min_label: "0.0 GB", max_label: "64.0 GB" },
  Quota { label: "STORAGE_IO", percent: 95, color: "bg-[var(--error)]", overload: true, min_label: "0.0 MB/s", max_label: "1.0 GB/s" },
];

#[component]
pub fn QuotasPanel() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4",
      div { class: "flex items-center gap-3 mb-4",
        div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
        span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]",
          "RESOURCE_QUOTAS"
        }
        div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
      }
      div { class: "flex gap-3",
        for quota in QUOTAS {
          {
              let pct = format!("{}%", quota.percent);
              let width = format!("width: {}%;", quota.percent);
              rsx! {
                div { class: "flex-1 bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-3",
                  div { class: "flex items-center justify-between mb-2",
                    span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]", "{quota.label}" }
                    if quota.overload {
                      span { class: "text-[10px] uppercase tracking-wider text-[var(--error)] font-semibold",
                        "OVERLOAD"
                      }
                    } else {
                      span { class: "text-sm font-semibold text-[var(--on-surface)]", "{pct}" }
                    }
                  }
                  div { class: "h-2 bg-[var(--surface-container)] rounded-full overflow-hidden mb-2",
                    div { class: "h-full {quota.color} rounded-full", style: "{width}" }
                  }
                  div { class: "flex justify-between text-[10px] text-[var(--outline)]",
                    span { "{quota.min_label}" }
                    span { "{quota.max_label}" }
                  }
                }
              }
          }
        }
      }
    }
  }
}
