use dioxus::prelude::*;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, PartialEq)]
pub struct PluginSlotContext {
  pub company_id: Option<String>,
  pub company_prefix: Option<String>,
  pub project_id: Option<String>,
  pub entity_id: Option<String>,
  pub entity_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginSlotDeclaration {
  pub id: String,
  pub slot_type: String,
  pub display_name: String,
  pub export_name: String,
  pub order: Option<i32>,
  pub entity_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPluginSlot {
  pub id: String,
  pub slot_type: String,
  pub display_name: String,
  pub export_name: String,
  pub order: Option<i32>,
  pub entity_types: Vec<String>,
  pub plugin_id: String,
  pub plugin_key: String,
  pub plugin_display_name: String,
  pub plugin_version: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginUiContribution {
  pub plugin_id: String,
  pub plugin_key: String,
  pub display_name: String,
  pub version: String,
  pub ui_entry_file: String,
  pub slots: Vec<PluginSlotDeclaration>,
  pub updated_at: Option<String>,
}

type ComponentRenderFn = fn(ResolvedPluginSlot, PluginSlotContext) -> Element;

static COMPONENT_REGISTRY: OnceLock<Mutex<HashMap<String, ComponentRenderFn>>> = OnceLock::new();

fn component_registry() -> &'static Mutex<HashMap<String, ComponentRenderFn>> {
  COMPONENT_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

fn registry_key(plugin_key: &str, export_name: &str) -> String {
  format!("{plugin_key}:{export_name}")
}

pub fn register_plugin_component(plugin_key: &str, export_name: &str, render_fn: ComponentRenderFn) {
  let key = registry_key(plugin_key, export_name);
  if let Ok(mut registry) = component_registry().lock() {
    registry.insert(key, render_fn);
  }
}

pub fn resolve_plugin_component(plugin_key: &str, export_name: &str) -> Option<ComponentRenderFn> {
  let key = registry_key(plugin_key, export_name);
  component_registry().lock().ok()?.get(&key).copied()
}

pub fn resolve_slots(contributions: &[PluginUiContribution], slot_types: &[&str], entity_type: Option<&str>) -> Vec<ResolvedPluginSlot> {
  let allowed: std::collections::HashSet<&str> = slot_types.iter().copied().collect();
  let entity_scoped = ["detailTab", "taskDetailView", "contextMenuItem"];

  let mut rows: Vec<ResolvedPluginSlot> = Vec::new();
  for contribution in contributions {
    for slot in &contribution.slots {
      if !allowed.contains(slot.slot_type.as_str()) {
        continue;
      }
      if entity_scoped.contains(&slot.slot_type.as_str()) {
        let Some(et) = entity_type else {
          continue;
        };
        if !slot.entity_types.iter().any(|t| t == et) {
          continue;
        }
      }
      rows.push(ResolvedPluginSlot {
        id: slot.id.clone(),
        slot_type: slot.slot_type.clone(),
        display_name: slot.display_name.clone(),
        export_name: slot.export_name.clone(),
        order: slot.order,
        entity_types: slot.entity_types.clone(),
        plugin_id: contribution.plugin_id.clone(),
        plugin_key: contribution.plugin_key.clone(),
        plugin_display_name: contribution.display_name.clone(),
        plugin_version: contribution.version.clone(),
      });
    }
  }
  rows.sort_by(|a, b| {
    let ao = a.order.unwrap_or(i32::MAX);
    let bo = b.order.unwrap_or(i32::MAX);
    ao.cmp(&bo).then_with(|| a.plugin_display_name.cmp(&b.plugin_display_name)).then_with(|| a.display_name.cmp(&b.display_name))
  });
  rows
}

#[component]
pub fn PluginSlotMount(slot: ResolvedPluginSlot, context: PluginSlotContext, show_placeholder: Option<bool>) -> Element {
  let component = resolve_plugin_component(&slot.plugin_key, &slot.export_name);
  match component {
    Some(render_fn) => render_fn(slot, context),
    None => {
      if show_placeholder.unwrap_or(false) {
        rsx! {
          div { class: "rounded-md border border-dashed border-[var(--outline-variant)] px-2 py-1 text-xs text-[var(--outline)]",
            "{slot.plugin_display_name}: {slot.display_name}"
          }
        }
      } else {
        rsx! {}
      }
    },
  }
}

#[component]
pub fn PluginSlotOutlet(slot_types: Vec<String>, context: PluginSlotContext, entity_type: Option<String>) -> Element {
  let contributions: Vec<PluginUiContribution> = vec![];
  let type_refs: Vec<&str> = slot_types.iter().map(|s| s.as_str()).collect();
  let slots = resolve_slots(&contributions, &type_refs, entity_type.as_deref());

  if slots.is_empty() {
    return rsx! {};
  }

  rsx! {
    div {
      for slot in slots.iter() {
        PluginSlotMount {
          key: "{slot.plugin_key}:{slot.id}",
          slot: slot.clone(),
          context: context.clone(),
        }
      }
    }
  }
}
