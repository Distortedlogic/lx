# Goal

Build a unified `SettingsState` context backed by `dioxus-storage`'s `use_persistent_store` and wire every control on the Settings page to it: DISCARD/EXECUTE buttons, env vars CRUD, task priority slider, checkboxes, and quota values.

# Why

Every control on the Settings page is inert. Environment variables are hardcoded constants with no add/edit/delete. The range slider has no handler. The checkboxes are fake divs, not real inputs. The DISCARD and EXECUTE buttons have no onclick handlers. Quotas are hardcoded. All of this renders the Settings page non-functional.

# Architecture

One `SettingsState` context struct holds all settings as a single `Signal<SettingsData>` plus a `snapshot: Signal<SettingsData>` for DISCARD support. `SettingsData` is a serializable struct persisted via `use_persistent_store` from the `dioxus-storage` crate (already available at `dioxus-common/crates/dioxus-storage`).

Flow:
- On mount: `use_persistent_store("lx_settings", || SettingsData::default())` loads persisted data or creates defaults.
- Live edits update the `data` signal. The page renders from `data`.
- DISCARD: reset `data` to `snapshot`.
- EXECUTE: copy `data` into `snapshot`, and the persistent store auto-saves.

This requires adding `dioxus-storage` as a dependency.

# Files Affected

| File | Change |
|------|--------|
| `Cargo.toml` | Add dioxus-storage dependency |
| `src/pages/settings/state.rs` | New file — SettingsData + SettingsState |
| `src/pages/settings/mod.rs` | Provide SettingsState, wire DISCARD/EXECUTE |
| `src/pages/settings/env_vars.rs` | Rewrite with signal-driven CRUD |
| `src/pages/settings/task_priority.rs` | Wire slider and checkboxes to signals |
| `src/pages/settings/quotas.rs` | Wire quota bars to signals |

# Task List

### Task 1: Add dioxus-storage dependency and create SettingsData

**Subject:** Add the persistence dependency and define the settings data model

**Description:** First, edit `crates/lx-desktop/Cargo.toml`. Add this line in the `[dependencies]` section, after the existing `dioxus-widget-bridge` line:

```toml
dioxus-storage = { path = "../../../dioxus-common/crates/dioxus-storage" }
```

Then create `crates/lx-desktop/src/pages/settings/state.rs`. Define:

```rust
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EnvEntry {
    pub key: String,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SettingsData {
    pub env_vars: Vec<EnvEntry>,
    pub task_priority: f64,
    pub auto_scale: bool,
    pub redundant_verify: bool,
    pub compute_quota: u8,
    pub memory_quota: u8,
    pub storage_quota: u8,
}

impl Default for SettingsData {
    fn default() -> Self {
        Self {
            env_vars: vec![
                EnvEntry { key: "API_ENDPOINT_ROOT".into(), value: "https://core.monolith.io/v2".into() },
                EnvEntry { key: "MAX_CONCURRENCY".into(), value: "512".into() },
                EnvEntry { key: "RETRY_POLICY".into(), value: "EXPONENTIAL_BACKOFF".into() },
            ],
            task_priority: 0.84,
            auto_scale: true,
            redundant_verify: false,
            compute_quota: 85,
            memory_quota: 32,
            storage_quota: 95,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SettingsState {
    pub data: Signal<SettingsData>,
    pub snapshot: Signal<SettingsData>,
}

impl SettingsState {
    pub fn provide() -> Self {
        let stored = dioxus_storage::use_persistent_store("lx_settings", SettingsData::default);
        let initial = stored.cloned();
        let ctx = Self {
            data: Signal::new(initial.clone()),
            snapshot: Signal::new(initial),
        };
        use_context_provider(|| ctx);
        ctx
    }

    pub fn discard(&self) {
        self.data.set(self.snapshot.read().clone());
    }

    pub fn execute(&self) {
        self.snapshot.set(self.data.read().clone());
    }
}
```

