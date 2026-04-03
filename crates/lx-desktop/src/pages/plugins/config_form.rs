use crate::components::ui::select::{Select, SelectOption};
use dioxus::prelude::*;
use std::collections::HashMap;

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
pub struct ConfigSchemaField {
  pub key: String,
  pub label: String,
  pub field_type: String,
  pub description: Option<String>,
  pub required: bool,
  pub default_value: Option<String>,
}

#[component]
pub fn PluginConfigForm(
  plugin_id: String,
  fields: Vec<ConfigSchemaField>,
  values: HashMap<String, String>,
  on_save: EventHandler<HashMap<String, String>>,
  on_test: Option<EventHandler<HashMap<String, String>>>,
  is_saving: bool,
  is_testing: bool,
  save_message: Option<(String, String)>,
  test_result: Option<(String, String)>,
  plugin_status: String,
) -> Element {
  let mut form_values = use_signal(|| values.clone());

  rsx! {
    div { class: "space-y-4",
      for field in fields.iter() {
        {
            let key = field.key.clone();
            let current_value = form_values()
                .get(&field.key)
                .cloned()
                .unwrap_or_default();
            rsx! {
              div { class: "space-y-1.5",
                label { class: "text-sm font-medium text-[var(--on-surface)]",
                  "{field.label}"
                  if field.required {
                    span { class: "text-red-500 ml-0.5", "*" }
                  }
                }
                if let Some(ref desc) = field.description {
                  p { class: "text-xs text-[var(--outline)]", "{desc}" }
                }
                match field.field_type.as_str() {
                    "boolean" => {
                        let cv = current_value.clone();
                        rsx! {
                          button {
                            class: if current_value == "true" { "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600" } else { "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)]" },
                            onclick: move |_| {
                                let mut vals = form_values();
                                let new_val = if cv == "true" { "false" } else { "true" };
                                vals.insert(key.clone(), new_val.to_string());
                                form_values.set(vals);
                            },
                            span { class: if current_value == "true" { "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-4" } else { "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-0.5" } }
                          }
                        }
                    }
                    "textarea" => rsx! {
                      textarea {
                        class: "w-full min-h-20 rounded-md border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm outline-none text-[var(--on-surface)]",
                        value: "{current_value}",
                        oninput: move |evt| {
                            let mut vals = form_values();
                            vals.insert(key.clone(), evt.value());
                            form_values.set(vals);
                        },
                      }
                    },
                    "model" => {
                        let cv = current_value.clone();
                        rsx! {
                          Select {
                            class: "w-full".to_string(),
                            value: cv.clone(),
                            searchable: true,
                            placeholder: "Select a model...".to_string(),
                            options: {
                                let mut opts: Vec<SelectOption> = MODEL_OPTIONS
                                    .iter()
                                    .map(|(v, l)| SelectOption::new(*v, *l))
                                    .collect();
                                if !cv.is_empty() && !opts.iter().any(|o| o.value == cv) {
                                    opts.insert(0, SelectOption::new(cv.clone(), cv));
                                }
                                opts
                            },
                            onchange: move |val: String| {
                                let mut vals = form_values();
                                vals.insert(key.clone(), val);
                                form_values.set(vals);
                            },
                          }
                        }
                    }
                    _ => rsx! {
                      input {
                        class: "w-full rounded-md border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm outline-none text-[var(--on-surface)]",
                        r#type: "text",
                        value: "{current_value}",
                        oninput: move |evt| {
                            let mut vals = form_values();
                            vals.insert(key.clone(), evt.value());
                            form_values.set(vals);
                        },
                      }
                    },
                }
              }
            }
        }
      }
      if let Some((ref msg_type, ref text)) = save_message {
        div { class: if msg_type == "success" { "text-sm p-2 rounded border text-green-600 bg-green-50 border-green-200" } else { "text-sm p-2 rounded border text-red-500 bg-red-500/10 border-red-500/20" },
          "{text}"
        }
      }
      if let Some((ref msg_type, ref text)) = test_result {
        div { class: if msg_type == "success" { "text-sm p-2 rounded border text-green-600 bg-green-50 border-green-200" } else { "text-sm p-2 rounded border text-red-500 bg-red-500/10 border-red-500/20" },
          "{text}"
        }
      }
      div { class: "flex items-center gap-2 pt-2",
        button {
          class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-1.5 text-xs font-semibold",
          disabled: is_saving,
          onclick: move |_| on_save.call(form_values()),
          if is_saving {
            "Saving..."
          } else {
            "Save Configuration"
          }
        }
        if plugin_status == "ready" {
          if let Some(test_handler) = on_test {
            button {
              class: "border border-[var(--outline-variant)] rounded px-4 py-1.5 text-xs",
              disabled: is_testing,
              onclick: move |_| test_handler.call(form_values()),
              if is_testing {
                "Testing..."
              } else {
                "Test Configuration"
              }
            }
          }
        }
      }
    }
  }
}
