pub mod catalog;
mod controller;
pub mod inspector;
mod sample;
pub mod storage;
pub mod validation;
mod workspace;

use dioxus::prelude::*;

use self::controller::FlowEditorState;
use self::storage::{provide_flow_persistence, use_flow_persistence};
use self::workspace::FlowWorkspace;

#[component]
pub fn Flows() -> Element {
  let persistence = provide_flow_persistence();
  let flow_id = persistence.resolve_default_flow_id();
  rsx! {
    FlowWorkspacePage { flow_id }
  }
}

#[component]
pub fn FlowDetail(flow_id: String) -> Element {
  let _persistence = provide_flow_persistence();
  rsx! {
    FlowWorkspacePage { flow_id }
  }
}

#[component]
fn FlowWorkspacePage(flow_id: String) -> Element {
  let persistence = use_flow_persistence();
  let _state = FlowEditorState::provide(flow_id, &persistence);
  rsx! {
    FlowWorkspace {}
  }
}
