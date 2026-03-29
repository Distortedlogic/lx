use dioxus::prelude::*;

#[component]
pub fn Projects() -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Projects (stub)" }
  }
}

#[component]
pub fn ProjectDetail(project_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Project {project_id} (stub)" }
  }
}
