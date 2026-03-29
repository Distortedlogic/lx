use crate::plugins::slots::{PluginSlotContext, PluginSlotMount, ResolvedPluginSlot};
use dioxus::prelude::*;

#[component]
pub fn PluginPage(plugin_id: String) -> Element {
  let page_slot: Option<ResolvedPluginSlot> = None;

  let context = PluginSlotContext { company_id: None, company_prefix: None, project_id: None, entity_id: None, entity_type: None };

  rsx! {
    div { class: "space-y-4 p-4",
      div { class: "flex items-center gap-2",
        button { class: "flex items-center gap-1 px-3 py-1.5 text-xs rounded hover:bg-[var(--surface-container)]",
          span { class: "material-symbols-outlined text-sm", "arrow_back" }
          "Back"
        }
      }
      if let Some(slot) = page_slot {
        PluginSlotMount { slot, context, show_placeholder: Some(true) }
      } else {
        div { class: "text-sm text-[var(--outline)]", "No page slot found for this plugin." }
      }
    }
  }
}
