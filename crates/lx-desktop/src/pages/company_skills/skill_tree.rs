use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct SkillTreeNode {
  pub name: String,
  pub path: Option<String>,
  pub kind: SkillNodeKind,
  pub file_kind: Option<String>,
  pub children: Vec<SkillTreeNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SkillNodeKind {
  Dir,
}

fn file_icon(kind: Option<&str>) -> &'static str {
  match kind {
    Some("script" | "reference") => "code",
    _ => "description",
  }
}

#[component]
pub fn SkillTree(
  nodes: Vec<SkillTreeNode>,
  selected_path: String,
  expanded_dirs: HashSet<String>,
  on_toggle_dir: EventHandler<String>,
  on_select_path: EventHandler<String>,
  depth: Option<usize>,
) -> Element {
  let d = depth.unwrap_or(0);
  let base_indent = 16;
  let step_indent = 24;

  rsx! {
    div {
      for node in nodes.iter() {
        {
            let indent = base_indent + d * step_indent;
            if node.kind == SkillNodeKind::Dir {
                let expanded = node.path.as_ref().is_some_and(|p| expanded_dirs.contains(p));
                let dir_path = node.path.clone().unwrap_or_default();
                let dir_path2 = dir_path.clone();
                rsx! {
                  div { key: "{node.name}",
                    div { class: "group flex w-full items-center gap-1 pr-3 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] min-h-9",
                      button {
                        class: "flex min-w-0 items-center gap-2 py-1 text-left",
                        style: "padding-left: {indent}px",
                        onclick: move |_| on_toggle_dir.call(dir_path.clone()),
                        span { class: "material-symbols-outlined text-sm",
                          if expanded {
                            "folder_open"
                          }
                          button { "folder" }
                        }
                        span { class: "truncate", "{node.name}" }
                      }
                      button {
                        class: "ml-auto flex h-9 w-9 items-center justify-center",
                        onclick: move |_| on_toggle_dir.call(dir_path2.clone()),
                        span { class: "material-symbols-outlined text-sm",
                          if expanded {
                            "expand_more"
                          } else {
                            "chevron_right"
                          }
                        }
                      }
                    }
                    if expanded {
                      SkillTree {
                        nodes: node.children.clone(),
                        selected_path: selected_path.clone(),
                        expanded_dirs: expanded_dirs.clone(),
                        on_toggle_dir,
                        on_select_path,
                        depth: Some(d + 1),
                      }
                    }
                  }
                }
            } else {
                let is_selected = node.path.as_deref() == Some(selected_path.as_str());
                let file_path = node.path.clone().unwrap_or_default();
                let icon = file_icon(node.file_kind.as_deref());
                let sel_class = if is_selected {
                    " text-[var(--on-surface)] bg-[var(--surface-container)]/20"
                } else {
                    ""
                };
                rsx! {
                  div { key: "{node.name}",
                    button {
                      class: "flex w-full min-w-0 items-center gap-2 py-1 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] min-h-9",
                      class: "{sel_class}",
                      style: "padding-left: {indent}px",
                      onclick: move |_| on_select_path.call(file_path.clone()),
                      span { class: "material-symbols-outlined text-sm", "{icon}" }
                      span { class: "truncate", "{node.name}" }
                    }
                  }
                }
            }
        }
      }
    }
  }
}
