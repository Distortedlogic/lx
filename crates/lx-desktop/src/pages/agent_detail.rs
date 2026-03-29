use dioxus::prelude::*;

#[component]
pub fn AgentDetail(agent_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Agent {agent_id} (stub)" }
  }
}
