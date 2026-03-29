use dioxus::prelude::*;

#[component]
pub fn Routines() -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Routines (stub)" }
  }
}

#[component]
pub fn RoutineDetail(routine_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Routine {routine_id} (stub)" }
  }
}
