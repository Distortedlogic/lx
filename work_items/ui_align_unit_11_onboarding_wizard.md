# Unit 11: OnboardingWizard redesign for lx

## Goal

Redesign the onboarding wizard for lx-specific concepts: add model ID input on the agent step, update task step copy to reference lx concepts, show lx-specific summary on launch, add Cmd+Enter step advancement, and add per-step loading spinners.

## Preconditions

- No other unit dependencies
- CSS variables from `tailwind.css` are available (`--surface-container`, `--on-surface`, `--outline`, `--primary`, `--on-primary`, `--outline-variant`, `--surface-container-highest`, `--error`, `--success`)
- Material Symbols Outlined font is loaded (for spinner icon `progress_activity` and other icons)

## Files to Modify

- `crates/lx-desktop/src/components/onboarding/wizard.rs`
- `crates/lx-desktop/src/components/onboarding/step_agent.rs`
- `crates/lx-desktop/src/components/onboarding/step_task.rs`
- `crates/lx-desktop/src/components/onboarding/step_launch.rs`

## Steps

### 1. Add model_id signal to wizard.rs

In `crates/lx-desktop/src/components/onboarding/wizard.rs`, inside the `OnboardingWizard` component, add a new signal after the `agent_adapter` signal (line 60):

```rust
let mut agent_model_id = use_signal(|| "claude-sonnet-4-20250514".to_string());
```

In the `use_effect` reset block (the one that fires when `!is_open`), add after `agent_adapter.set("claude_local".to_string());`:

```rust
agent_model_id.set("claude-sonnet-4-20250514".to_string());
```

### 2. Pass model_id to StepAgent

Update the `StepAgent` invocation in wizard.rs from:

```rust
StepAgent { agent_name, agent_role, agent_description, agent_adapter }
```

to:

```rust
StepAgent { agent_name, agent_role, agent_description, agent_adapter, agent_model_id }
```

### 3. Pass model_id to StepLaunch

Update the `StepLaunch` invocation in wizard.rs from:

```rust
StepLaunch {
  company_name: company_name.read().clone(),
  agent_name: agent_name.read().clone(),
  agent_role: agent_role.read().clone(),
  agent_adapter: agent_adapter.read().clone(),
  task_title: task_title.read().clone(),
}
```

to:

```rust
StepLaunch {
  company_name: company_name.read().clone(),
  agent_name: agent_name.read().clone(),
  agent_role: agent_role.read().clone(),
  agent_adapter: agent_adapter.read().clone(),
  agent_model_id: agent_model_id.read().clone(),
  task_title: task_title.read().clone(),
}
```

### 4. Add Cmd+Enter keyboard shortcut to advance steps

In `crates/lx-desktop/src/components/onboarding/wizard.rs`, add an `onkeydown` handler to the modal container div (the `div` with class `"fixed top-[10%] left-1/2 ..."`). Place it after the `onclick: move |e| e.stop_propagation()` line:

```rust
onkeydown: move |evt: KeyboardEvent| {
    if evt.modifiers().meta() && evt.key() == Key::Enter {
        match *step.read() {
            WizardStep::Company => step.set(WizardStep::Agent),
            WizardStep::Agent => step.set(WizardStep::Task),
            WizardStep::Task => step.set(WizardStep::Launch),
            WizardStep::Launch => {
                onboarding.close_wizard();
            }
        }
    }
},
```

### 5. Add per-step loading text with spinner

In `crates/lx-desktop/src/components/onboarding/wizard.rs`, add a `loading_text` signal:

```rust
let mut loading_text = use_signal(|| Option::<String>::None);
```

In the reset `use_effect`, add:

```rust
loading_text.set(None);
```

In the `WizardFooter` component, update the signature to accept `loading_text`:

```rust
#[component]
fn WizardFooter(step: Signal<WizardStep>, loading: Signal<bool>, loading_text: Signal<Option<String>>, onboarding: OnboardingCtx) -> Element {
```

Update the `WizardFooter` invocation in `OnboardingWizard` to pass the new prop:

```rust
WizardFooter { step, loading, loading_text, onboarding }
```

In the `WizardFooter` render, replace the existing `if *loading.read() { "Working..." }` block with:

```rust
if *loading.read() {
    div { class: "flex items-center gap-2",
        span { class: "material-symbols-outlined text-sm animate-spin", "progress_activity" }
        {
            let text = loading_text.read().as_ref().cloned().unwrap_or_else(|| "Working...".into());
            rsx! { span { "{text}" } }
        }
    }
}
```

Note: The `animate-spin` class is provided by Tailwind v4. No CSS addition needed.

### 6. Update step_agent.rs to add model ID field

In `crates/lx-desktop/src/components/onboarding/step_agent.rs`, update the component signature to accept `agent_model_id`:

```rust
#[component]
pub fn StepAgent(agent_name: Signal<String>, agent_role: Signal<String>, agent_description: Signal<String>, agent_adapter: Signal<String>, agent_model_id: Signal<String>) -> Element {
```

