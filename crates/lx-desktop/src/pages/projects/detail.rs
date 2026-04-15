use dioxus::prelude::*;
use dioxus::router::Navigator;

use super::types::{PROJECT_COLORS, PROJECT_STATUSES, Project};
use crate::components::page_skeleton::PageSkeleton;
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
pub fn ProjectDetail(project_id: String) -> Element {
  rsx! {
    SuspenseBoundary {
      fallback: |_| rsx! {
        PageSkeleton { variant: "detail".to_string() }
      },
      ProjectDetailInner { project_id }
    }
  }
}

#[component]
fn ProjectDetailInner(project_id: String) -> Element {
  let projects = dioxus_storage::use_persistent("lx_projects", Vec::<Project>::new);
  let mut active_tab = use_signal(|| "overview");
  let nav = use_navigator();

  let all = projects();
  let Some((idx, project)) = all.iter().enumerate().find(|(_, p)| p.id == project_id) else {
    return rsx! {
      div { class: "p-4 text-sm text-[var(--outline)]", "Project not found" }
    };
  };
  let project = project.clone();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex items-center gap-3",
        div {
          class: "w-3 h-3 rounded-full shrink-0",
          style: "background-color: {project.color}",
        }
        h2 { class: "page-heading", "{project.name}" }
        span {
          class: "text-xs uppercase font-semibold tracking-wider",
          class: "{status_color(&project.status)}",
          "{project.status}"
        }
      }
      div { class: "flex gap-1 border-b border-[var(--outline-variant)]/20 pb-0",
        button {
          class: if active_tab() == "overview" { "px-4 py-2 text-xs uppercase font-semibold border-b-2 border-[var(--primary)] text-[var(--on-surface)]" } else { "px-4 py-2 text-xs uppercase font-semibold text-[var(--outline)] hover:text-[var(--on-surface)]" },
          onclick: move |_| active_tab.set("overview"),
          "OVERVIEW"
        }
        button {
          class: if active_tab() == "configuration" { "px-4 py-2 text-xs uppercase font-semibold border-b-2 border-[var(--primary)] text-[var(--on-surface)]" } else { "px-4 py-2 text-xs uppercase font-semibold text-[var(--outline)] hover:text-[var(--on-surface)]" },
          onclick: move |_| active_tab.set("configuration"),
          "CONFIGURATION"
        }
      }
      if active_tab() == "overview" {
        div { class: "flex flex-col gap-4",
          p { class: "text-sm text-[var(--on-surface-variant)]",
            if let Some(ref desc) = project.description {
              "{desc}"
            } else {
              "No description"
            }
          }
          div { class: "grid grid-cols-2 gap-4",
            div { class: "flex flex-col gap-1",
              span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
                "STATUS"
              }
              span {
                class: "text-sm",
                class: "{status_color(&project.status)}",
                "{project.status}"
              }
            }
            div { class: "flex flex-col gap-1",
              span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
                "TARGET DATE"
              }
              span { class: "text-sm text-[var(--on-surface)]",
                if let Some(ref date) = project.target_date {
                  "{date}"
                } else {
                  "Not set"
                }
              }
            }
          }
        }
      } else {
        {render_configuration(projects, idx, &project, nav)}
      }
    }
  }
}

fn render_configuration(mut projects: Signal<Vec<Project>>, idx: usize, project: &Project, nav: Navigator) -> Element {
  let current_status = project.status.clone();
  let current_color = project.color.clone();
  let current_target = project.target_date.clone().unwrap_or_default();

  rsx! {
    div { class: "flex flex-col gap-4",
      div { class: "flex flex-col gap-1",
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
          "STATUS"
        }
        div { class: "flex gap-1 flex-wrap",
          for s in PROJECT_STATUSES.iter() {
            {
                let s_val = s.to_string();
                let active = current_status == *s;
                rsx! {
                  button {
                    class: if active { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--primary)] text-[var(--on-primary)]" } else { "px-2 py-1 text-[10px] uppercase font-semibold rounded bg-[var(--surface-container-lowest)] text-[var(--outline)] hover:text-[var(--on-surface)]" },
                    onclick: move |_| {
                        projects.write()[idx].status = s_val.clone();
                    },
                    "{s}"
                  }
                }
            }
          }
        }
      }
      div { class: "flex flex-col gap-1",
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
          "COLOR"
        }
        div { class: "flex gap-2 flex-wrap",
          for c in PROJECT_COLORS.iter() {
            {
                let c_val = c.to_string();
                let active = current_color == *c;
                let ring = if active { "ring-2 ring-white" } else { "" };
                rsx! {
                  button {
                    class: "w-6 h-6 rounded",
                    class: "{ring}",
                    style: "background-color: {c}",
                    onclick: move |_| {
                        projects.write()[idx].color = c_val.clone();
                    },
                  }
                }
            }
          }
        }
      }
      div { class: "flex flex-col gap-1",
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
          "TARGET DATE"
        }
        input {
          class: "bg-[var(--surface-container-lowest)] text-sm px-3 py-2 rounded outline-none text-[var(--on-surface)] w-fit",
          r#type: "date",
          value: "{current_target}",
          oninput: move |evt| {
              let val = evt.value();
              projects.write()[idx].target_date = if val.is_empty() {
                  None
              } else {
                  Some(val)
              };
          },
        }
      }
      button {
        class: "mt-4 px-4 py-2 text-xs uppercase font-semibold text-[var(--error)] border border-[var(--error)]/30 rounded hover:bg-[var(--error)]/10 w-fit",
        onclick: move |_| {
            projects.write()[idx].archived_at = Some("archived".to_string());
            nav.push(Route::Projects {});
        },
        "ARCHIVE PROJECT"
      }
    }
  }
}
