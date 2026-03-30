use dioxus::prelude::*;

fn pseudo_random_height(i: usize) -> usize {
  (i * 17 + 5) % 80 + 10
}

#[component]
pub fn ChartCard(title: String, #[props(optional)] subtitle: Option<String>, children: Element) -> Element {
  rsx! {
    div { class: "border border-[var(--outline-variant)] rounded-lg p-4 space-y-3",
      div {
        h3 { class: "text-xs font-medium text-[var(--on-surface-variant)]",
          "{title}"
        }
        if let Some(ref sub) = subtitle {
          span { class: "text-[10px] text-[var(--outline)]", "{sub}" }
        }
      }
      {children}
    }
  }
}

#[component]
pub fn ActivitySummaryChart() -> Element {
  rsx! {
    div { class: "flex items-end gap-[3px] h-20",
      for i in 0..14 {
        div {
          class: "flex-1 bg-[var(--outline-variant)]/30 rounded-sm",
          style: "height: {pseudo_random_height(i)}%",
        }
      }
    }
  }
}

#[component]
pub fn EventBreakdownChart() -> Element {
  rsx! {
    div { class: "flex items-end gap-[3px] h-20",
      for i in 0..14 {
        div {
          class: "flex-1 bg-[var(--primary)]/30 rounded-sm",
          style: "height: {pseudo_random_height(i)}%",
        }
      }
    }
  }
}
