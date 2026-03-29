# Unit 15: Plugin System

## Scope

Port the plugin manager page, plugin settings page, plugin page host, and plugin bridge/slot system from Paperclip React into Dioxus 0.7.3 in lx-desktop. This creates the infrastructure for loading, configuring, and rendering plugin-contributed UI extensions.

## Paperclip Source Files

| Source | What it contains |
|--------|-----------------|
| `reference/paperclip/ui/src/pages/PluginManager.tsx` (509 lines) | Plugin list (installed + examples), install dialog, uninstall confirmation, enable/disable toggles, error details dialog, alpha warning banner |
| `reference/paperclip/ui/src/pages/PluginSettings.tsx` (837 lines) | Single plugin detail: about section, config form (auto-generated from JSON schema), status tab (runtime dashboard, health checks, logs, details, permissions), `PluginConfigForm` inner component |
| `reference/paperclip/ui/src/pages/PluginPage.tsx` (157 lines) | Host page for plugin-contributed `page` slots, resolves plugin by ID or route path, renders `PluginSlotMount` |
| `reference/paperclip/ui/src/plugins/bridge.ts` (475 lines) | Bridge runtime: `PluginBridgeContext`, `usePluginData`, `usePluginAction`, `useHostContext`, `usePluginToast`, `usePluginStream` hooks, error extraction |
| `reference/paperclip/ui/src/plugins/slots.tsx` (855 lines) | Slot system: `usePluginSlots` hook, `PluginSlotMount`, `PluginSlotOutlet`, error boundary, dynamic ESM module loader with bare-specifier rewriting, component registry |
| `reference/paperclip/ui/src/plugins/launchers.tsx` (833 lines) | Launcher system: `usePluginLaunchers`, `PluginLauncherProvider`, `PluginLauncherOutlet`, modal/drawer/popover shells, focus trap, launcher button |

## Target Directory Structure

```
crates/lx-desktop/src/
  plugins/
    mod.rs         (new — plugin module root)
    bridge.rs      (new — bridge context, data/action hooks)
    slots.rs       (new — slot types, registry, slot mount component)
    launchers.rs   (new — launcher types, launcher provider, launcher outlet)
  pages/
    plugins/
      mod.rs              (new — plugin manager page, replaces Unit 3 stub)
      plugin_card.rs      (new — per-plugin row with status/actions)
      plugin_settings.rs  (new — single plugin settings/status page)
      plugin_page.rs      (new — plugin page host)
      config_form.rs      (new — auto-generated config form from schema)
    mod.rs                (existing — already has plugins module from Unit 3)
  lib.rs                  (existing — add plugins module)
```

## Preconditions

- Units 13-14 complete (lib.rs has components module)
- **Unit 3 is complete:** Unit 3 created a stub `pages/plugins.rs`. This unit replaces it with a real plugins directory module. Delete `src/pages/plugins.rs` (the Unit 3 stub) and create `src/pages/plugins/mod.rs` with the real PluginManager component. The `routes.rs` Route enum already has `PluginManager {}`, `PluginPage { plugin_id: String }`, and `PluginSettingsPage { plugin_id: String }` variants importing from `crate::pages::plugins` -- no changes to `routes.rs` are needed.
- `crates/lx-desktop/src/pages/mod.rs` exists (already has `pub mod plugins;` from Unit 3)
- `crates/lx-desktop/src/lib.rs` exists with module declarations

## Tasks

### Task 1: Create `crates/lx-desktop/src/plugins/mod.rs`

```rust
pub mod bridge;
pub mod launchers;
pub mod slots;
```

### Task 2: Edit lib.rs

Edit `lib.rs` -- add `pub mod plugins;` after the existing `pub mod pages;` line. Note: `pub mod components;` already exists (added by Unit 1). Do NOT re-add it.

### Task 3: Create `crates/lx-desktop/src/plugins/bridge.rs`

Port the bridge context and types from `bridge.ts` lines 1-475. In Dioxus, the bridge is implemented as Rust context providers and types rather than React hooks. The bridge provides the mechanism for plugins to request data from the host and perform actions.

Reference: `bridge.ts` defines `PluginBridgeContext` (line 139), `PluginHostContext` (lines 70-79), `PluginBridgeError` (lines 46-50), `PluginDataResult` (lines 56-61), `usePluginData` (lines 232-304), `usePluginAction` (lines 324-350), `useHostContext` (lines 362-365).

