mod ast_config;
mod chunks_panel;
mod file_tree;

use dioxus::prelude::*;

use self::ast_config::AstConfig;
use self::chunks_panel::ChunksPanel;
use self::file_tree::FileTree;

#[component]
pub fn Repos() -> Element {
  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "flex flex-1 min-h-0",
        FileTree {}
        AstConfig {}
        ChunksPanel {}
      }
      div { class: "p-4 border-t border-[var(--outline-variant)]/15",
        button { class: "w-full bg-[var(--success)] text-[var(--on-primary)] rounded py-3 text-sm uppercase tracking-wider font-semibold hover:brightness-110 transition-all duration-150",
          "\u{26A1} RUN ANALYSIS ENGINE"
        }
      }
    }
  }
}
