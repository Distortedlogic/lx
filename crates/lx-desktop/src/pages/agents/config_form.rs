use super::types::{ADAPTER_LABELS, LxAgentConfig};
use crate::components::ui::select::{Select, SelectOption};
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};
use dioxus::prelude::*;

const MODEL_OPTIONS: &[(&str, &str)] = &[
  ("claude-sonnet-4-20250514", "Claude Sonnet 4"),
  ("claude-opus-4-20250514", "Claude Opus 4"),
  ("claude-haiku-3-5-20241022", "Claude Haiku 3.5"),
  ("o4-mini", "o4-mini"),
  ("o3", "o3"),
  ("gemini-2.5-pro", "Gemini 2.5 Pro"),
  ("gemini-2.5-flash", "Gemini 2.5 Flash"),
  ("gpt-4.1", "GPT-4.1"),
  ("gpt-4.1-mini", "GPT-4.1 Mini"),
];

#[derive(Clone, Debug, PartialEq)]
pub struct AgentConfigUpdate {
  pub adapter_type: String,
  pub model: String,
}

#[component]
pub fn AgentConfigPanel(config: LxAgentConfig, #[props(optional)] on_save: Option<EventHandler<AgentConfigUpdate>>) -> Element {
  let mut adapter_type = use_signal(|| config.adapter_type.clone());
  let mut model = use_signal(|| config.model.clone());
  let mut dirty = use_signal(|| false);

  rsx! {
    div { class: "max-w-3xl space-y-6",
      ConfigSection { title: "Source Definition",
        div { class: "relative",
          pre { class: "text-xs font-mono leading-relaxed text-[var(--on-surface)] bg-[var(--surface)] border border-[var(--outline-variant)]/30 rounded p-4 overflow-x-auto max-h-80 overflow-y-auto whitespace-pre",
            "{config.source_text}"
          }
          button {
            class: "absolute top-2 right-2 text-xs text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
            title: "Copy source",
            onclick: {
                let source = config.source_text.clone();
                move |_| {
                    let escaped = source
                        .replace('\\', "\\\\")
                        .replace('\'', "\\'")
                        .replace('\n', "\\n");
                    spawn(async move {
                        let _ = document::eval(
                                &format!("navigator.clipboard.writeText('{escaped}')"),
                            )
                            .await;
                    });
                }
            },
            span { class: "material-symbols-outlined text-sm", "content_copy" }
          }
        }
      }
      ConfigSection { title: "Model & Backend",
        div { class: "space-y-3",
          label { class: "text-xs text-[var(--outline)] block", "Adapter" }
          select {
            class: INPUT_FIELD,
            value: "{adapter_type}",
            onchange: move |evt| {
                adapter_type.set(evt.value().to_string());
                dirty.set(true);
            },
            for (key , label) in ADAPTER_LABELS {
              option { value: *key, "{label}" }
            }
          }
          label { class: "text-xs text-[var(--outline)] block", "Model" }
          Select {
            class: "w-full".to_string(),
            value: model.read().clone(),
            searchable: true,
            placeholder: "Select a model...".to_string(),
            options: {
                let cur = model.read().clone();
                let mut opts: Vec<SelectOption> = MODEL_OPTIONS
                    .iter()
                    .map(|(v, l)| SelectOption::new(*v, *l))
                    .collect();
                if !cur.is_empty() && !opts.iter().any(|o| o.value == cur) {
                    opts.insert(0, SelectOption::new(cur.clone(), cur));
                }
                opts
            },
            onchange: move |val: String| {
                model.set(val);
                dirty.set(true);
            },
          }
        }
      }
      ConfigSection { title: "Tools",
        if config.tools.is_empty() {
          div { class: "text-sm text-[var(--outline)] italic", "No tools declared" }
        } else {
          div { class: "space-y-1",
            for tool in config.tools.iter() {
              div { class: "flex items-center gap-3 py-1.5 border-b border-[var(--outline-variant)]/20 last:border-b-0",
                span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
                  "build"
                }
                span { class: "text-sm font-mono text-[var(--on-surface)]",
                  "{tool.path}"
                }
                if tool.alias != tool.path {
                  span { class: "text-xs text-[var(--outline)]", "as" }
                  span { class: "text-sm font-mono text-[var(--primary)]",
                    "{tool.alias}"
                  }
                }
              }
            }
          }
        }
      }
      ConfigSection { title: "Channels",
        if config.channels.is_empty() {
          div { class: "text-sm text-[var(--outline)] italic", "No channel subscriptions" }
        } else {
          div { class: "flex flex-wrap gap-2",
            for ch in config.channels.iter() {
              span { class: "inline-flex items-center gap-1.5 rounded border border-[var(--outline-variant)]/30 bg-[var(--surface)] px-2.5 py-1 text-xs font-mono text-[var(--on-surface)]",
                span { class: "material-symbols-outlined text-xs text-[var(--outline)]",
                  "tag"
                }
                "{ch}"
              }
            }
          }
        }
      }
      if !config.fields.is_empty() {
        ConfigSection { title: "Fields",
          div { class: "space-y-2",
            for field in config.fields.iter() {
              div { class: "flex items-baseline gap-3",
                span { class: "text-xs font-mono text-[var(--outline)] w-28 shrink-0",
                  "{field.name}"
                }
                span { class: "text-sm font-mono text-[var(--on-surface)]",
                  "{field.value}"
                }
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
                adapter_type.set(config.adapter_type.clone());
                model.set(config.model.clone());
                dirty.set(false);
            },
            "Cancel"
          }
          button {
            class: BTN_PRIMARY_SM,
            onclick: move |_| {
                let update = AgentConfigUpdate {
                    adapter_type: adapter_type.read().clone(),
                    model: model.read().clone(),
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
