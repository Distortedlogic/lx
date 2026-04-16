pub mod catalog;
mod connectors;
mod controller;
mod credentials;
pub mod inspector;
mod registry;
mod runtime_bar;
mod sample;
pub mod storage;
pub mod validation;
mod workspace;

use dioxus::prelude::*;

use self::controller::FlowEditorState;
use self::runtime_bar::FlowRuntimeBar;
use self::storage::provide_flow_persistence;
use self::workspace::FlowWorkspace;

#[component]
pub fn Flows() -> Element {
  rsx! {
    div { class: "flex flex-col h-full min-h-0",
      FlowRuntimeBar {}
      FlowWorkspace {}
    }
  }
}

#[component]
pub fn FlowDetail(flow_id: String) -> Element {
  let _ = flow_id;
  rsx! {
    div { class: "flex flex-col h-full min-h-0",
      FlowRuntimeBar {}
      FlowWorkspace {}
    }
  }
}

#[component]
pub fn FlowRouteScope(flow_id: Option<String>, children: Element) -> Element {
  let persistence = provide_flow_persistence();
  let panel = use_context::<crate::contexts::panel::PanelState>();
  let effective_flow_id = flow_id.unwrap_or_else(|| persistence.resolve_default_flow_id());
  let _state = FlowEditorState::provide(effective_flow_id, &persistence);

  use_drop(move || {
    panel.close();
  });

  rsx! {
    {children}
  }
}
