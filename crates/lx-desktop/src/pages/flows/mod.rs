mod controller;
mod sample;
mod workspace;

use dioxus::prelude::*;

use self::controller::FlowEditorState;
use self::sample::DEFAULT_FLOW_ID;
use self::workspace::FlowWorkspace;

#[component]
pub fn Flows() -> Element {
  rsx! {
    FlowWorkspacePage { flow_id: DEFAULT_FLOW_ID.to_string() }
  }
}

#[component]
pub fn FlowDetail(flow_id: String) -> Element {
  rsx! {
    FlowWorkspacePage { flow_id }
  }
}

#[component]
fn FlowWorkspacePage(flow_id: String) -> Element {
  let _state = FlowEditorState::provide(flow_id);
  rsx! {
    FlowWorkspace {}
  }
}
