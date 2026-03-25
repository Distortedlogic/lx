use super::state::{AnalysisMode, ReposState};
use dioxus::prelude::*;

#[component]
pub fn AstConfig() -> Element {
  let repos = use_context::<ReposState>();
  let current_mode = (repos.mode)();
  let depth = (repos.tree_depth)();
  let selected = (repos.selected_file)();
  let results = repos.results.read();
  let node_count = results.as_ref().map(|r| r.total_tokens).unwrap_or(0);

  let file_content = use_resource(move || {
    let sel = selected.clone();
    async move {
      match sel {
        Some(path) if !path.is_empty() => tokio::fs::read_to_string(&path).await.ok(),
        _ => None,
      }
    }
  });

  let modes = [AnalysisMode::Syntactic, AnalysisMode::Semantic, AnalysisMode::Hybrid];

  rsx! {
    div { class: "flex-1 flex flex-col p-4 overflow-auto min-w-0",
      div { class: "flex items-center justify-between mb-4",
        span { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
          "AST CONFIGURATION & ANALYSIS"
        }
        span { class: "text-xs text-[var(--outline)] uppercase tracking-wider",
          "TOKEN_EST: {node_count}"
        }
      }
      span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
        "ANALYSIS MODE"
      }
      div { class: "flex gap-0 mb-4",
        for mode in modes {
          {
              let is_active = mode == current_mode;
              let cls = if is_active {
                  "bg-[var(--primary)] text-[var(--on-primary)] px-6 py-2 text-xs uppercase tracking-wider font-semibold cursor-pointer"
              } else {
                  "bg-[var(--surface-container)] text-[var(--outline)] px-6 py-2 text-xs uppercase tracking-wider border border-[var(--outline-variant)]/30 cursor-pointer hover:text-[var(--on-surface)] transition-colors duration-150"
              };
              rsx! {
                span {
                  class: "{cls}",
                  onclick: move |_| {
                      let mut mode_sig = repos.mode;
                      mode_sig.set(mode);
                  },
                  "{mode}"
                }
              }
          }
        }
      }
      div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 mb-4",
        div { class: "flex items-center justify-between mb-2",
          span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
            "TREE_DEPTH"
          }
          span { class: "text-[var(--warning)]", "{depth:.0}" }
        }
        input {
          r#type: "range",
          min: "1",
          max: "10",
          step: "1",
          value: "{depth}",
          class: "w-full accent-[var(--primary)] mb-3",
          oninput: move |evt| {
              if let Ok(v) = evt.value().parse::<f64>() {
                  let mut depth_sig = repos.tree_depth;
                  depth_sig.set(v);
              }
          },
        }
      }
      div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex-1 overflow-auto",
        p { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)] mb-3",
          "FILE_PREVIEW"
        }
        match &*file_content.value().read() {
            Some(Some(content)) => rsx! {
              pre { class: "text-xs font-mono text-[var(--on-surface-variant)] whitespace-pre leading-relaxed max-h-64 overflow-auto",
                "{content}"
              }
            },
            _ => rsx! {
              p { class: "text-xs text-[var(--outline)]", "Select a file to preview" }
            },
        }
      }
    }
  }
}
