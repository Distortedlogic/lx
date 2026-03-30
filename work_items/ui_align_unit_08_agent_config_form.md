# Unit 08: AgentConfigForm redesign for lx

## Goal
Redesign the agent configuration panel to show lx-native data: the agent's .lx source definition, model/backend configuration, tool declarations from `use` blocks, and channel subscriptions -- reading from lx program AST data instead of a REST API.

## Preconditions
- No other units required first
- `crates/lx-desktop/src/pages/agents/config_form.rs` exists (152 lines)
- `crates/lx-desktop/src/pages/agents/types.rs` exists (122 lines) with `AgentDetail`, `ADAPTER_LABELS`
- `crates/lx-ast/src/ast/types.rs` has `KeywordDeclData` (agent declarations with `keyword: KeywordKind::Agent`, `name`, `fields`, `methods`, `uses`)
- `crates/lx-ast/src/ast/types.rs` has `UseStmt` with `UseKind::Tool { command, alias }`
- `crates/lx-ast/src/ast/mod.rs` has `Stmt::ChannelDecl(Sym)`, `Stmt::Use(UseStmt)`, `Stmt::KeywordDecl(KeywordDeclData)`

## Files to Modify
- `crates/lx-desktop/src/pages/agents/config_form.rs` (rewrite)
- `crates/lx-desktop/src/pages/agents/types.rs` (add new data types)

## Current State

`config_form.rs` has `AgentConfigPanel` that takes an `AgentDetail` (REST-style data with `adapter_config` and `runtime_config` as `serde_json::Value`). It renders 3 sections: Adapter (dropdown + model input), Heartbeat (toggle + interval), and a dirty-tracking save/cancel bar. It also has `ConfigSection`, `ToggleSwitch` helper components.

For lx-desktop, the config form should instead display data derived from the agent's `.lx` source program, not from REST API JSON.

## Steps

### Step 1: Define LxAgentConfig in types.rs

