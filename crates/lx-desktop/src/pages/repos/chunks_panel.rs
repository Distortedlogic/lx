use super::state::ReposState;
use dioxus::prelude::*;

#[component]
pub fn ChunksPanel() -> Element {
  let repos = use_context::<ReposState>();
  let results = repos.results.read();

  rsx! {
    div { class: "w-72 bg-[var(--surface-container-low)] border-l border-[var(--outline-variant)]/15 p-4 flex flex-col shrink-0 overflow-auto",
      div { class: "flex items-center justify-between mb-4",
        span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
          "EXTRACTED CHUNKS"
        }
      }
      match results.as_ref() {
          Some(analysis) => rsx! {
            div { class: "flex flex-col gap-3 flex-1",
              for chunk in analysis.chunks.iter() {
                {
                    let score_color = if chunk.score > 0.5 {
                        "bg-[var(--success)]"
                    } else if chunk.score > 0.2 {
                        "bg-[var(--warning)]"
                    } else {
                        "bg-[var(--error)]"
                    };
                    let score_text = format!("{:.3}", chunk.score);
                    rsx! {
                      div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-3",
                        div { class: "flex items-center justify-between mb-2",
                          span { class: "text-xs text-[var(--primary)] font-semibold", "{chunk.id}" }
                          span { class: "{score_color} text-[var(--on-primary)] text-[10px] px-2 py-0.5 rounded font-semibold",
                            "{score_text}"
                          }
                        }
                        p { class: "text-[10px] text-[var(--on-surface-variant)] leading-relaxed mb-2",
                          "{chunk.description}"
                        }
                        if !chunk.tags.is_empty() {
                          div { class: "flex gap-1",
                            for tag in chunk.tags.iter() {
                              span { class: "bg-[var(--surface-container-high)] text-[10px] text-[var(--outline)] px-2 py-0.5 rounded uppercase tracking-wider",
                                "{tag}"
                              }
                            }
                          }
                        }
                      }
                    }
                }
              }
            }
            div { class: "mt-4 pt-3 border-t border-[var(--outline-variant)]/15",
              p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
                "ANALYSIS STATS"
              }
              p { class: "text-xs text-[var(--on-surface-variant)]", "TOTAL_TOKENS: {analysis.total_tokens}" }
              p { class: "text-xs text-[var(--on-surface-variant)]", "LATENCY: {analysis.latency_ms}ms" }
            }
          },
          None => rsx! {
            div { class: "flex-1 flex items-center justify-center",
              p { class: "text-xs text-[var(--outline)] text-center", "Run analysis to see results" }
            }
          },
      }
    }
  }
}
