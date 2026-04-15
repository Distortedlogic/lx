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
pub fn GoalProperties(goal: Goal) -> Element {
  rsx! {
    div { class: "flex flex-col gap-3",
      div { class: "flex items-center gap-3",
        span { class: "w-20 text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold shrink-0",
          "Status"
        }
        span { class: "text-sm", class: "{status_color(&goal.status)}", "{goal.status}" }
      }
      div { class: "flex items-center gap-3",
        span { class: "w-20 text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold shrink-0",
          "Level"
        }
        span { class: "text-sm text-[var(--on-surface)] capitalize", "{goal.level}" }
      }
      div { class: "flex items-center gap-3",
        span { class: "w-20 text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold shrink-0",
          "Owner"
        }
        span { class: "text-sm text-[var(--outline)]", "None" }
      }
      div { class: "flex items-center gap-3",
        span { class: "w-20 text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold shrink-0",
          "Parent Goal"
        }
        if let Some(ref pid) = goal.parent_id {
          Link {
            to: Route::GoalDetail {
                goal_id: pid.clone(),
            },
            class: "text-sm text-[var(--primary)] hover:underline font-mono",
            "{pid}"
          }
        } else {
          span { class: "text-sm text-[var(--outline)]", "None" }
        }
      }
      div { class: "border-t border-[var(--outline-variant)]/20 my-1" }
      div { class: "flex items-center gap-3",
        span { class: "w-20 text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold shrink-0",
          "Created"
        }
        span { class: "text-sm text-[var(--on-surface-variant)]", "{goal.created_at}" }
      }
      div { class: "flex items-center gap-3",
        span { class: "w-20 text-[10px] uppercase tracking-wider text-[var(--outline)] font-semibold shrink-0",
          "Updated"
        }
        span { class: "text-sm text-[var(--on-surface-variant)]", "{goal.updated_at}" }
      }
    }
  }
}
