mod config_form;
pub mod plugin_card;
mod plugin_page;
mod plugin_settings;

use self::plugin_card::{PluginCard, PluginRecord};
use dioxus::prelude::*;

pub use self::plugin_page::PluginPage;
pub use self::plugin_settings::PluginSettingsPage;

#[component]
pub fn PluginManager() -> Element {
  let mut install_dialog_open = use_signal(|| false);
  let mut install_package = use_signal(String::new);
  let mut uninstall_plugin_id: Signal<Option<String>> = use_signal(|| None);
  let uninstall_plugin_name = use_signal(String::new);

  let installed_plugins: Vec<PluginRecord> = vec![];

  rsx! {
    div { class: "space-y-6 max-w-5xl p-4 overflow-auto",
      div { class: "flex items-center justify-between",
        div { class: "flex items-center gap-2",
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
            "extension"
          }
          h1 { class: "text-xl font-semibold text-[var(--on-surface)]", "Plugin Manager" }
        }
        button {
          class: "flex items-center gap-2 bg-[var(--primary)] text-[var(--on-primary)] rounded px-3 py-1.5 text-xs font-semibold",
          onclick: move |_| install_dialog_open.set(true),
          span { class: "material-symbols-outlined text-sm", "add" }
          "Install Plugin"
        }
      }

      div { class: "rounded-lg border border-amber-500/30 bg-amber-500/5 px-4 py-3",
        div { class: "flex items-start gap-3",
          span { class: "material-symbols-outlined mt-0.5 text-amber-700 text-base shrink-0",
            "warning"
          }
          div { class: "space-y-1 text-sm",
            p { class: "font-medium text-[var(--on-surface)]", "Plugins are alpha." }
            p { class: "text-[var(--outline)]",
              "The plugin runtime and API surface are still changing. Expect breaking changes."
            }
          }
        }
      }

      div { class: "space-y-3",
        div { class: "flex items-center gap-2",
          span { class: "material-symbols-outlined text-base text-[var(--outline)]",
            "extension"
          }
          h2 { class: "text-base font-semibold text-[var(--on-surface)]",
            "Installed Plugins"
          }
        }
        if installed_plugins.is_empty() {
          div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container)]/30",
            div { class: "flex flex-col items-center justify-center py-10",
              span { class: "material-symbols-outlined text-4xl text-[var(--outline)] mb-4",
                "extension"
              }
              p { class: "text-sm font-medium text-[var(--on-surface)]",
                "No plugins installed"
              }
              p { class: "text-xs text-[var(--outline)] mt-1",
                "Install a plugin to extend functionality."
              }
            }
          }
        } else {
          div { class: "divide-y rounded-md border bg-[var(--surface-container-lowest)]",
            for plugin in installed_plugins.iter() {
              PluginCard {
                key: "{plugin.id}",
                plugin: plugin.clone(),
                on_enable: move |_id: String| {},
                on_disable: move |_id: String| {},
                on_uninstall: move |_id: String| {},
                is_example: false,
                enable_pending: false,
                disable_pending: false,
                uninstall_pending: false,
              }
            }
          }
        }
      }

      if install_dialog_open() {
        div { class: "fixed inset-0 bg-black/45 z-50 flex items-center justify-center",
          div { class: "bg-[var(--surface-container-lowest)] rounded-2xl border border-[var(--outline-variant)] shadow-2xl w-full max-w-md",
            div { class: "px-6 py-4 border-b border-[var(--outline-variant)]",
              h3 { class: "text-lg font-semibold text-[var(--on-surface)]",
                "Install Plugin"
              }
              p { class: "text-sm text-[var(--outline)] mt-1",
                "Enter the npm package name of the plugin you wish to install."
              }
            }
            div { class: "px-6 py-4",
              label { class: "text-sm font-medium text-[var(--on-surface)]",
                "npm Package Name"
              }
              input {
                class: "w-full mt-2 rounded-md border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm outline-none text-[var(--on-surface)]",
                placeholder: "@example/plugin-name",
                value: "{install_package}",
                oninput: move |evt| install_package.set(evt.value()),
              }
            }
            div { class: "flex justify-end gap-2 px-6 py-4 border-t border-[var(--outline-variant)]",
              button {
                class: "border border-[var(--outline-variant)] rounded px-4 py-2 text-sm",
                onclick: move |_| install_dialog_open.set(false),
                "Cancel"
              }
              button {
                class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-2 text-sm font-semibold",
                disabled: install_package().is_empty(),
                "Install"
              }
            }
          }
        }
      }

      if uninstall_plugin_id().is_some() {
        div { class: "fixed inset-0 bg-black/45 z-50 flex items-center justify-center",
          div { class: "bg-[var(--surface-container-lowest)] rounded-2xl border border-[var(--outline-variant)] shadow-2xl w-full max-w-md",
            div { class: "px-6 py-4 border-b border-[var(--outline-variant)]",
              h3 { class: "text-lg font-semibold text-[var(--on-surface)]",
                "Uninstall Plugin"
              }
              p { class: "text-sm text-[var(--outline)] mt-1",
                "Are you sure you want to uninstall "
                strong { "{uninstall_plugin_name}" }
                "? This action cannot be undone."
              }
            }
            div { class: "flex justify-end gap-2 px-6 py-4",
              button {
                class: "border border-[var(--outline-variant)] rounded px-4 py-2 text-sm",
                onclick: move |_| uninstall_plugin_id.set(None),
                "Cancel"
              }
              button { class: "bg-red-600 text-white rounded px-4 py-2 text-sm font-semibold",
                "Uninstall"
              }
            }
          }
        }
      }
    }
  }
}
