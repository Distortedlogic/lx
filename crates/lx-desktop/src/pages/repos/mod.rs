mod ast_config;
mod chunks_panel;
mod file_tree;
mod state;

use self::ast_config::AstConfig;
use self::chunks_panel::ChunksPanel;
use self::file_tree::FileTree;
use self::state::{ReposState, run_analysis};
use dioxus::prelude::*;

#[component]
pub fn Repos() -> Element {
  let repos = ReposState::provide();
  let analyzing = (repos.analyzing)();

  rsx! {
    div { class: "flex h-full",
      FileTree {}
      div { class: "flex-1 flex flex-col min-h-0 min-w-0",
        AstConfig {}
        div { class: "p-4 border-t border-[var(--outline-variant)]/15",
          button {
            class: "w-full bg-[var(--success)] text-[var(--on-primary)] rounded py-3 text-sm uppercase tracking-wider font-semibold hover:brightness-110 transition-all duration-150",
            disabled: analyzing,
            onclick: move |_| {
                let root = (repos.root_path)();
                spawn(async move {
                    let mut analyzing_sig = repos.analyzing;
                    analyzing_sig.set(true);
                    let results = run_analysis(&root).await;
                    let mut results_sig = repos.results;
                    results_sig.set(Some(results));
                    analyzing_sig.set(false);
                });
            },
            if analyzing {
              "\u{23F3} ANALYZING..."
            } else {
              "\u{26A1} RUN ANALYSIS ENGINE"
            }
          }
        }
      }
      ChunksPanel {}
    }
  }
}
