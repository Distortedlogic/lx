# WU-09: Model picker for agent config

## Fixes
- Fix 1: Define a `MODEL_OPTIONS` constant with known model identifiers
- Fix 2: Import the `Select` and `SelectOption` components into config_form.rs
- Fix 3: Replace the plain `input` element for model with a `Select` component using `searchable: true`
- Fix 4: Ensure the Select allows arbitrary/unknown models by including the current model value in the options if not already present
- Fix 5: Preserve the dirty-tracking and save behavior when model selection changes

## Files Modified
- `crates/lx-desktop/src/pages/agents/config_form.rs` (154 lines)

## Preconditions
- `config_form.rs` at line 56-64 has a plain `input` element for model selection with `INPUT_FIELD` class, `oninput` handler setting `model` signal and `dirty` flag
- `Select` component at `crates/lx-desktop/src/components/ui/select.rs` (verified API):
  - Component props: `class: String` (default empty), `value: String`, `options: Vec<SelectOption>`, `placeholder: String` (default empty), `disabled: bool` (default false), `searchable: bool` (default false), `onchange: EventHandler<String>`
  - `SelectOption` struct has public fields: `pub value: String`, `pub label: String`, `pub disabled: bool`
  - `SelectOption::new(value: impl Into<String>, label: impl Into<String>) -> Self` — sets `disabled: false`
  - The `onchange` handler fires with `String` (the selected option's value), not `FormEvent`
- `ADAPTER_LABELS` is imported from `super::types` at line 1; the Select component must be imported from `crate::components::ui::select`
- Current model signal is initialized from `config.model.clone()` at line 14

## Steps

### Step 1: Add MODEL_OPTIONS constant to config_form.rs
- Open `crates/lx-desktop/src/pages/agents/config_form.rs`
- After line 3 (`use dioxus::prelude::*;`), add the import for Select and the constant:

```rust
use crate::components::ui::select::{Select, SelectOption};

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
```

- Why: A predefined list of known models enables the searchable dropdown UX

### Step 2: Replace the model input with a Select component
- Open `crates/lx-desktop/src/pages/agents/config_form.rs`
- Find lines 55-64:

```rust
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
```

Replace with:

```rust
          label { class: "text-xs text-[var(--outline)] block", "Model" }
          Select {
            class: "w-full".to_string(),
            value: model.read().clone(),
            searchable: true,
            placeholder: "Select a model...".to_string(),
            options: {
              let cur = model.read().clone();
              let mut opts: Vec<SelectOption> = MODEL_OPTIONS
                .iter()
                .map(|(v, l)| SelectOption::new(*v, *l))
                .collect();
              if !cur.is_empty() && !opts.iter().any(|o| o.value == cur) {
                opts.insert(0, SelectOption::new(cur.clone(), cur));
              }
              opts
            },
            onchange: move |val: String| {
              model.set(val);
              dirty.set(true);
            },
          }
```

- Why: The Select component with `searchable: true` provides a dropdown with type-to-filter, and the dynamic insertion of the current model value ensures that an unknown/custom model already set on the agent is still displayed and selectable

## File Size Check
- `config_form.rs`: was 154 lines, now ~175 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compile errors or warnings
- The model field should render as a searchable dropdown instead of a plain text input
- Typing in the search box should filter the model list
- If the agent's current model is not in MODEL_OPTIONS, it should still appear as the first option
- Selecting a model should mark the form dirty and enable the Save/Cancel buttons
- The save flow should still produce the correct `AgentConfigUpdate` with the selected model string
