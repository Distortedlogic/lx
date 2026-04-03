use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PluginRecord {
  pub id: String,
  pub plugin_key: String,
  pub package_name: String,
  pub version: String,
  pub display_name: String,
  pub description: String,
  pub status: String,
  pub last_error: Option<String>,
  pub categories: Vec<String>,
}

fn status_badge_class(status: &str) -> &'static str {
  match status {
    "ready" => "bg-green-600 text-white",
    "error" => "bg-red-600 text-white",
    _ => "bg-[var(--surface-container)] text-[var(--outline)]",
  }
}

#[component]
pub fn PluginCard(
  plugin: PluginRecord,
  on_enable: EventHandler<String>,
  on_disable: EventHandler<String>,
  on_uninstall: EventHandler<String>,
  is_example: bool,
  enable_pending: bool,
  disable_pending: bool,
  uninstall_pending: bool,
) -> Element {
  let status_class = status_badge_class(&plugin.status);
  let id_enable = plugin.id.clone();
  let id_disable = plugin.id.clone();
  let id_uninstall = plugin.id.clone();
  let is_ready = plugin.status == "ready";
  let is_error = plugin.status == "error";

  rsx! {
    div { class: "flex items-start gap-4 px-4 py-3",
      div { class: "min-w-0 flex-1",
        div { class: "flex flex-wrap items-center gap-2",
          span { class: "font-medium text-[var(--on-surface)] truncate",
            "{plugin.display_name}"
          }
          if is_example {
            span { class: "px-1.5 py-0.5 text-[10px] rounded border border-[var(--outline-variant)] text-[var(--outline)]",
              "Example"
            }
          }
        }
        p { class: "text-xs text-[var(--outline)] mt-0.5 truncate",
          "{plugin.package_name} · v{plugin.version}"
        }
        p { class: "text-sm text-[var(--outline)] truncate mt-0.5", "{plugin.description}" }
        if is_error {
          if let Some(ref err) = plugin.last_error {
            div { class: "mt-3 rounded-md border border-red-500/25 bg-red-500/[0.06] px-3 py-2",
              div { class: "flex items-center gap-2 text-sm font-medium text-red-500",
                span { class: "material-symbols-outlined text-sm", "warning" }
                "Plugin error"
              }
              p { class: "mt-1 text-sm text-red-500/90 break-words truncate",
                "{err}"
              }
            }
          }
        }
      }
      div { class: "flex shrink-0 self-center",
        div { class: "flex flex-col items-end gap-2",
          div { class: "flex items-center gap-2",
            span { class: "shrink-0 px-2 py-0.5 rounded text-xs font-medium {status_class}",
              "{plugin.status}"
            }
            button {
              class: if is_ready { "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] text-green-600" } else { "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] text-[var(--outline)]" },
              disabled: enable_pending || disable_pending,
              onclick: move |_| {
                  if is_ready {
                      on_disable.call(id_disable.clone());
                  } else {
                      on_enable.call(id_enable.clone());
                  }
              },
              span { class: "material-symbols-outlined text-sm", "power_settings_new" }
            }
            button {
              class: "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] text-red-500 hover:text-red-400",
              disabled: uninstall_pending,
              onclick: move |_| on_uninstall.call(id_uninstall.clone()),
              span { class: "material-symbols-outlined text-sm", "delete" }
            }
          }
        }
      }
    }
  }
}
