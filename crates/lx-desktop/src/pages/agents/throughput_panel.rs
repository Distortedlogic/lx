use dioxus::prelude::*;

#[component]
pub fn ThroughputPanel() -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container)] rounded-lg p-4",
      p { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)] mb-3", "SYSTEM THROUGHPUT" }
      div { class: "flex flex-col gap-3",
        ProgressBar { label: "GLOBAL MEMORY", value: "4.8 / 16 GB", percent: 30.0 }
        ProgressBar { label: "COMPUTE LOAD", value: "62%", percent: 62.0 }
      }
      div { class: "flex gap-3 mt-4",
        StatTile { label: "TOKENS/SEC", value: "142.4" }
        StatTile { label: "ACTIVE TASKS", value: "09" }
      }
    }
  }
}

#[component]
fn ProgressBar(label: &'static str, value: &'static str, percent: f64) -> Element {
  let width = format!("{percent}%");
  rsx! {
    div {
      div { class: "flex justify-between text-xs mb-1",
        span { class: "text-[var(--outline)] uppercase tracking-wider", "{label}" }
        span { class: "text-[var(--on-surface-variant)]", "{value}" }
      }
      div { class: "h-1.5 bg-[var(--surface-container-low)] rounded-full overflow-hidden",
        div { class: "h-full bg-[var(--primary)] rounded-full", style: "width: {width};" }
      }
    }
  }
}

#[component]
fn StatTile(label: &'static str, value: &'static str) -> Element {
  rsx! {
    div { class: "flex-1 bg-[var(--surface-container-low)] rounded p-3 text-center",
      p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1", "{label}" }
      p { class: "text-2xl font-bold text-[var(--on-surface)]", "{value}" }
    }
  }
}
