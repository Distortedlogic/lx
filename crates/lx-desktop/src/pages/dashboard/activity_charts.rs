use dioxus::prelude::*;

#[component]
pub fn ChartCard(title: String, #[props(optional)] subtitle: Option<String>, children: Element) -> Element {
  rsx! {
    div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4 space-y-3",
      div {
        h3 { class: "text-xs font-medium text-[var(--outline)]", "{title}" }
        if let Some(ref sub) = subtitle {
          span { class: "text-[10px] text-[var(--outline)]/60", "{sub}" }
        }
      }
      {children}
    }
  }
}

#[component]
pub fn ActivitySummaryChart(buckets: Vec<usize>) -> Element {
  let max_val = buckets.iter().copied().max().unwrap_or(1).max(1);
  rsx! {
    div { class: "flex items-end gap-[3px] h-20",
      for (i, count) in buckets.iter().enumerate() {
        {
            let pct = (*count as f64 / max_val as f64 * 100.0) as usize;
            let pct = pct.max(2);
            rsx! {
              div {
                key: "{i}",
                class: "flex-1 bg-[var(--surface-container-high)]/30 rounded-sm",
                style: "height: {pct}%",
              }
            }
        }
      }
    }
  }
}

#[component]
pub fn EventBreakdownChart(segments: Vec<(String, usize)>) -> Element {
  let max_val = segments.iter().map(|(_, c)| *c).max().unwrap_or(1).max(1);
  let colors = ["bg-[var(--success)]/30", "bg-[var(--tertiary)]/30", "bg-[var(--warning)]/30", "bg-[var(--error)]/30", "bg-violet-700/30"];
  rsx! {
    div { class: "flex items-end gap-[3px] h-20",
      for (i, (_label, count)) in segments.iter().enumerate() {
        {
            let pct = (*count as f64 / max_val as f64 * 100.0) as usize;
            let pct = pct.max(2);
            let color = colors[i % colors.len()];
            rsx! {
              div {
                key: "{i}",
                class: "flex-1 rounded-sm relative group",
                class: "{color}",
                style: "height: {pct}%",
                div { class: "absolute -top-5 left-1/2 -translate-x-1/2 text-[9px] text-[var(--outline)] opacity-0 group-hover:opacity-100 whitespace-nowrap",
                  "{_label}: {count}"
                }
              }
            }
        }
      }
    }
    if !segments.is_empty() {
      div { class: "flex flex-wrap gap-x-3 gap-y-1 mt-2",
        for (i, (label, count)) in segments.iter().enumerate() {
          div { class: "flex items-center gap-1",
            span {
              class: "w-2 h-2 rounded-sm",
              class: "{colors[i % colors.len()]}",
            }
            span { class: "text-[10px] text-[var(--outline)]", "{label} ({count})" }
          }
        }
      }
    }
  }
}
