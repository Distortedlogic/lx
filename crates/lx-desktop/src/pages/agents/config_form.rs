use super::types::{ADAPTER_LABELS, AgentDetail};
use crate::components::ui::select::{Select, SelectOption};
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct ConfigUpdate {
  pub adapter_type: String,
  pub model: String,
  pub heartbeat_enabled: bool,
  pub heartbeat_interval_sec: u32,
}

#[component]
pub fn AgentConfigPanel(
  agent: AgentDetail,
  #[props(optional)] on_save: Option<EventHandler<ConfigUpdate>>,
  #[props(optional)] on_cancel: Option<EventHandler<()>>,
) -> Element {
  let original_adapter = agent.adapter_type.clone();
  let original_model = agent.adapter_config.get("model").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let original_hb_enabled = agent.runtime_config.get("heartbeat").and_then(|v| v.get("enabled")).and_then(|v| v.as_bool()).unwrap_or(false);
  let original_interval = agent.runtime_config.get("heartbeat").and_then(|v| v.get("intervalSec")).and_then(|v| v.as_u64()).unwrap_or(300) as u32;

  let mut adapter_type = use_signal(|| original_adapter.clone());
  let mut model = use_signal(|| original_model.clone());
  let mut heartbeat_enabled = use_signal(|| original_hb_enabled);
  let mut interval_sec = use_signal(|| original_interval);
  let mut dirty = use_signal(|| false);

  rsx! {
    div { class: "max-w-3xl space-y-6",
      ConfigSection { title: "Adapter",
        div { class: "space-y-3",
          label { class: "text-xs text-[var(--outline)] block", "Adapter type" }
          Select {
            value: adapter_type.read().clone(),
            options: ADAPTER_LABELS.iter().map(|(k, l)| SelectOption::new(*k, *l)).collect::<Vec<_>>(),
            onchange: move |val: String| {
                adapter_type.set(val);
                dirty.set(true);
            },
          }
          label { class: "text-xs text-[var(--outline)] block", "Model" }
          input {
            class: INPUT_FIELD,
            value: "{model}",
            placeholder: "e.g. claude-sonnet-4-20250514",
            oninput: move |evt| {
                model.set(evt.value().to_string());
                dirty.set(true);
            },
          }
        }
      }
      ConfigSection { title: "Heartbeat",
        div { class: "space-y-3",
          div { class: "flex items-center justify-between",
            span { class: "text-sm text-[var(--on-surface)]", "Enabled" }
            ToggleSwitch {
              checked: *heartbeat_enabled.read(),
              on_toggle: move |v: bool| {
                  heartbeat_enabled.set(v);
                  dirty.set(true);
              },
            }
          }
          if *heartbeat_enabled.read() {
            div {
              label { class: "text-xs text-[var(--outline)] block mb-1",
                "Interval (seconds)"
              }
              input {
                class: INPUT_FIELD,
                r#type: "number",
                value: "{interval_sec}",
                oninput: move |evt| {
                    if let Ok(v) = evt.value().parse::<u32>() {
                        interval_sec.set(v);
                        dirty.set(true);
                    }
                },
              }
            }
          }
        }
      }
      if *dirty.read() {
        div { class: "flex items-center justify-end gap-2 pt-4 border-t border-[var(--outline-variant)]/30",
          button {
            class: BTN_OUTLINE_SM,
            onclick: move |_| {
                adapter_type.set(original_adapter.clone());
                model.set(original_model.clone());
                heartbeat_enabled.set(original_hb_enabled);
                interval_sec.set(original_interval);
                dirty.set(false);
                if let Some(ref handler) = on_cancel {
                    handler.call(());
                }
            },
            "Cancel"
          }
          button {
            class: BTN_PRIMARY_SM,
            onclick: move |_| {
                let update = ConfigUpdate {
                    adapter_type: adapter_type.read().clone(),
                    model: model.read().clone(),
                    heartbeat_enabled: *heartbeat_enabled.read(),
                    heartbeat_interval_sec: *interval_sec.read(),
                };
                dirty.set(false);
                if let Some(ref handler) = on_save {
                    handler.call(update);
                }
            },
            "Save"
          }
        }
      }
    }
  }
}

#[component]
fn ConfigSection(title: &'static str, children: Element) -> Element {
  rsx! {
    div { class: "border border-[var(--outline-variant)]/30 rounded-lg",
      div { class: "px-4 py-3 border-b border-[var(--outline-variant)]/30",
        h3 { class: "text-sm font-medium text-[var(--on-surface)]", "{title}" }
      }
      div { class: "px-4 py-4", {children} }
    }
  }
}

#[component]
fn ToggleSwitch(checked: bool, on_toggle: EventHandler<bool>) -> Element {
  let bg = if checked { "bg-[var(--success)]" } else { "bg-[var(--outline-variant)]" };
  let translate = if checked { "translate-x-4" } else { "translate-x-0.5" };
  rsx! {
    button {
      class: "relative inline-flex h-5 w-9 items-center rounded-full transition-colors shrink-0 {bg}",
      onclick: move |_| on_toggle.call(!checked),
      span { class: "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform {translate}" }
    }
  }
}
