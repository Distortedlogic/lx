use crate::components::file_tree::{FileNodeKind, FileTree, build_file_tree, count_files};
use dioxus::prelude::*;
use std::collections::HashSet;

#[component]
pub fn CompanyExport() -> Element {
  let mut search_query = use_signal(String::new);
  let mut selected_file: Signal<Option<String>> = use_signal(|| None);
  let mut expanded_dirs: Signal<HashSet<String>> = use_signal(HashSet::new);
  let mut checked_files: Signal<HashSet<String>> = use_signal(HashSet::new);

  let demo_files: Vec<String> = vec![];
  let tree = build_file_tree(&demo_files, None);
  let total_files = count_files(&tree);
  let checked_count = checked_files().len();

  rsx! {
    div { class: "flex flex-col h-full",
      div { class: "flex items-center gap-2 px-4 py-3 border-b border-[var(--outline-variant)]",
        span { class: "material-symbols-outlined text-[var(--outline)]", "inventory_2" }
        h1 { class: "text-lg font-semibold text-[var(--on-surface)]", "Export Company Package" }
      }
      div { class: "flex flex-1 overflow-hidden",
        div { class: "w-80 border-r border-[var(--outline-variant)] flex flex-col",
          div { class: "px-3 py-2 border-b border-[var(--outline-variant)]",
            div { class: "flex items-center gap-2",
              span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                "search"
              }
              input {
                class: "flex-1 bg-transparent text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
                placeholder: "Search files...",
                value: "{search_query}",
                oninput: move |evt| search_query.set(evt.value()),
              }
            }
          }
          div { class: "px-3 py-1.5 text-xs text-[var(--outline)] border-b border-[var(--outline-variant)]",
            "{checked_count} / {total_files} files selected"
          }
          div { class: "flex-1 overflow-auto",
            FileTree {
              nodes: tree,
              selected_file: selected_file(),
              expanded_dirs: expanded_dirs(),
              checked_files: Some(checked_files()),
              on_toggle_dir: move |path: String| {
                  let mut dirs = expanded_dirs();
                  if dirs.contains(&path) {
                      dirs.remove(&path);
                  } else {
                      dirs.insert(path);
                  }
                  expanded_dirs.set(dirs);
              },
              on_select_file: move |path: String| {
                  selected_file.set(Some(path));
              },
              on_toggle_check: Some(
                  EventHandler::new(move |(path, kind): (String, FileNodeKind)| {
                      let mut files = checked_files();
                      if kind == FileNodeKind::File {
                          if files.contains(&path) {
                              files.remove(&path);
                          } else {
                              files.insert(path);
                          }
                      }
                      checked_files.set(files);
                  }),
              ),
            }
          }
          div { class: "px-3 py-2 border-t border-[var(--outline-variant)]",
            button {
              class: "w-full flex items-center justify-center gap-2 bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-2 text-sm font-semibold",
              disabled: checked_count == 0,
              span { class: "material-symbols-outlined text-sm", "download" }
              "Export Package"
            }
          }
        }
        div { class: "flex-1 overflow-auto",
          if selected_file().is_some() {
            div { class: "p-5",
              div { class: "border-b border-[var(--outline-variant)] pb-3 mb-4",
                span { class: "font-mono text-sm text-[var(--on-surface)]",
                  "{selected_file().unwrap_or_default()}"
                }
              }
              p { class: "text-sm text-[var(--outline)]",
                "File preview content would appear here."
              }
            }
          } else {
            div { class: "flex flex-col items-center justify-center h-full text-[var(--outline)]",
              span { class: "material-symbols-outlined text-4xl mb-4",
                "inventory_2"
              }
              p { class: "text-sm", "Select a file to preview its contents." }
            }
          }
        }
      }
    }
  }
}