```rust
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct PluginBridgeError {
    pub code: String,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginDataResult<T: Clone + PartialEq + 'static> {
    pub data: Option<T>,
    pub loading: bool,
    pub error: Option<PluginBridgeError>,
}

impl<T: Clone + PartialEq + 'static> Default for PluginDataResult<T> {
    fn default() -> Self {
        Self {
            data: None,
            loading: true,
            error: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginHostContext {
    pub company_id: Option<String>,
    pub company_prefix: Option<String>,
    pub project_id: Option<String>,
    pub entity_id: Option<String>,
    pub entity_type: Option<String>,
    pub user_id: Option<String>,
}

impl Default for PluginHostContext {
    fn default() -> Self {
        Self {
            company_id: None,
            company_prefix: None,
            project_id: None,
            entity_id: None,
            entity_type: None,
            user_id: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginBridgeContextValue {
    pub plugin_id: String,
    pub host_context: PluginHostContext,
}

pub fn use_plugin_bridge() -> Option<PluginBridgeContextValue> {
    use_context::<Signal<Option<PluginBridgeContextValue>>>()
        .read()
        .clone()
}

pub fn provide_plugin_bridge(value: PluginBridgeContextValue) {
    use_context_provider(|| Signal::new(Some(value)));
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginModalBoundsRequest {
    pub bounds: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginRenderCloseEvent {
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginRenderEnvironmentContext {
    pub environment: Option<String>,
    pub launcher_id: Option<String>,
    pub bounds: Option<String>,
}
```

### Task 4: Create `crates/lx-desktop/src/plugins/slots.rs`

Port slot types and the slot mount component from `slots.tsx`. The React version dynamically imports ESM modules and maintains a component registry. In Dioxus, plugins are Rust modules compiled at build time, so the slot system becomes a registry of Rust component functions.

Reference: `slots.tsx` defines `PluginSlotContext` (lines 48-57), `ResolvedPluginSlot` (lines 59-64), component registry (lines 96-149), `usePluginSlots` hook (lines 544-596), `PluginSlotMount` (lines 723-778), `PluginSlotOutlet` (lines 791-829).

```rust
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

pub fn register_plugin_component(
    plugin_key: &str,
    export_name: &str,
    render_fn: ComponentRenderFn,
) {
    let key = registry_key(plugin_key, export_name);
    component_registry()
        .lock()
        .unwrap()
        .insert(key, render_fn);
}

pub fn resolve_plugin_component(
    plugin_key: &str,
    export_name: &str,
) -> Option<ComponentRenderFn> {
    let key = registry_key(plugin_key, export_name);
    component_registry().lock().unwrap().get(&key).copied()
}

pub fn resolve_slots(
    contributions: &[PluginUiContribution],
    slot_types: &[&str],
    entity_type: Option<&str>,
) -> Vec<ResolvedPluginSlot> {
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
        ao.cmp(&bo)
            .then_with(|| a.plugin_display_name.cmp(&b.plugin_display_name))
            .then_with(|| a.display_name.cmp(&b.display_name))
    });
    rows
}

#[component]
pub fn PluginSlotMount(
    slot: ResolvedPluginSlot,
    context: PluginSlotContext,
    show_placeholder: Option<bool>,
) -> Element {
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
        }
    }
}

#[component]
pub fn PluginSlotOutlet(
    slot_types: Vec<String>,
    context: PluginSlotContext,
    entity_type: Option<String>,
) -> Element {
    let contributions: Vec<PluginUiContribution> = vec![];
    let type_refs: Vec<&str> = slot_types.iter().map(|s| s.as_str()).collect();
    let slots = resolve_slots(
        &contributions,
        &type_refs,
        entity_type.as_deref(),
    );

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
```

### Task 5: Create `crates/lx-desktop/src/plugins/launchers.rs`

Port launcher types and launcher provider from `launchers.tsx`. The launcher system discovers plugin-contributed UI triggers (buttons that open modals/drawers/popovers) and manages their lifecycle.

Reference: `launchers.tsx` defines `ResolvedPluginLauncher` (lines 55-61), `LauncherInstance` (lines 93-104), `PluginLauncherProvider` (lines 589-717), `PluginLauncherOutlet` (lines 758-801).