After the Adapter `select` block (lines 50-59), add a new model ID input field:

```rust
div { class: "space-y-1",
    label { class: "text-xs text-[var(--outline)] block", "Model ID" }
    input {
        class: INPUT_CLS,
        placeholder: "claude-sonnet-4-20250514",
        value: "{agent_model_id}",
        oninput: move |e| agent_model_id.set(e.value()),
    }
    p { class: "text-[10px] text-[var(--outline)]/60 mt-0.5",
        "The model identifier your adapter will use (e.g. claude-sonnet-4-20250514, gpt-4o, gemini-2.0-flash)"
    }
}
```

Update the header description text from:

```rust
p { class: "text-xs text-[var(--outline)]",
    "Configure the agent that will handle your first task."
}
```

to:

```rust
p { class: "text-xs text-[var(--outline)]",
    "Configure the agent that will run your first lx flow."
}
```

### 7. Update step_task.rs with lx-specific copy

In `crates/lx-desktop/src/components/onboarding/step_task.rs`, update the header text.

Change the `h3` from:

```rust
h3 { class: "text-sm font-medium text-[var(--on-surface)]",
    "Give it something to do"
}
```

to:

```rust
h3 { class: "text-sm font-medium text-[var(--on-surface)]",
    "Define your first flow"
}
```

Change the `p` description from:

```rust
p { class: "text-xs text-[var(--outline)]",
    "Give your agent a small task to start with."
}
```

to:

```rust
p { class: "text-xs text-[var(--outline)]",
    "An lx flow orchestrates agents, channels, and tools to complete work."
}
```

Change the task title label from `"Task title"` to `"Flow name"`.

Change the task title placeholder from `"e.g. Research competitor pricing"` to `"e.g. Research competitor pricing"` -- actually, update it to:

```
"e.g. analyze-codebase, draft-proposal, review-pr"
```

Change the description label from `"Description (optional)"` to `"Flow description (optional)"`.

Change the description placeholder from `"Add more detail about what the agent should do..."` to:

```
"Describe what this flow should accomplish. The agent will use this to plan its steps."
```

### 8. Update step_launch.rs with lx-specific summary

In `crates/lx-desktop/src/components/onboarding/step_launch.rs`, update the component signature to accept `agent_model_id`:

```rust
#[component]
pub fn StepLaunch(company_name: String, agent_name: String, agent_role: String, agent_adapter: String, agent_model_id: String, task_title: String) -> Element {
```

Update the header description from:

```rust
p { class: "text-xs text-[var(--outline)]",
    "Everything is set up. Launch will create the task and open it."
}
```

to:

```rust
p { class: "text-xs text-[var(--outline)]",
    "Launch will generate an lx flow file and start execution."
}
```

Add a new `SummaryRow` for the model ID between the Adapter and Task rows:

```rust
SummaryRow { icon: "model_training", label: "Model", value: agent_model_id }
```

Change the Task row icon and label to reflect lx terminology:

From:
```rust
SummaryRow { icon: "checklist", label: "Task", value: task_title }
```

To:
```rust
SummaryRow { icon: "account_tree", label: "Flow", value: task_title }
```

### 9. Verify all files stay under 300 lines

Expected line counts after changes:
- `wizard.rs`: ~215 lines (was 205, added ~10 lines for model_id signal, loading_text, keyboard handler)
- `step_agent.rs`: ~85 lines (was 72, added ~13 lines for model ID field)
- `step_task.rs`: ~44 lines (was 43, minimal text changes)
- `step_launch.rs`: ~55 lines (was 54, added one SummaryRow + prop)

All well under 300 lines.

## Verification

1. Run `just diagnose` -- must compile with no warnings and no clippy errors.
2. Open the desktop app and trigger the onboarding wizard.
3. Verify Company step: unchanged -- name and goal fields work as before.
4. Verify Agent step:
   - Header now says "Configure the agent that will run your first lx flow."
   - All original fields present: Agent name, Role (select), Adapter (select), Description (textarea).
   - New "Model ID" text input appears after Adapter, pre-filled with `claude-sonnet-4-20250514`.
   - Helper text appears below Model ID field.
5. Verify Task step:
   - Header says "Define your first flow" with lx-specific description.
   - Labels read "Flow name" and "Flow description (optional)".
   - Placeholders reference lx concepts.
6. Verify Launch step:
   - Header says "Launch will generate an lx flow file and start execution."
   - Summary shows 6 rows: Company, Agent, Role, Adapter, Model, Flow.
   - Model row shows the entered model ID.
   - Flow row shows the flow name.
7. Verify Cmd+Enter advances through each step (Company -> Agent -> Task -> Launch -> closes wizard).
8. Verify the Next/Create & Launch button shows a spinning `progress_activity` icon when `loading` is true (can verify by temporarily setting `loading.set(true)` or by inspecting the code path).
9. All modified files are under 300 lines.
