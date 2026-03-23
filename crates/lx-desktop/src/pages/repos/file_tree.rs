use dioxus::prelude::*;

struct TreeEntry {
  name: &'static str,
  indent: u8,
  is_folder: bool,
}

const ENTRIES: &[TreeEntry] = &[
  TreeEntry { name: "src/core/rag_engine", indent: 0, is_folder: true },
  TreeEntry { name: "src/utils/ast_parser", indent: 0, is_folder: true },
  TreeEntry { name: "config.xml", indent: 0, is_folder: false },
  TreeEntry { name: "transformer.js", indent: 1, is_folder: false },
  TreeEntry { name: "analyzer.py", indent: 1, is_folder: false },
  TreeEntry { name: "docs/spec", indent: 0, is_folder: true },
];

#[component]
pub fn FileTree() -> Element {
  rsx! {
    div { class: "w-64 bg-[var(--surface-container-low)] border-r border-[var(--outline-variant)]/15 p-4 flex flex-col shrink-0",
      div { class: "flex items-center justify-between mb-4",
        span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
          "REPOSITORY HUB"
        }
        span { class: "text-xs text-[var(--outline)] cursor-pointer", "\u{25BC}" }
      }
      div { class: "flex flex-col gap-0.5",
        for entry in ENTRIES {
          {
              let pad = format!("padding-left: {}rem;", entry.indent as f32 * 1.0 + 0.5);
              let icon = if entry.is_folder { "\u{1F4C1}" } else { "\u{2192}" };
              let color = if entry.is_folder {
                  "text-[var(--primary)]"
              } else {
                  "text-[var(--on-surface-variant)]"
              };
              rsx! {
                div {
                  class: "flex items-center gap-2 py-1.5 px-2 text-xs rounded cursor-pointer hover:bg-[var(--surface-container-high)] transition-colors duration-150 {color}",
                  style: "{pad}",
                  span { "{icon}" }
                  span { "{entry.name}" }
                }
              }
          }
        }
      }
    }
  }
}
