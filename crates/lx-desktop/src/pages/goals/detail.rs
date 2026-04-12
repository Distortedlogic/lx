use dioxus::prelude::*;

use super::properties::GoalProperties;
use super::tree::GoalTree;
use crate::components::page_skeleton::PageSkeleton;
use crate::pages::projects::types::{Goal, Project};
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
pub fn GoalDetail(goal_id: String) -> Element {
  rsx! {
    SuspenseBoundary {
      fallback: |_| rsx! { PageSkeleton { variant: "detail".to_string() } },
      GoalDetailInner { goal_id }
    }
  }
}

#[component]
fn GoalDetailInner(goal_id: String) -> Element {
  let goals = dioxus_storage::use_persistent("lx_goals", Vec::<Goal>::new);
  let projects = dioxus_storage::use_persistent("lx_projects", Vec::<Project>::new);
  let mut active_tab = use_signal(|| "children");

  let all_goals = goals();
  let all_projects = projects();

  let Some(goal) = all_goals.iter().find(|g| g.id == goal_id) else {
    return rsx! {
      div { class: "p-4 text-sm text-[var(--outline)]", "Goal not found" }
    };
  };
  let goal = goal.clone();

  let children: Vec<Goal> = all_goals.iter().filter(|g| g.parent_id.as_ref() == Some(&goal_id)).cloned().collect();
  let child_count = children.len();

  let linked_projects: Vec<Project> = all_projects.iter().filter(|p| p.goal_ids.contains(&goal_id)).cloned().collect();
  let project_count = linked_projects.len();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex items-center gap-3",
        span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold",
          "{goal.level}"
        }
        span { class: "text-xs uppercase font-semibold tracking-wider {status_color(&goal.status)}",
          "{goal.status}"
        }
      }
      h2 { class: "page-heading", "{goal.title}" }
      p { class: "text-sm text-[var(--on-surface-variant)]",
        if let Some(ref desc) = goal.description {
          "{desc}"
        } else {
          "No description"
        }
      }
      GoalProperties { goal: goal.clone() }
      div { class: "flex gap-1 border-b border-[var(--outline-variant)]/20 pb-0",
        button {
          class: if active_tab() == "children" { "px-4 py-2 text-xs uppercase font-semibold border-b-2 border-[var(--primary)] text-[var(--on-surface)]" } else { "px-4 py-2 text-xs uppercase font-semibold text-[var(--outline)] hover:text-[var(--on-surface)]" },
          onclick: move |_| active_tab.set("children"),
          "SUB-GOALS ({child_count})"
        }
        button {
          class: if active_tab() == "projects" { "px-4 py-2 text-xs uppercase font-semibold border-b-2 border-[var(--primary)] text-[var(--on-surface)]" } else { "px-4 py-2 text-xs uppercase font-semibold text-[var(--outline)] hover:text-[var(--on-surface)]" },
          onclick: move |_| active_tab.set("projects"),
          "PROJECTS ({project_count})"
        }
      }
      if active_tab() == "children" {
        if children.is_empty() {
          div { class: "text-sm text-[var(--outline)] py-4", "No sub-goals" }
        } else {
          GoalTree { goals: children }
        }
      } else {
        if linked_projects.is_empty() {
          div { class: "text-sm text-[var(--outline)] py-4", "No linked projects" }
        } else {
          div { class: "flex flex-col border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden",
            for proj in linked_projects.iter() {
              Link {
                to: Route::ProjectDetail {
                    project_id: proj.id.clone(),
                },
                class: "flex items-center gap-3 px-4 py-3 hover:bg-white/5 transition-colors border-b border-[var(--outline-variant)]/20 last:border-b-0",
                span { class: "font-semibold text-sm text-[var(--on-surface)]",
                  "{proj.name}"
                }
                if let Some(ref desc) = proj.description {
                  span { class: "text-xs text-[var(--outline)] truncate flex-1",
                    "{desc}"
                  }
                }
                span { class: "text-[10px] uppercase font-semibold tracking-wider shrink-0 {status_color(&proj.status)}",
                  "{proj.status}"
                }
              }
            }
          }
        }
      }
    }
  }
}
