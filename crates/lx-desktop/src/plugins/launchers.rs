use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PluginLauncherAction {
  pub action_type: String,
  pub target: String,
  pub params: Option<std::collections::HashMap<String, String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginLauncherDeclaration {
  pub id: String,
  pub display_name: String,
  pub placement_zone: String,
  pub action: PluginLauncherAction,
  pub order: Option<i32>,
  pub entity_types: Vec<String>,
  pub render_environment: Option<String>,
  pub render_bounds: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedPluginLauncher {
  pub id: String,
  pub display_name: String,
  pub placement_zone: String,
  pub action: PluginLauncherAction,
  pub order: Option<i32>,
  pub entity_types: Vec<String>,
  pub plugin_id: String,
  pub plugin_key: String,
  pub plugin_display_name: String,
  pub plugin_version: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginLauncherContext {
  pub company_id: Option<String>,
  pub company_prefix: Option<String>,
  pub project_id: Option<String>,
  pub entity_id: Option<String>,
  pub entity_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
struct LauncherInstance {
  key: String,
  launcher: ResolvedPluginLauncher,
  bounds: String,
}

pub fn resolve_launchers(
  contributions: &[super::slots::PluginUiContribution],
  placement_zones: &[&str],
  entity_type: Option<&str>,
) -> Vec<ResolvedPluginLauncher> {
  let _ = (contributions, placement_zones, entity_type);
  vec![]
}

#[component]
pub fn PluginLauncherProvider(children: Element) -> Element {
  let stack: Signal<Vec<LauncherInstance>> = use_signal(Vec::new);

  rsx! {
    {children}
    for instance in stack().iter() {
      div { class: "fixed inset-0 bg-black/45 z-[1000]",
        div {
          class: "fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 max-h-[calc(100vh-2rem)] overflow-hidden rounded-2xl border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] shadow-2xl z-[1001]",
          style: "width: min(40rem, calc(100vw - 2rem))",
          div { class: "flex items-center gap-3 border-b border-[var(--outline-variant)] px-4 py-3",
            div { class: "min-w-0",
              h2 { class: "truncate text-sm font-semibold text-[var(--on-surface)]",
                "{instance.launcher.display_name}"
              }
              p { class: "truncate text-xs text-[var(--outline)]",
                "{instance.launcher.plugin_display_name}"
              }
            }
            button { class: "ml-auto px-3 py-1 text-xs rounded hover:bg-[var(--surface-container)]",
              "Close"
            }
          }
          div { class: "overflow-auto p-4 max-h-[calc(100vh-7rem)]",
            p { class: "text-sm text-[var(--outline)]",
              "Plugin launcher content would render here."
            }
          }
        }
      }
    }
  }
}

#[component]
pub fn PluginLauncherOutlet(placement_zones: Vec<String>, context: PluginLauncherContext, entity_type: Option<String>) -> Element {
  let launchers: Vec<ResolvedPluginLauncher> = vec![];

  if launchers.is_empty() {
    return rsx! {};
  }

  rsx! {
    div {
      for launcher in launchers.iter() {
        button {
          key: "{launcher.plugin_key}:{launcher.id}",
          class: "h-8 px-3 py-1 text-xs rounded border border-[var(--outline-variant)] hover:bg-[var(--surface-container)]",
          "{launcher.display_name}"
        }
      }
    }
  }
}
