use dioxus::prelude::*;

use crate::contexts::panel::{PanelContent, PanelState};
use crate::pages::flows::inspector::FlowInspector;

#[component]
pub fn PropertiesPanel() -> Element {
  let panel = use_context::<PanelState>();
  let content = (panel.content)();
  let visible = (panel.visible)();

  if content.is_none() {
    return rsx! {};
  }

  let width = if visible { "320px" } else { "0px" };
  let opacity = if visible { "1" } else { "0" };
  let style = format!("width: {width}; opacity: {opacity};");

  rsx! {
    aside {
      class: "border-l border-gray-700/50 bg-[var(--surface-container)] flex-col shrink-0 overflow-hidden transition-[width,opacity] duration-200 ease-in-out",
      style: "{style}",
      div { class: "w-80 flex-1 flex flex-col min-w-[320px]",
        div { class: "flex items-center justify-between px-4 py-2 border-b border-gray-700/50",
          span { class: "text-sm font-medium", "Properties" }
          button {
            class: "p-1 hover:bg-white/10 rounded transition-colors",
            onclick: move |_| panel.set_visible(false),
            span { class: "material-symbols-outlined text-sm", "close" }
          }
        }
        div { class: "flex-1 overflow-y-auto p-4",
          if let Some(content) = content {
            match content {
                PanelContent::FlowNode { node_id } => rsx! {
                  FlowInspector { content: PanelContent::FlowNode { node_id } }
                },
                PanelContent::FlowEdge { edge_id } => rsx! {
                  FlowInspector { content: PanelContent::FlowEdge { edge_id } }
                },
            }
          }
        }
      }
    }
  }
}
