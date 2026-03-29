use dioxus::prelude::*;

#[component]
pub fn Goals() -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Goals (stub)" }
  }
}

#[component]
pub fn GoalDetail(goal_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Goal {goal_id} (stub)" }
  }
}