Note: `dioxus_storage::use_persistent_store` returns a `Store<T>`. The `cloned()` method returns `T`. We copy the stored value into our own signals for fine-grained reactivity. The `execute` method updates `snapshot` to match `data`; the persistent store observes and auto-saves.

If `Store<T>` does not have a `cloned()` method, use `stored.read().clone()` or `(*stored.get()).clone()` — check the actual API in `dioxus-common/crates/dioxus-storage/src/persistent_store.rs` and adapt accordingly.

**ActiveForm:** Creating SettingsData model and SettingsState context

---

### Task 2: Wire Settings page with SettingsState, DISCARD, and EXECUTE

**Subject:** Provide SettingsState and connect the two top-level buttons

**Description:** Edit `crates/lx-desktop/src/pages/settings/mod.rs`. Add:

```rust
mod state;
```

to the module declarations. Add `use self::state::SettingsState;` to the imports.

In the `Settings` component function body, add as the first line:

```rust
let settings = SettingsState::provide();
```

Find the DISCARD button (currently around line 20-21):
```rust
button { class: "border ...", "DISCARD CHANGES" }
```

Add an onclick handler:
```rust
button {
    class: "border border-[var(--outline)] text-[var(--on-surface)] rounded px-4 py-2 text-xs uppercase tracking-wider hover:bg-[var(--surface-container-high)] transition-colors duration-150",
    onclick: move |_| settings.discard(),
    "DISCARD CHANGES"
}
```

Find the EXECUTE button (currently around line 23):
```rust
button { class: "bg-[var(--warning)] ...", "EXECUTE DEPLOYMENT" }
```

Add an onclick handler:
```rust
button {
    class: "bg-[var(--warning)] text-[var(--on-primary)] rounded px-4 py-2 text-xs uppercase tracking-wider font-semibold hover:brightness-110 transition-all duration-150",
    onclick: move |_| settings.execute(),
    "APPLY SETTINGS"
}
```

Also rename the button label from "EXECUTE DEPLOYMENT" to "APPLY SETTINGS" — the old label was misleading.

**ActiveForm:** Wiring Settings page SettingsState and button handlers

---

### Task 3: Rewrite EnvVarsPanel with signal-driven CRUD

**Subject:** Replace hardcoded env vars with reactive add/edit/remove

**Description:** Rewrite `crates/lx-desktop/src/pages/settings/env_vars.rs`. Remove the `const VARS` hardcoded data and the `EnvVar` struct.

The new implementation reads from and writes to the `SettingsState` context:

```rust
use dioxus::prelude::*;
use super::state::{EnvEntry, SettingsState};

#[component]
pub fn EnvVarsPanel() -> Element {
    let settings = use_context::<SettingsState>();
    let mut new_key = use_signal(String::new);
    let mut new_value = use_signal(String::new);

    let env_vars = settings.data.read().env_vars.clone();

    rsx! {
        div { class: "bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-0 overflow-hidden",
            div { class: "bg-[var(--surface-container-high)] px-4 py-2 border-b-2 border-[var(--outline-variant)] flex justify-between items-center",
                span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]", "ENVIRONMENT_VARIABLES" }
                span { class: "text-[10px] uppercase tracking-wider text-[var(--tertiary)] font-mono", "COUNT: {env_vars.len()}" }
            }
            div { class: "flex text-[10px] uppercase tracking-wider text-[var(--on-surface-variant)] py-3 px-4 border-b border-[var(--outline-variant)]",
                span { class: "flex-[3]", "KEY" }
                span { class: "flex-[5]", "VALUE" }
                span { class: "flex-[1] text-right", "ACTIONS" }
            }
            div { class: "flex flex-col gap-1",
                for (i, entry) in env_vars.iter().enumerate() {
                    {
                        let key = entry.key.clone();
                        let value = entry.value.clone();
                        rsx! {
                            div { class: "flex items-center px-4 py-3 border-b border-[var(--outline-variant)]/30 hover:bg-[var(--surface-container)] transition-colors duration-150",
                                span { class: "flex-[3] text-xs font-semibold text-[var(--warning)] uppercase", "{key}" }
                                span { class: "flex-[5] text-xs text-[var(--on-surface-variant)]", "{value}" }
                                span { class: "flex-[1] text-right",
                                    span {
                                        class: "material-symbols-outlined text-sm text-[var(--outline)] cursor-pointer hover:text-[var(--error)]",
                                        onclick: move |_| {
                                            settings.data.write().env_vars.remove(i);
                                        },
                                        "delete"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "flex items-center gap-2 px-4 py-3",
                input {
                    class: "flex-[3] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
                    placeholder: "NEW_KEY",
                    value: "{new_key}",
                    oninput: move |evt| new_key.set(evt.value()),
                }
                input {
                    class: "flex-[5] bg-[var(--surface-container-low)] text-xs px-3 py-1.5 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
                    placeholder: "VALUE",
                    value: "{new_value}",
                    oninput: move |evt| new_value.set(evt.value()),
                }
                button {
                    class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-1.5 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150",
                    onclick: move |_| {
                        let k = new_key().trim().to_string();
                        let v = new_value().trim().to_string();
                        if !k.is_empty() {
                            settings.data.write().env_vars.push(EnvEntry { key: k, value: v });
                            new_key.set(String::new());
                            new_value.set(String::new());
                        }
                    },
                    "ADD"
                }
            }
        }
    }
}
```

