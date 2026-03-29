use dioxus::prelude::*;

#[component]
pub fn PluginManager() -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Plugin Manager (stub)" }
  }
}

#[component]
pub fn PluginPage(plugin_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Plugin {plugin_id} (stub)" }
  }
}

#[component]
pub fn PluginSettingsPage(plugin_id: String) -> Element {
  rsx! {
    div { class: "p-4 text-sm text-muted-foreground", "Plugin Settings {plugin_id} (stub)" }
  }
}
