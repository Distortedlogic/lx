use dioxus::prelude::*;

use crate::contexts::panel::PanelState;

#[component]
pub fn PropertiesPanel() -> Element {
  let panel = use_context::<PanelState>();
  let content_id = (panel.content_id)();
  let visible = (panel.visible)();

  if content_id.is_none() {
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
          if let Some(ref id) = content_id {
            span { class: "text-sm text-gray-400", "Panel: {id}" }
          }
        }
      }
    }
  }
}
