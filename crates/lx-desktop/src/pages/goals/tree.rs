use dioxus::prelude::*;

use crate::pages::projects::types::Goal;
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
pub fn GoalTree(goals: Vec<Goal>) -> Element {
  let goal_ids: Vec<String> = goals.iter().map(|g| g.id.clone()).collect();
  let roots: Vec<&Goal> = goals
    .iter()
    .filter(|g| match &g.parent_id {
      None => true,
      Some(pid) => !goal_ids.contains(pid),
    })
    .collect();

  rsx! {
    div { class: "border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden",
      for root in roots {
        GoalNode { goal: root.clone(), all_goals: goals.clone(), depth: 0 }
      }
    }
  }
}

#[component]
fn GoalNode(goal: Goal, all_goals: Vec<Goal>, depth: u32) -> Element {
  let mut expanded = use_signal(|| true);
  let children: Vec<Goal> = all_goals.iter().filter(|g| g.parent_id.as_ref() == Some(&goal.id)).cloned().collect();
  let has_children = !children.is_empty();
  let pad = depth * 16 + 12;

  rsx! {
    Link {
      to: Route::GoalDetail {
          goal_id: goal.id.clone(),
      },
      class: "flex items-center gap-2 px-2 py-2 hover:bg-[var(--on-surface)]/5 transition-colors border-b border-[var(--outline-variant)]/20 last:border-b-0",
      style: "padding-left: {pad}px",
      if has_children {
        button {
          class: "text-[var(--outline)] hover:text-[var(--on-surface)] text-sm shrink-0 w-4 flex items-center justify-center transition-transform",
          onclick: move |evt| {
              evt.prevent_default();
              evt.stop_propagation();
              expanded.set(!expanded());
          },
          span { class: if expanded() { "material-symbols-outlined text-sm rotate-90 transition-transform" } else { "material-symbols-outlined text-sm transition-transform" },
            "chevron_right"
          }
        }
      } else {
        span { class: "w-4 shrink-0" }
      }
      span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold shrink-0 w-14",
        "{goal.level}"
      }
      span { class: "flex-1 text-sm text-[var(--on-surface)] truncate", "{goal.title}" }
      span { class: "text-[10px] uppercase font-semibold tracking-wider shrink-0 {status_color(&goal.status)}",
        "{goal.status}"
      }
    }
    if expanded() && has_children {
      for child in children.iter() {
        GoalNode {
          goal: child.clone(),
          all_goals: all_goals.clone(),
          depth: depth + 1,
        }
      }
    }
  }
}
