use super::config_form::{ConfigSchemaField, PluginConfigForm};
use super::plugin_card::PluginRecord;
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsTab {
  Configuration,
  Status,
}

#[component]
pub fn PluginSettingsPage(plugin_id: String) -> Element {
  let mut active_tab = use_signal(|| SettingsTab::Configuration);

  let plugin = PluginRecord {
    id: plugin_id.clone(),
    plugin_key: "example-plugin".into(),
    package_name: "@example/plugin".into(),
    version: "1.0.0".into(),
    display_name: "Example Plugin".into(),
    description: "An example plugin.".into(),
    status: "ready".into(),
    last_error: None,
    categories: vec![],
  };

  let status_class = match plugin.status.as_str() {
    "ready" => "bg-green-600 text-white",
    "error" => "bg-red-600 text-white",
    _ => "bg-[var(--surface-container)] text-[var(--outline)]",
  };

  let config_fields: Vec<ConfigSchemaField> = vec![];

  rsx! {
    div { class: "space-y-6 max-w-5xl p-4 overflow-auto",
      div { class: "flex items-center gap-4",
        button { class: "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] hover:bg-[var(--surface-container)]",
          span { class: "material-symbols-outlined text-base", "arrow_back" }
        }
        div { class: "flex items-center gap-2",
          span { class: "material-symbols-outlined text-[var(--outline)]",
            "extension"
          }
          h1 { class: "text-xl font-semibold text-[var(--on-surface)]",
            "{plugin.display_name}"
          }
          span { class: "px-2 py-0.5 rounded text-xs font-medium ml-2 {status_class}",
            "{plugin.status}"
          }
          span { class: "px-2 py-0.5 rounded text-xs border border-[var(--outline-variant)] text-[var(--outline)] ml-1",
            "v{plugin.version}"
          }
        }
      }

      div { class: "flex border-b border-[var(--outline-variant)]",
        button {
          class: if active_tab() == SettingsTab::Configuration { "px-4 py-2 text-xs font-semibold uppercase tracking-wider border-b-2 border-[var(--primary)] text-[var(--on-surface)]" } else { "px-4 py-2 text-xs uppercase tracking-wider text-[var(--outline)] hover:text-[var(--on-surface)] cursor-pointer" },
          onclick: move |_| active_tab.set(SettingsTab::Configuration),
          "Configuration"
        }
        button {
          class: if active_tab() == SettingsTab::Status { "px-4 py-2 text-xs font-semibold uppercase tracking-wider border-b-2 border-[var(--primary)] text-[var(--on-surface)]" } else { "px-4 py-2 text-xs uppercase tracking-wider text-[var(--outline)] hover:text-[var(--on-surface)] cursor-pointer" },
          onclick: move |_| active_tab.set(SettingsTab::Status),
          "Status"
        }
      }

      match active_tab() {
          SettingsTab::Configuration => rsx! {
            div { class: "space-y-8",
              div { class: "space-y-5",
                h2 { class: "text-base font-semibold text-[var(--on-surface)]", "About" }
                div { class: "space-y-2",
                  h3 { class: "text-sm font-medium text-[var(--outline)]", "Description" }
                  p { class: "text-sm text-[var(--on-surface)]/90", "{plugin.description}" }
                }
              }
              div { class: "border-t border-[var(--outline-variant)]" }
              div { class: "space-y-4",
                h2 { class: "text-base font-semibold text-[var(--on-surface)]", "Settings" }
                if config_fields.is_empty() {
                  p { class: "text-sm text-[var(--outline)]", "This plugin does not require any settings." }
                } else {
                  PluginConfigForm {
                    plugin_id: plugin_id.clone(),
                    fields: config_fields,
                    values: std::collections::HashMap::new(),
                    on_save: move |_vals| {},
                    on_test: None,
                    is_saving: false,
                    is_testing: false,
                    save_message: None,
                    test_result: None,
                    plugin_status: plugin.status.clone(),
                  }
                }
              }
            }
          },
          SettingsTab::Status => rsx! {
            div { class: "grid gap-6 xl:grid-cols-[minmax(0,1.2fr)_320px]",
              div { class: "space-y-6",
                div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)]",
                  div { class: "px-4 py-3 border-b border-[var(--outline-variant)]",
                    h3 { class: "text-base font-semibold flex items-center gap-1.5 text-[var(--on-surface)]",
                      span { class: "material-symbols-outlined text-base", "memory" }
                      "Runtime Dashboard"
                    }
                    p { class: "text-xs text-[var(--outline)]",
                      "Worker process, scheduled jobs, and webhook deliveries"
                    }
                  }
                  div { class: "p-4 text-sm text-[var(--outline)]",
                    "Runtime diagnostics are unavailable right now."
                  }
                }
              }
              div { class: "space-y-6",
                div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)]",
                  div { class: "px-4 py-3 border-b border-[var(--outline-variant)]",
                    h3 { class: "text-base font-semibold flex items-center gap-1.5 text-[var(--on-surface)]",
                      span { class: "material-symbols-outlined text-base", "monitor_heart" }
                      "Health Status"
                    }
                  }
                  div { class: "p-4",
                    div { class: "flex items-center justify-between text-sm",
                      span { class: "text-[var(--outline)]", "Lifecycle" }
                      span { class: "px-2 py-0.5 rounded text-xs font-medium {status_class}",
                        "{plugin.status}"
                      }
                    }
                    p { class: "text-sm text-[var(--outline)] mt-2",
                      "Health checks run once the plugin is ready."
                    }
                  }
                }
                div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)]",
                  div { class: "px-4 py-3 border-b border-[var(--outline-variant)]",
                    h3 { class: "text-base font-semibold text-[var(--on-surface)]", "Details" }
                  }
                  div { class: "p-4 space-y-3 text-sm text-[var(--outline)]",
                    div { class: "flex justify-between gap-3",
                      span { "Plugin ID" }
                      span { class: "font-mono text-xs text-right", "{plugin.id}" }
                    }
                    div { class: "flex justify-between gap-3",
                      span { "Plugin Key" }
                      span { class: "font-mono text-xs text-right", "{plugin.plugin_key}" }
                    }
                    div { class: "flex justify-between gap-3",
                      span { "NPM Package" }
                      span { class: "text-xs text-right truncate max-w-[170px]", "{plugin.package_name}" }
                    }
                    div { class: "flex justify-between gap-3",
                      span { "Version" }
                      span { class: "text-right text-[var(--on-surface)]", "v{plugin.version}" }
                    }
                  }
                }
                div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)]",
                  div { class: "px-4 py-3 border-b border-[var(--outline-variant)]",
                    h3 { class: "text-base font-semibold flex items-center gap-1.5 text-[var(--on-surface)]",
                      span { class: "material-symbols-outlined text-base", "shield" }
                      "Permissions"
                    }
                  }
                  div { class: "p-4",
                    p { class: "text-sm text-[var(--outline)] italic",
                      "No special permissions requested."
                    }
                  }
                }
              }
            }
          },
      }
    }
  }
}