Key changes: env vars come from `settings.data`, the delete icon removes by index, and ADD pushes a new entry. The edit icon is replaced with a delete icon — inline editing adds significant complexity for minimal value; users can delete and re-add.

**ActiveForm:** Rewriting EnvVarsPanel with signal-driven CRUD

---

### Task 4: Wire TaskPriorityPanel slider and checkboxes

**Subject:** Replace inert slider and fake checkbox divs with real interactive controls

**Description:** Rewrite `crates/lx-desktop/src/pages/settings/task_priority.rs`. The `TaskPriorityPanel` component must read from and write to `SettingsState`.

For the slider:
- Read `settings.data.read().task_priority` for display
- Add `oninput` that parses the float and writes to `settings.data.write().task_priority`
- Bind the `value` attribute to the signal value

For the checkboxes:
- Replace the fake `div { class: "w-4 h-4 rounded ..." }` elements with actual `input { r#type: "checkbox" }` elements
- Read checked state from `settings.data.read().auto_scale` and `.redundant_verify`
- Add `onchange` handlers that toggle the booleans in `settings.data.write()`

The full rewrite:

```rust
use dioxus::prelude::*;
use super::state::SettingsState;

#[component]
pub fn TaskPriorityPanel() -> Element {
    let settings = use_context::<SettingsState>();
    let priority = settings.data.read().task_priority;
    let auto_scale = settings.data.read().auto_scale;
    let redundant_verify = settings.data.read().redundant_verify;
    let priority_display = format!("{:.2}", priority);

    rsx! {
        div { class: "bg-[var(--surface-container-low)] border-2 border-[var(--outline-variant)] p-6",
            span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--warning)] mb-4", "TASK_PRIORITY" }
            div { class: "flex items-center justify-between mb-2",
                span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]", "WEIGHTING_INDEX" }
                span { class: "text-sm font-semibold text-[var(--on-surface)]", "{priority_display}" }
            }
            input {
                r#type: "range",
                min: "0",
                max: "1",
                step: "0.01",
                value: "{priority}",
                class: "w-full accent-[var(--warning)] mb-3",
                oninput: move |evt| {
                    if let Ok(v) = evt.value().parse::<f64>() {
                        settings.data.write().task_priority = v;
                    }
                },
            }
            div { class: "flex justify-between text-[10px] text-[var(--outline)] mb-4",
                span { "LOW_LATENCY" }
                span { "HIGH_THROUGHPUT" }
            }
            div { class: "flex flex-col gap-2",
                label { class: "flex items-center gap-2 text-xs text-[var(--on-surface-variant)] cursor-pointer",
                    input {
                        r#type: "checkbox",
                        checked: auto_scale,
                        class: "w-4 h-4 accent-[var(--warning)]",
                        onchange: move |_| {
                            let mut d = settings.data.write();
                            d.auto_scale = !d.auto_scale;
                        },
                    }
                    "AUTO-SCALE_RESOURCES"
                }
                label { class: "flex items-center gap-2 text-xs text-[var(--on-surface-variant)] cursor-pointer",
                    input {
                        r#type: "checkbox",
                        checked: redundant_verify,
                        class: "w-4 h-4 accent-[var(--warning)]",
                        onchange: move |_| {
                            let mut d = settings.data.write();
                            d.redundant_verify = !d.redundant_verify;
                        },
                    }
                    "REDUNDANT_VERIFICATION"
                }
            }
        }
    }
}
```