```rust
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
                        button {
                            class: "ml-auto px-3 py-1 text-xs rounded hover:bg-[var(--surface-container)]",
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
pub fn PluginLauncherOutlet(
    placement_zones: Vec<String>,
    context: PluginLauncherContext,
    entity_type: Option<String>,
) -> Element {
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
```

### Task 6: Create `crates/lx-desktop/src/pages/plugins/plugin_card.rs`

Port the per-plugin row from `PluginManager.tsx` lines 324-434. Shows plugin name, package, version, status badge, enable/disable toggle, settings link, uninstall button, error display.

```rust
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
                p { class: "text-sm text-[var(--outline)] truncate mt-0.5",
                    "{plugin.description}"
                }
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
                            class: if is_ready {
                                "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] text-green-600"
                            } else {
                                "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] text-[var(--outline)]"
                            },
                            disabled: enable_pending || disable_pending,
                            onclick: move |_| {
                                if is_ready {
                                    on_disable.call(id_disable.clone());
                                } else {
                                    on_enable.call(id_enable.clone());
                                }
                            },
                            span { class: "material-symbols-outlined text-base", "power_settings_new" }
                        }
                        button {
                            class: "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] text-red-500 hover:text-red-400",
                            disabled: uninstall_pending,
                            onclick: move |_| on_uninstall.call(id_uninstall.clone()),
                            span { class: "material-symbols-outlined text-base", "delete" }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 7: Create `crates/lx-desktop/src/pages/plugins/config_form.rs`

Port `PluginConfigForm` from `PluginSettings.tsx` lines 547-740. Auto-generated form from a JSON schema with save/test buttons.

```rust
use dioxus::prelude::*;
use std::collections::HashMap;

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
                                "boolean" => rsx! {
                                    button {
                                        class: if current_value == "true" {
                                            "relative inline-flex h-5 w-9 items-center rounded-full bg-green-600"
                                        } else {
                                            "relative inline-flex h-5 w-9 items-center rounded-full bg-[var(--surface-container)]"
                                        },
                                        onclick: move |_| {
                                            let mut vals = form_values();
                                            let new_val = if current_value == "true" {
                                                "false"
                                            } else {
                                                "true"
                                            };
                                            vals.insert(key.clone(), new_val.to_string());
                                            form_values.set(vals);
                                        },
                                        span {
                                            class: if current_value == "true" {
                                                "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-4"
                                            } else {
                                                "inline-block h-3.5 w-3.5 rounded-full bg-white translate-x-0.5"
                                            },
                                        }
                                    }
                                },
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
                div {
                    class: if msg_type == "success" {
                        "text-sm p-2 rounded border text-green-600 bg-green-50 border-green-200"
                    } else {
                        "text-sm p-2 rounded border text-red-500 bg-red-500/10 border-red-500/20"
                    },
                    "{text}"
                }
            }
            if let Some((ref msg_type, ref text)) = test_result {
                div {
                    class: if msg_type == "success" {
                        "text-sm p-2 rounded border text-green-600 bg-green-50 border-green-200"
                    } else {
                        "text-sm p-2 rounded border text-red-500 bg-red-500/10 border-red-500/20"
                    },
                    "{text}"
                }
            }
            div { class: "flex items-center gap-2 pt-2",
                button {
                    class: "bg-[var(--primary)] text-[var(--on-primary)] rounded px-4 py-1.5 text-xs font-semibold",
                    disabled: is_saving,
                    onclick: move |_| on_save.call(form_values()),
                    if is_saving { "Saving..." } else { "Save Configuration" }
                }
                if plugin_status == "ready" {
                    if let Some(ref test_handler) = on_test {
                        button {
                            class: "border border-[var(--outline-variant)] rounded px-4 py-1.5 text-xs",
                            disabled: is_testing,
                            onclick: move |_| test_handler.call(form_values()),
                            if is_testing { "Testing..." } else { "Test Configuration" }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 8: Create `crates/lx-desktop/src/pages/plugins/plugin_settings.rs`

Port `PluginSettings` from `PluginSettings.tsx` lines 60-541. Single plugin detail page with configuration tab and status tab. Configuration tab shows about section, auto-generated config form or custom settings page. Status tab shows runtime dashboard, health checks, logs, details, permissions.

```rust
use dioxus::prelude::*;
use super::config_form::{ConfigSchemaField, PluginConfigForm};
use super::plugin_card::PluginRecord;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingsTab {
    Configuration,
    Status,
}

#[component]
pub fn PluginSettingsPage(plugin_id: String) -> Element {
    let mut active_tab = use_signal(|| SettingsTab::Configuration);

    // PluginRecord is received as a component prop in the full implementation.
    // Here we use a placeholder for development purposes.
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
                button {
                    class: "h-8 w-8 flex items-center justify-center rounded border border-[var(--outline-variant)] hover:bg-[var(--surface-container)]",
                    span { class: "material-symbols-outlined text-base", "arrow_back" }
                }
                div { class: "flex items-center gap-2",
                    span { class: "material-symbols-outlined text-[var(--outline)]", "extension" }
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

            // Tab bar
            div { class: "flex border-b border-[var(--outline-variant)]",
                button {
                    class: if active_tab() == SettingsTab::Configuration {
                        "px-4 py-2 text-xs font-semibold uppercase tracking-wider border-b-2 border-[var(--primary)] text-[var(--on-surface)]"
                    } else {
                        "px-4 py-2 text-xs uppercase tracking-wider text-[var(--outline)] hover:text-[var(--on-surface)] cursor-pointer"
                    },
                    onclick: move |_| active_tab.set(SettingsTab::Configuration),
                    "Configuration"
                }
                button {
                    class: if active_tab() == SettingsTab::Status {
                        "px-4 py-2 text-xs font-semibold uppercase tracking-wider border-b-2 border-[var(--primary)] text-[var(--on-surface)]"
                    } else {
                        "px-4 py-2 text-xs uppercase tracking-wider text-[var(--outline)] hover:text-[var(--on-surface)] cursor-pointer"
                    },
                    onclick: move |_| active_tab.set(SettingsTab::Status),
                    "Status"
                }
            }

            match active_tab() {
                SettingsTab::Configuration => rsx! {
                    div { class: "space-y-8",
                        // About
                        div { class: "space-y-5",
                            h2 { class: "text-base font-semibold text-[var(--on-surface)]", "About" }
                            div { class: "space-y-2",
                                h3 { class: "text-sm font-medium text-[var(--outline)]", "Description" }
                                p { class: "text-sm text-[var(--on-surface)]/90", "{plugin.description}" }
                            }
                        }
                        div { class: "border-t border-[var(--outline-variant)]" }
                        // Settings
                        div { class: "space-y-4",
                            h2 { class: "text-base font-semibold text-[var(--on-surface)]", "Settings" }
                            if config_fields.is_empty() {
                                p { class: "text-sm text-[var(--outline)]",
                                    "This plugin does not require any settings."
                                }
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
                            // Runtime Dashboard
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
                            // Health Status
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
                            // Details
                            div { class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)]",
                                div { class: "px-4 py-3 border-b border-[var(--outline-variant)]",
                                    h3 { class: "text-base font-semibold text-[var(--on-surface)]",
                                        "Details"
                                    }
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
                                        span { class: "text-xs text-right truncate max-w-[170px]",
                                            "{plugin.package_name}"
                                        }
                                    }
                                    div { class: "flex justify-between gap-3",
                                        span { "Version" }
                                        span { class: "text-right text-[var(--on-surface)]",
                                            "v{plugin.version}"
                                        }
                                    }
                                }
                            }
                            // Permissions
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
```

### Task 9: Create `crates/lx-desktop/src/pages/plugins/plugin_page.rs`

Port `PluginPage` from `PluginPage.tsx` lines 1-157. Host page that renders a plugin's `page` slot contribution.

```rust
use dioxus::prelude::*;
use crate::plugins::slots::{PluginSlotContext, PluginSlotMount, ResolvedPluginSlot};

#[component]
pub fn PluginPage(plugin_id: String) -> Element {
    let page_slot: Option<ResolvedPluginSlot> = None;

    let context = PluginSlotContext {
        company_id: None,
        company_prefix: None,
        project_id: None,
        entity_id: None,
        entity_type: None,
    };

    rsx! {
        div { class: "space-y-4 p-4",
            div { class: "flex items-center gap-2",
                button {
                    class: "flex items-center gap-1 px-3 py-1.5 text-xs rounded hover:bg-[var(--surface-container)]",
                    span { class: "material-symbols-outlined text-sm", "arrow_back" }
                    "Back"
                }
            }
            if let Some(slot) = page_slot {
                PluginSlotMount {
                    slot: slot,
                    context: context,
                    show_placeholder: Some(true),
                }
            } else {
                div { class: "text-sm text-[var(--outline)]",
                    "No page slot found for this plugin."
                }
            }
        }
    }
}
```

### Task 10: Create `crates/lx-desktop/src/pages/plugins/mod.rs`

Port `PluginManager` from `PluginManager.tsx` lines 63-509. Plugin list page with installed plugins, available examples, install dialog, alpha warning.

```rust
mod config_form;
pub mod plugin_card;
mod plugin_page;
mod plugin_settings;

use dioxus::prelude::*;
use self::plugin_card::{PluginCard, PluginRecord};

pub use self::plugin_page::PluginPage;
pub use self::plugin_settings::PluginSettingsPage;

#[component]
pub fn PluginManager() -> Element {
    let mut install_dialog_open = use_signal(|| false);
    let mut install_package = use_signal(String::new);
    let mut uninstall_plugin_id: Signal<Option<String>> = use_signal(|| None);
    let mut uninstall_plugin_name = use_signal(String::new);

    let installed_plugins: Vec<PluginRecord> = vec![];

    rsx! {
        div { class: "space-y-6 max-w-5xl p-4 overflow-auto",
            div { class: "flex items-center justify-between",
                div { class: "flex items-center gap-2",
                    span { class: "material-symbols-outlined text-[var(--outline)]", "extension" }
                    h1 { class: "text-xl font-semibold text-[var(--on-surface)]",
                        "Plugin Manager"
                    }
                }
                button {
                    class: "flex items-center gap-2 bg-[var(--primary)] text-[var(--on-primary)] rounded px-3 py-1.5 text-xs font-semibold",
                    onclick: move |_| install_dialog_open.set(true),
                    span { class: "material-symbols-outlined text-sm", "add" }
                    "Install Plugin"
                }
            }

            // Alpha warning
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

            // Installed Plugins
            div { class: "space-y-3",
                div { class: "flex items-center gap-2",
                    span { class: "material-symbols-outlined text-[var(--outline)]", "extension" }
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

            // Install dialog
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

            // Uninstall confirmation dialog
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
                            button {
                                class: "bg-red-600 text-white rounded px-4 py-2 text-sm font-semibold",
                                "Uninstall"
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### Task 11: Verify `crates/lx-desktop/src/pages/mod.rs`

The `pub mod plugins;` declaration already exists from Unit 3. No changes needed.

### Task 12: Note on routes

Unit 3 already has `PluginManager`, `PluginPage`, and `PluginSettingsPage` route variants with imports pointing at `crate::pages::plugins`. Creating the real directory module at that path replaces the stub automatically. Do NOT modify `routes.rs` or `pages/mod.rs`.

## Definition of Done

1. `just diagnose` passes with zero warnings
2. All new files exist at the paths listed above
3. All new files are under 300 lines
4. `lib.rs` includes `pub mod plugins`
5. `plugins/mod.rs` declares bridge, slots, and launchers modules
6. `pages/mod.rs` already includes `pub mod plugins` (from Unit 3)
7. `routes.rs` compiles with existing plugin route variants (already defined by Unit 3)
8. `bridge.rs` defines `PluginBridgeContextValue`, `PluginHostContext`, `PluginBridgeError`, `PluginDataResult`, and context provider/consumer functions
9. `slots.rs` defines `ResolvedPluginSlot`, `PluginSlotContext`, component registry with `register_plugin_component`/`resolve_plugin_component`, `resolve_slots`, `PluginSlotMount`, and `PluginSlotOutlet` components
10. `launchers.rs` defines `ResolvedPluginLauncher`, `PluginLauncherContext`, `PluginLauncherProvider` with modal shell rendering, and `PluginLauncherOutlet`
11. `PluginManager` page renders alpha warning, empty installed list with placeholder, install dialog, uninstall confirmation dialog
12. `PluginSettingsPage` renders plugin detail with Configuration/Status tabs, about section, config form placeholder, runtime dashboard placeholder, health status, details card, permissions card
13. `PluginPage` renders back button and slot mount (or "no page slot found" fallback)
14. `PluginCard` renders plugin name, package, version, status badge, enable/disable button, uninstall button, error display
15. `PluginConfigForm` renders auto-generated form fields with save/test buttons and status messages
