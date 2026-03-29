use dioxus::prelude::*;

#[component]
pub fn Approvals() -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Approvals (stub)" }
  }
}

#[component]
pub fn ApprovalDetail(approval_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Approval {approval_id} (stub)" }
  }
}
