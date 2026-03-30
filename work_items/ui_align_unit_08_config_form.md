# UNIT 8: Wire AgentConfigForm Save/Cancel to State Mutation

## Goal

The `AgentConfigPanel` in `config_form.rs` has Save and Cancel buttons that currently only toggle the `dirty` signal to `false`. Wire Save to call an `on_save` callback with the modified config data, and wire Cancel to reset all form signals to their original values.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/pages/agents/config_form.rs` | Add `on_save`/`on_cancel` props, wire button handlers, reset on cancel |
| `crates/lx-desktop/src/pages/agents/detail.rs` | Pass `on_save`/`on_cancel` callbacks to AgentConfigPanel |

## Reference Files (read-only)

| File | Why |
|------|-----|
| `crates/lx-desktop/src/pages/agents/types.rs` | `AgentDetail` struct with `adapter_type`, `adapter_config`, `runtime_config` fields |
| `crates/lx-desktop/src/styles.rs` | `BTN_OUTLINE_SM`, `BTN_PRIMARY_SM`, `INPUT_FIELD` constants |

---

## Current State

### `config_form.rs` lines 73-87 (Save/Cancel buttons)
```rust
      if *dirty.read() {
        div { class: "flex items-center justify-end gap-2 pt-4 border-t border-[var(--outline-variant)]/30",
          button {
            class: BTN_OUTLINE_SM,
            onclick: move |_| dirty.set(false),
            "Cancel"
          }
          button {
            class: BTN_PRIMARY_SM,
            onclick: move |_| dirty.set(false),
            "Save"
          }
        }
      }
```

### `detail.rs` lines 85-87
```rust
          AgentDetailTab::Config => rsx! {
            AgentConfigPanel { agent: agent.clone() }
          },
```

---

## Data Type for Config Changes

The save callback needs to communicate which fields changed. Define a struct in `config_form.rs` to carry the updated values.

---

## Step 1: Rewrite `config_form.rs`

Replace the entire file `crates/lx-desktop/src/pages/agents/config_form.rs` with:

```rust
use super::types::{ADAPTER_LABELS, AgentDetail};
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
  let bg = if checked { "bg-green-600" } else { "bg-[var(--outline-variant)]" };
  let translate = if checked { "translate-x-4" } else { "translate-x-0.5" };
  rsx! {
    button {
      class: "relative inline-flex h-5 w-9 items-center rounded-full transition-colors shrink-0 {bg}",
      onclick: move |_| on_toggle.call(!checked),
      span { class: "inline-block h-3.5 w-3.5 rounded-full bg-white transition-transform {translate}" }
    }
  }
}
```

**What changed vs. the original `config_form.rs`:**

1. Added `ConfigUpdate` struct (lines 6-11) to carry saved values.
2. `AgentConfigPanel` signature: added `#[props(optional)] on_save: Option<EventHandler<ConfigUpdate>>` and `#[props(optional)] on_cancel: Option<EventHandler<()>>`.
3. Extracted `original_*` variables from the `agent` prop at the top of the function body, before creating signals.
4. Cancel button handler: resets all four signals (`adapter_type`, `model`, `heartbeat_enabled`, `interval_sec`) back to `original_*` values, sets `dirty` to false, and calls `on_cancel` if provided.
5. Save button handler: reads current signal values into a `ConfigUpdate`, sets `dirty` to false, and calls `on_save` if provided.
6. Both handlers are `Option` so existing call sites without callbacks compile unchanged.

---

## Step 2: Pass callbacks from `detail.rs`

In `crates/lx-desktop/src/pages/agents/detail.rs`:

### Change 2a: Add import for ConfigUpdate

Old text (lines 2-2):
```rust
use super::config_form::AgentConfigPanel;
```

New text:
```rust
use super::config_form::{AgentConfigPanel, ConfigUpdate};
```

### Change 2b: Wire on_save with ActivityLog logging

Old text (lines 85-87):
```rust
          AgentDetailTab::Config => rsx! {
            AgentConfigPanel { agent: agent.clone() }
          },
```

New text:
```rust
          AgentDetailTab::Config => rsx! {
            AgentConfigPanel {
              agent: agent.clone(),
              on_save: move |update: ConfigUpdate| {
                  log.push("config_save", &format!("{}: adapter={}, model={}", agent.name, update.adapter_type, update.model));
              },
              on_cancel: move |_| {},
            }
          },
```

This logs a `config_save` event to ActivityLog when the user saves. The `on_cancel` handler is a no-op (the form already resets its own signals). In a future unit, `on_save` will also mutate the agent's in-memory state or persist to backend.

---

## Verification

After all changes:
- `config_form.rs` is ~127 lines (under 300).
- `detail.rs` grows by ~5 lines to ~180 (under 300).
- No code comments or docstrings.
- No `#[allow(...)]` macros.
- Cancel resets form fields to their values at component mount time.
- Save packages current field values into `ConfigUpdate` and fires the callback.
- Existing call sites without `on_save`/`on_cancel` (if any exist elsewhere) continue to compile due to `#[props(optional)]`.
