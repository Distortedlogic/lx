use dioxus::prelude::*;

use super::new_dialog::NewGoalDialog;
use super::tree::GoalTree;
use crate::pages::projects::types::Goal;
use crate::styles::{FLEX_BETWEEN, PAGE_HEADING};

#[component]
pub fn Goals() -> Element {
  let goals = dioxus_storage::use_persistent("lx_goals", Vec::<Goal>::new);
  let mut show_dialog = use_signal(|| false);

  let goals_list = goals();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: FLEX_BETWEEN,
        h1 { class: PAGE_HEADING, "GOALS" }
        button {
          class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
          onclick: move |_| show_dialog.set(true),
          "NEW GOAL"
        }
      }
      if goals_list.is_empty() {
        div { class: "flex-1 flex items-center justify-center text-sm text-[var(--outline)]",
          "No goals yet"
        }
      } else {
        GoalTree { goals: goals_list }
      }
      if show_dialog() {
        NewGoalDialog { open: show_dialog, goals, parent_id: None::<String> }
      }
    }
  }
}
