mod new_skill_form;
mod skill_tree;

use self::new_skill_form::{NewSkillForm, NewSkillPayload};
use self::skill_tree::{SkillTree, SkillTreeNode};
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
struct SkillListItem {
  id: String,
  name: String,
  slug: String,
  description: Option<String>,
  source_badge: String,
}

#[component]
pub fn CompanySkills() -> Element {
  let mut selected_skill_id: Signal<Option<String>> = use_signal(|| None);
  let mut show_new_form = use_signal(|| false);
  let mut search_query = use_signal(String::new);
  let mut selected_file = use_signal(|| "SKILL.md".to_string());
  let mut expanded_dirs: Signal<std::collections::HashSet<String>> = use_signal(std::collections::HashSet::new);

  let skills: Vec<SkillListItem> = vec![];
  let tree_nodes: Vec<SkillTreeNode> = vec![];

  rsx! {
    div { class: "flex h-full",
      div { class: "w-72 border-r border-[var(--outline-variant)] flex flex-col",
        div { class: "flex items-center justify-between px-3 py-2 border-b border-[var(--outline-variant)]",
          div { class: "flex items-center gap-2",
            span { class: "material-symbols-outlined text-[var(--outline)]",
              "widgets"
            }
            h1 { class: "text-base font-semibold text-[var(--on-surface)]",
              "Skills"
            }
          }
          button {
            class: "p-1 rounded hover:bg-[var(--surface-container)]",
            onclick: move |_| {
                let current = show_new_form();
                show_new_form.set(!current);
            },
            span { class: "material-symbols-outlined text-sm", "add" }
          }
        }
        div { class: "px-3 py-2 border-b border-[var(--outline-variant)]",
          div { class: "flex items-center gap-2",
            span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
              "search"
            }
            input {
              class: "flex-1 bg-transparent text-sm outline-none text-[var(--on-surface)] placeholder-[var(--outline)]",
              placeholder: "Search skills...",
              value: "{search_query}",
              oninput: move |evt| search_query.set(evt.value()),
            }
          }
        }
        if show_new_form() {
          NewSkillForm {
            on_create: move |_payload: NewSkillPayload| {
                show_new_form.set(false);
            },
            on_cancel: move |_| show_new_form.set(false),
            is_pending: false,
          }
        }
        div { class: "flex-1 overflow-auto",
          if skills.is_empty() {
            div { class: "flex flex-col items-center justify-center py-12 text-[var(--outline)]",
              span { class: "material-symbols-outlined text-3xl mb-3",
                "widgets"
              }
              p { class: "text-xs", "No skills yet." }
            }
          }
          for skill in skills.iter() {
            {
                let skill_id = skill.id.clone();
                let is_selected = selected_skill_id() == Some(skill.id.clone());
                let bg = if is_selected { " bg-[var(--surface-container)]" } else { "" };
                rsx! {
                  button {
                    class: "w-full text-left px-3 py-2 border-b border-[var(--outline-variant)]/30 hover:bg-[var(--surface-container)]{bg}",
                    onclick: move |_| selected_skill_id.set(Some(skill_id.clone())),
                    div { class: "text-sm font-medium text-[var(--on-surface)]", "{skill.name}" }
                    if let Some(ref desc) = skill.description {
                      p { class: "text-xs text-[var(--outline)] mt-0.5 truncate", "{desc}" }
                    }
                    div { class: "text-[10px] text-[var(--outline)] mt-0.5", "{skill.source_badge}" }
                  }
                }
            }
          }
        }
      }
      div { class: "flex-1 flex",
        if selected_skill_id().is_some() {
          div { class: "w-56 border-r border-[var(--outline-variant)] overflow-auto",
            SkillTree {
              nodes: tree_nodes.clone(),
              selected_path: selected_file(),
              expanded_dirs: expanded_dirs(),
              on_toggle_dir: move |path: String| {
                  let mut dirs = expanded_dirs();
                  if dirs.contains(&path) {
                      dirs.remove(&path);
                  } else {
                      dirs.insert(path);
                  }
                  expanded_dirs.set(dirs);
              },
              on_select_path: move |path: String| {
                  selected_file.set(path);
              },
            }
          }
          div { class: "flex-1 overflow-auto p-5",
            div { class: "border-b border-[var(--outline-variant)] pb-3 mb-4",
              span { class: "font-mono text-sm text-[var(--on-surface)]",
                "{selected_file}"
              }
            }
            p { class: "text-sm text-[var(--outline)]", "File content would appear here." }
          }
        } else {
          div { class: "flex-1 flex flex-col items-center justify-center text-[var(--outline)]",
            span { class: "material-symbols-outlined text-4xl mb-4", "widgets" }
            p { class: "text-sm", "Select a skill to view its files." }
          }
        }
      }
    }
  }
}
