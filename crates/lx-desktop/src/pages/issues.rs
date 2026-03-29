use dioxus::prelude::*;

#[component]
pub fn Issues() -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Issues (stub)" }
  }
}

#[component]
pub fn IssueDetail(issue_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Issue {issue_id} (stub)" }
  }
}
