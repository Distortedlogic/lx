use super::state::SettingsState;
use dioxus::prelude::*;

struct QuotaDisplay {
  label: &'static str,
  percent: u8,
  min_label: &'static str,
  max_label: &'static str,
}

#[component]
pub fn QuotasPanel() -> Element {
  let settings = use_context::<SettingsState>();
  let data = settings.data.read();
  let quotas = [
    QuotaDisplay { label: "COMPUTE_CORE", percent: data.compute_quota, min_label: "0.0 GHz/s", max_label: "12.4 GHz/s" },
    QuotaDisplay { label: "MEMORY_BUFFER", percent: data.memory_quota, min_label: "0.0 GB", max_label: "64.0 GB" },
    QuotaDisplay { label: "STORAGE_IO", percent: data.storage_quota, min_label: "0.0 MB/s", max_label: "1.0 GB/s" },
  ];
  drop(data);

  rsx! {
    div { class: "space-y-4",
      span { class: "text-xs uppercase tracking-wider font-semibold text-white border-l-4 border-[var(--warning)] pl-3",
        "RESOURCE_QUOTAS"
      }
      div { class: "flex gap-3",
        for quota in quotas.iter() {
          {
              let pct_str = format!("{}%", quota.percent);
              let width = format!("width: {}%;", quota.percent);
              let color = if quota.percent > 90 {
                  "bg-[var(--error)]"
              } else if quota.percent > 70 {
                  "bg-[var(--warning)]"
              } else {
                  "bg-[var(--primary)]"
              };
              let overload = quota.percent > 90;
              rsx! {
                div { class: "flex-1 bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-4",
                  div { class: "flex items-center justify-between mb-2",
                    span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]", "{quota.label}" }
                    if overload {
                      span { class: "text-[10px] uppercase tracking-wider text-[var(--error)] font-semibold",
                        "OVERLOAD"
                      }
                    } else {
                      span { class: "text-sm font-semibold text-[var(--on-surface)]", "{pct_str}" }
                    }
                  }
                  div { class: "h-2 bg-[var(--surface-container)] rounded-full overflow-hidden mb-2",
                    div {
                      class: "h-full rounded-full",
                      class: "{color}",
                      style: "{width}",
                    }
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
