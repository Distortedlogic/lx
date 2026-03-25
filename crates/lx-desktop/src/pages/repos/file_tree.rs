use super::state::{ReposState, read_dir_tree};
use dioxus::prelude::*;

#[component]
pub fn FileTree() -> Element {
  let repos = use_context::<ReposState>();
  let root = (repos.root_path)();
  let depth = (repos.tree_depth)() as u8;
  let selected = (repos.selected_file)();

  let tree = use_resource(move || {
    let root = root.clone();
    async move { read_dir_tree(&root, depth).await }
  });

  rsx! {
    div { class: "w-64 bg-[var(--surface-container-low)] border-r border-[var(--outline-variant)]/15 p-4 flex flex-col shrink-0 overflow-auto",
      div { class: "flex items-center justify-between mb-4",
        span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]",
          "REPOSITORY HUB"
        }
      }
      match &*tree.value().read() {
          Some(nodes) => rsx! {
            div { class: "flex flex-col gap-0.5",
              for node in nodes.iter() {
                {
                    let path = node.path.clone();
                    let is_selected = selected.as_deref() == Some(path.as_str());
                    let pad = format!("padding-left: {}rem;", node.depth as f32 * 1.0 + 0.5);
                    let icon = if node.is_dir { "\u{1F4C1}" } else { "\u{2192}" };
                    let color = if node.is_dir {
                        "text-[var(--primary)]"
                    } else {
                        "text-[var(--on-surface-variant)]"
                    };
                    let bg = if is_selected { " bg-[var(--surface-container-high)]" } else { "" };
                    rsx! {
                      div {
                        class: "flex items-center gap-2 py-1.5 px-2 text-xs rounded cursor-pointer hover:bg-[var(--surface-container-high)] transition-colors duration-150 {color}{bg}",
                        style: "{pad}",
                        onclick: move |_| {
                            let mut selected_file = repos.selected_file;
                            selected_file.set(Some(path.clone()));
                        },
                        span { "{icon}" }
                        span { "{node.name}" }
                      }
                    }
                }
              }
            }
          },
          None => rsx! {
            div { class: "text-xs text-[var(--outline)] py-4 text-center", "Loading..." }
          },
      }
    }
  }
}