Keep the `ArchitectCard` and `SystemNotice` components unchanged — they are display-only cards that don't need settings wiring.

**ActiveForm:** Wiring TaskPriorityPanel slider and checkboxes to SettingsState

---

### Task 5: Wire QuotasPanel to SettingsState signals

**Subject:** Replace hardcoded quota bars with signal-driven configurable values

**Description:** Rewrite `crates/lx-desktop/src/pages/settings/quotas.rs`. Remove the `const QUOTAS` hardcoded data and the `Quota` struct.

The new implementation reads quota percentages from `SettingsState`:

```rust
use dioxus::prelude::*;
use super::state::SettingsState;

struct QuotaDisplay {
    label: &'static str,
    percent: u8,
    min_label: &'static str,
    max_label: &'static str,
}

#[component]
pub fn QuotasPanel() -> Element {
    let settings = use_context::<SettingsState>();
    let data = settings.data.read();
    let quotas = [
        QuotaDisplay { label: "COMPUTE_CORE", percent: data.compute_quota, min_label: "0.0 GHz/s", max_label: "12.4 GHz/s" },
        QuotaDisplay { label: "MEMORY_BUFFER", percent: data.memory_quota, min_label: "0.0 GB", max_label: "64.0 GB" },
        QuotaDisplay { label: "STORAGE_IO", percent: data.storage_quota, min_label: "0.0 MB/s", max_label: "1.0 GB/s" },
    ];
    drop(data);

    rsx! {
        div { class: "space-y-4",
            span { class: "text-xs uppercase tracking-wider font-semibold text-white border-l-4 border-[var(--warning)] pl-3", "RESOURCE_QUOTAS" }
            div { class: "flex gap-3",
                for quota in quotas.iter() {
                    {
                        let pct_str = format!("{}%", quota.percent);
                        let width = format!("width: {}%;", quota.percent);
                        let color = if quota.percent > 90 { "bg-[var(--error)]" } else if quota.percent > 70 { "bg-[var(--warning)]" } else { "bg-[var(--primary)]" };
                        let overload = quota.percent > 90;
                        rsx! {
                            div { class: "flex-1 bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-4",
                                div { class: "flex items-center justify-between mb-2",
                                    span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]", "{quota.label}" }
                                    if overload {
                                        span { class: "text-[10px] uppercase tracking-wider text-[var(--error)] font-semibold", "OVERLOAD" }
                                    } else {
                                        span { class: "text-sm font-semibold text-[var(--on-surface)]", "{pct_str}" }
                                    }
                                }
                                div { class: "h-2 bg-[var(--surface-container)] rounded-full overflow-hidden mb-2",
                                    div { class: "h-full {color} rounded-full", style: "{width}" }
                                }
                                div { class: "flex justify-between text-[10px] text-[var(--outline)]",
                                    span { "{quota.min_label}" }
                                    span { "{quota.max_label}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

The color and overload threshold are now derived dynamically from the quota percentage rather than being separate hardcoded fields. Quotas above 90% show red with OVERLOAD; 70-90% show warning; below 70% show primary.

**ActiveForm:** Wiring QuotasPanel to SettingsState signals

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_SETTINGS_PAGE.md" })
```
