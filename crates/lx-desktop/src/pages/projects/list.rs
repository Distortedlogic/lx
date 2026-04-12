use dioxus::prelude::*;

use super::new_dialog::NewProjectDialog;
use super::types::Project;
use crate::routes::Route;
fn status_color(status: &str) -> &'static str {
  match status {
    "in_progress" => "text-[var(--primary)]",
    "completed" => "text-[var(--success)]",
    "cancelled" => "text-[var(--error)]",
    "planned" => "text-[var(--warning)]",
    _ => "text-[var(--outline)]",
  }
}

#[component]
pub fn Projects() -> Element {
  let projects = dioxus_storage::use_persistent("lx_projects", Vec::<Project>::new);
  let mut show_dialog = use_signal(|| false);

  let active: Vec<Project> = projects().into_iter().filter(|p| p.archived_at.is_none()).collect();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex-between",
        h1 { class: "page-heading", "PROJECTS" }
        button {
          class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
          onclick: move |_| show_dialog.set(true),
          "ADD PROJECT"
        }
      }
      if active.is_empty() {
        div { class: "flex-1 flex items-center justify-center text-sm text-[var(--outline)]",
          "No projects yet"
        }
      } else {
        div { class: "flex flex-col border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden",
          for project in active.iter() {
            Link {
              to: Route::ProjectDetail {
                  project_id: project.id.clone(),
              },
              class: "flex items-center gap-3 px-4 py-3 hover:bg-white/5 transition-colors border-b border-[var(--outline-variant)]/20 last:border-b-0",
              div {
                class: "w-[5px] h-[5px] rounded-full shrink-0",
                style: "background-color: {project.color}",
              }
              span { class: "font-semibold text-sm text-[var(--on-surface)]",
                "{project.name}"
              }
              if let Some(ref desc) = project.description {
                span { class: "text-xs text-[var(--outline)] truncate flex-1",
                  "{desc}"
                }
              }
              if let Some(ref date) = project.target_date {
                span { class: "text-xs text-[var(--outline)] shrink-0",
                  "{date}"
                }
              }
              span { class: "text-[10px] uppercase font-semibold tracking-wider shrink-0 {status_color(&project.status)}",
                "{project.status}"
              }
            }
          }
        }
      }
      if show_dialog() {
        NewProjectDialog { open: show_dialog, projects }
      }
    }
  }
}