Add a new struct in `crates/lx-desktop/src/pages/agents/types.rs` that represents the agent config as parsed from an lx program:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LxAgentConfig {
    pub name: String,
    pub source_text: String,
    pub adapter_type: String,
    pub model: String,
    pub tools: Vec<LxToolDecl>,
    pub channels: Vec<String>,
    pub fields: Vec<LxAgentField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LxToolDecl {
    pub path: String,
    pub alias: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LxAgentField {
    pub name: String,
    pub value: String,
}
```

- `source_text`: the raw `.lx` source for this agent declaration (for the read-only code view)
- `tools`: extracted from `Stmt::Use(UseStmt { kind: UseKind::Tool { command, alias }, .. })` statements
- `channels`: extracted from `Stmt::ChannelDecl(sym)` statements
- `fields`: extracted from `KeywordDeclData.fields` (the `ClassField` entries with name and default expression rendered as string)

### Step 2: Rewrite AgentConfigPanel signature

Replace the existing component in `config_form.rs`:

```rust
use super::types::{ADAPTER_LABELS, LxAgentConfig, LxToolDecl, LxAgentField};
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM, INPUT_FIELD};
use dioxus::prelude::*;

#[component]
pub fn AgentConfigPanel(
    config: LxAgentConfig,
    #[props(optional)] on_save: Option<EventHandler<AgentConfigUpdate>>,
) -> Element {
```

### Step 3: Define AgentConfigUpdate

Keep a minimal update struct for the editable fields only (model and adapter):

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct AgentConfigUpdate {
    pub adapter_type: String,
    pub model: String,
}
```

Delete the old `ConfigUpdate` struct.

### Step 4: Internal state for editable fields

```rust
let mut adapter_type = use_signal(|| config.adapter_type.clone());
let mut model = use_signal(|| config.model.clone());
let mut dirty = use_signal(|| false);
```

### Step 5: Render Section 1 -- Source Definition

This is a read-only monospace `pre` block showing the agent's `.lx` source text. Render it first:

```rust
rsx! {
    div { class: "max-w-3xl space-y-6",
        ConfigSection { title: "Source Definition",
            div { class: "relative",
                pre {
                    class: "text-xs font-mono leading-relaxed text-[var(--on-surface)] bg-[var(--surface)] border border-[var(--outline-variant)]/30 rounded p-4 overflow-x-auto max-h-80 overflow-y-auto whitespace-pre",
                    "{config.source_text}"
                }
                button {
                    class: "absolute top-2 right-2 text-xs text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
                    title: "Copy source",
                    onclick: move |_| {
                        // copy to clipboard via eval
                    },
                    span { class: "material-symbols-outlined text-sm", "content_copy" }
                }
            }
        }
```

The `pre` block uses:
- `text-xs font-mono leading-relaxed` for code appearance
- `bg-[var(--surface)]` for slight contrast against the form background
- `max-h-80 overflow-y-auto` to cap height at ~320px and scroll
- `whitespace-pre` to preserve indentation (not `whitespace-pre-wrap` since lx code should not wrap)

### Step 6: Render Section 2 -- Model & Backend

This section has two editable fields: adapter dropdown and model text input. Reuse the existing pattern from the old form but with the new signal names:

```rust
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
                    for (key, label) in ADAPTER_LABELS {
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
```

Note: This uses a native `<select>` for now. If Unit 06 (custom Select) is done first, use the custom `Select` component with `SelectOption` instead. Either way works -- this unit should not depend on Unit 06.

### Step 7: Render Section 3 -- Tool Declarations

A read-only table/list of tools extracted from the agent's `use` blocks:

```rust
        ConfigSection { title: "Tools",
            if config.tools.is_empty() {
                div { class: "text-sm text-[var(--outline)] italic", "No tools declared" }
            } else {
                div { class: "space-y-1",
                    for tool in config.tools.iter() {
                        div { class: "flex items-center gap-3 py-1.5 border-b border-[var(--outline-variant)]/20 last:border-b-0",
                            span { class: "material-symbols-outlined text-sm text-[var(--outline)]", "build" }
                            span { class: "text-sm font-mono text-[var(--on-surface)]", "{tool.path}" }
                            if tool.alias != tool.path {
                                span { class: "text-xs text-[var(--outline)]", "as" }
                                span { class: "text-sm font-mono text-[var(--primary)]", "{tool.alias}" }
                            }
                        }
                    }
                }
            }
        }
```

Each tool row shows:
- Wrench icon (`build`)
- Tool path in monospace
- If aliased, show `as <alias>` with the alias in primary color

### Step 8: Render Section 4 -- Channel Subscriptions

A read-only list of channels this agent subscribes to:

```rust
        ConfigSection { title: "Channels",
            if config.channels.is_empty() {
                div { class: "text-sm text-[var(--outline)] italic", "No channel subscriptions" }
            } else {
                div { class: "flex flex-wrap gap-2",
                    for ch in config.channels.iter() {
                        span {
                            class: "inline-flex items-center gap-1.5 rounded border border-[var(--outline-variant)]/30 bg-[var(--surface)] px-2.5 py-1 text-xs font-mono text-[var(--on-surface)]",
                            span { class: "material-symbols-outlined text-xs text-[var(--outline)]", "tag" }
                            "{ch}"
                        }
                    }
                }
            }
        }
```

Channels render as pill/badge elements with a `tag` icon and the channel name in monospace.

### Step 9: Render Section 5 -- Agent Fields

If the agent declaration has fields (from `KeywordDeclData.fields`), show them:

```rust
        if !config.fields.is_empty() {
            ConfigSection { title: "Fields",
                div { class: "space-y-2",
                    for field in config.fields.iter() {
                        div { class: "flex items-baseline gap-3",
                            span { class: "text-xs font-mono text-[var(--outline)] w-28 shrink-0", "{field.name}" }
                            span { class: "text-sm font-mono text-[var(--on-surface)]", "{field.value}" }
                        }
                    }
                }
            }
        }
```

### Step 10: Render the save/cancel bar

Keep the dirty-tracking save bar from the original, but only for the editable fields (adapter + model):

```rust
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
```

### Step 11: Keep ConfigSection and ToggleSwitch helpers

`ConfigSection` stays unchanged (lines 128-138 of original). `ToggleSwitch` can be removed since heartbeat config is gone. If `ToggleSwitch` is used elsewhere in the codebase, keep it; if not, delete it.

Search for `ToggleSwitch` usage: `grep -r "ToggleSwitch" crates/lx-desktop/src/`. If only used in `config_form.rs`, delete it.

### Step 12: Update call sites

Find where `AgentConfigPanel` is rendered. It will be in the agent detail page (likely `pages/agents/detail.rs` or similar). The caller must now construct a `LxAgentConfig` instead of passing `AgentDetail`. For now, the caller can build a placeholder:

```rust
let config = LxAgentConfig {
    name: agent.name.clone(),
    source_text: format!("agent {} {{\n  -- source not yet loaded\n}}", agent.name),
    adapter_type: agent.adapter_type.clone(),
    model: agent.adapter_config.get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string(),
    tools: vec![],
    channels: vec![],
    fields: vec![],
};
```

This placeholder will be replaced when the desktop app gains the ability to parse `.lx` files and extract agent declarations from the AST.

### Step 13: File length check

The rewritten `config_form.rs` should contain:
- `AgentConfigUpdate` struct (~5 lines)
- `AgentConfigPanel` component (~120 lines for all 5 sections + save bar)
- `ConfigSection` helper (~10 lines)

Total: approximately 135-150 lines, well under 300.

`types.rs` additions (~20 lines for `LxAgentConfig`, `LxToolDecl`, `LxAgentField`) bring it to ~142 lines, well under 300.

## Verification
1. Run `just diagnose` -- must compile with no errors or warnings
2. Launch the app, navigate to any agent's detail page, select the Config tab
3. Section 1 (Source Definition): shows a monospace code block with the agent's lx source text, scrollable, with a copy button in the top-right
4. Section 2 (Model & Backend): shows adapter dropdown and model text input, both editable
5. Section 3 (Tools): shows a list of tool declarations with wrench icons, or "No tools declared" if empty
6. Section 4 (Channels): shows channel badges with tag icons, or "No channel subscriptions" if empty
7. Section 5 (Fields): appears only when agent has fields; shows name-value pairs in monospace
8. Edit adapter or model -- "Save" and "Cancel" buttons appear
9. Click "Cancel" -- fields revert, buttons disappear
10. Click "Save" -- `on_save` fires with `AgentConfigUpdate`, buttons disappear
11. No `ConfigUpdate` struct remains (old one deleted)
12. No heartbeat section remains
13. Both modified files stay under 300 lines
