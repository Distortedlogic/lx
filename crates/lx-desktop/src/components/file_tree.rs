use dioxus::prelude::*;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct FileTreeNode {
  pub name: String,
  pub path: String,
  pub kind: FileNodeKind,
  pub children: Vec<FileTreeNode>,
  pub action: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileNodeKind {
  Dir,
  File,
}

pub fn build_file_tree(files: &[String], action_map: Option<&std::collections::HashMap<String, String>>) -> Vec<FileTreeNode> {
  let mut root = FileTreeNode { name: String::new(), path: String::new(), kind: FileNodeKind::Dir, children: vec![], action: None };

  for file_path in files {
    let segments: Vec<&str> = file_path.split('/').filter(|s| !s.is_empty()).collect();
    let mut current = &mut root;
    let mut current_path = String::new();

    for (i, segment) in segments.iter().enumerate() {
      if !current_path.is_empty() {
        current_path.push('/');
      }
      current_path.push_str(segment);
      let is_leaf = i == segments.len() - 1;

      let pos = current.children.iter().position(|c| c.name == *segment);
      if let Some(idx) = pos {
        current = &mut current.children[idx];
      } else {
        let node = FileTreeNode {
          name: segment.to_string(),
          path: current_path.clone(),
          kind: if is_leaf { FileNodeKind::File } else { FileNodeKind::Dir },
          children: vec![],
          action: if is_leaf { action_map.and_then(|m| m.get(file_path).cloned()) } else { None },
        };
        current.children.push(node);
        let last = current.children.len() - 1;
        current = &mut current.children[last];
      }
    }
  }

  fn sort_node(node: &mut FileTreeNode) {
    node.children.sort_by(|a, b| {
      if a.kind != b.kind {
        return if a.kind == FileNodeKind::File { std::cmp::Ordering::Less } else { std::cmp::Ordering::Greater };
      }
      a.name.cmp(&b.name)
    });
    for child in &mut node.children {
      sort_node(child);
    }
  }

  sort_node(&mut root);
  root.children
}

pub fn collect_file_paths(nodes: &[FileTreeNode]) -> HashSet<String> {
  let mut paths = HashSet::new();
  for node in nodes {
    if node.kind == FileNodeKind::File {
      paths.insert(node.path.clone());
    }
    paths.extend(collect_file_paths(&node.children));
  }
  paths
}

pub fn count_files(nodes: &[FileTreeNode]) -> usize {
  let mut count = 0;
  for node in nodes {
    if node.kind == FileNodeKind::File {
      count += 1;
    } else {
      count += count_files(&node.children);
    }
  }
  count
}

fn file_icon(name: &str) -> &'static str {
  if name.ends_with(".yaml") || name.ends_with(".yml") { "code" } else { "description" }
}

#[component]
pub fn FileTree(
  nodes: Vec<FileTreeNode>,
  selected_file: Option<String>,
  expanded_dirs: HashSet<String>,
  checked_files: Option<HashSet<String>>,
  on_toggle_dir: EventHandler<String>,
  on_select_file: EventHandler<String>,
  on_toggle_check: Option<EventHandler<(String, FileNodeKind)>>,
  show_checkboxes: Option<bool>,
  depth: Option<usize>,
) -> Element {
  let show_cb = show_checkboxes.unwrap_or(true);
  let d = depth.unwrap_or(0);
  let effective_checked = checked_files.unwrap_or_default();
  let base_indent = 16;
  let step_indent = 24;

  rsx! {
    div {
      for node in nodes.iter() {
        {
            let indent = base_indent + d * step_indent;

            if node.kind == FileNodeKind::Dir {
                let expanded = expanded_dirs.contains(&node.path);
                let child_files = collect_file_paths(&node.children);
                let all_checked = child_files.iter().all(|p| effective_checked.contains(p));
                let dir_path = node.path.clone();
                let dir_path2 = node.path.clone();
                let dir_path3 = node.path.clone();
                rsx! {
                  div { key: "{node.path}",
                    div {
                      class: "group flex w-full items-center gap-1 pr-3 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] min-h-9",
                      style: "padding-left: {indent}px",
                      if show_cb {
                        label { class: "flex items-center pl-2",
                          input {
                            r#type: "checkbox",
                            checked: all_checked,
                            onchange: move |_| {
                                if let Some(ref handler) = on_toggle_check {
                                    handler.call((dir_path.clone(), FileNodeKind::Dir));
                                }
                            },
                            class: "mr-2",
                          }
                        }
                      }
                      button {
                        class: "flex min-w-0 items-center gap-2 py-1 text-left",
                        onclick: move |_| on_toggle_dir.call(dir_path2.clone()),
                        span { class: "material-symbols-outlined text-sm",
                          if expanded {
                            "folder_open"
                          } else {
                            "folder"
                          }
                        }
                        span { class: "truncate", "{node.name}" }
                      }
                      button {
                        class: "ml-auto flex h-9 w-9 items-center justify-center rounded-sm text-[var(--outline)] hover:bg-[var(--surface-container)]",
                        onclick: move |_| on_toggle_dir.call(dir_path3.clone()),
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
                      FileTree {
                        nodes: node.children.clone(),
                        selected_file: selected_file.clone(),
                        expanded_dirs: expanded_dirs.clone(),
                        checked_files: Some(effective_checked.clone()),
                        on_toggle_dir,
                        on_select_file,
                        on_toggle_check,
                        show_checkboxes: Some(show_cb),
                        depth: Some(d + 1),
                      }
                    }
                  }
                }
            } else {
                let checked = effective_checked.contains(&node.path);
                let is_selected = selected_file.as_deref() == Some(&node.path);
                let file_path = node.path.clone();
                let file_path2 = node.path.clone();
                let icon_name = file_icon(&node.name);
                let sel_class = if is_selected {
                    "text-[var(--on-surface)] bg-[var(--surface-container)]/20"
                } else {
                    ""
                };
                rsx! {
                  div { key: "{node.path}",
                    div {
                      class: "flex w-full items-center gap-1 pr-3 text-left text-sm text-[var(--outline)] hover:bg-[var(--surface-container)]/30 hover:text-[var(--on-surface)] cursor-pointer min-h-9",
                      class: "{sel_class}",
                      style: "padding-left: {indent}px",
                      onclick: move |_| on_select_file.call(file_path.clone()),
                      if show_cb {
                        label { class: "flex items-center pl-2",
                          input {
                            r#type: "checkbox",
                            checked,
                            onchange: move |_| {
                                if let Some(ref handler) = on_toggle_check {
                                    handler.call((file_path2.clone(), FileNodeKind::File));
                                }
                            },
                            class: "mr-2",
                          }
                        }
                      }
                      span { class: "material-symbols-outlined text-sm", "{icon_name}" }
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
