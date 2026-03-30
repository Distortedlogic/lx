# UNIT 12: Onboarding Wizard Agent Step

## Goal

Replace the minimal agent name-only step with a substantive configuration step that collects:
agent name, role, description, and adapter type.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/components/onboarding/step_agent.rs` | Rewrite |
| `crates/lx-desktop/src/components/onboarding/wizard.rs` | Edit (add signals, pass props) |
| `crates/lx-desktop/src/components/onboarding/step_launch.rs` | Edit (show role + adapter in summary) |

## Current State

### step_agent.rs (34 lines)

Takes a single `agent_name: Signal<String>` prop. Renders one text input with placeholder "CEO".

### wizard.rs (197 lines)

Declares these signals (lines 55-59):
```rust
let mut company_name = use_signal(String::new);
let mut company_goal = use_signal(String::new);
let mut agent_name = use_signal(|| "CEO".to_string());
let mut task_title = use_signal(|| "Create a hiring plan".to_string());
let mut task_description = use_signal(String::new);
```

Passes only `agent_name` to StepAgent on line 103-104:
```rust
WizardStep::Agent => rsx! {
  StepAgent { agent_name }
},
```

Reset block (lines 63-73) resets agent_name to "CEO".

### step_launch.rs (51 lines)

Shows company_name, agent_name, task_title in summary rows. No role or adapter info.

### Adapter labels from `crates/lx-desktop/src/pages/agents/types.rs` lines 89-99:
```rust
pub const ADAPTER_LABELS: &[(&str, &str)] = &[
  ("claude_local", "Claude"),
  ("codex_local", "Codex"),
  ("gemini_local", "Gemini"),
  ("opencode_local", "OpenCode"),
  ("cursor", "Cursor"),
  ("hermes_local", "Hermes"),
  ("openclaw_gateway", "OpenClaw Gateway"),
  ("process", "Process"),
  ("http", "HTTP"),
];
```

### Role labels from `crates/lx-desktop/src/pages/agents/types.rs` lines 105-106:
```rust
pub const ROLE_LABELS: &[(&str, &str)] =
  &[("ceo", "CEO"), ("executive", "Executive"), ("manager", "Manager"), ("general", "General"), ("specialist", "Specialist")];
```

## Step 1: Replace `crates/lx-desktop/src/components/onboarding/step_agent.rs`

Replace the full file content with:

```rust
use dioxus::prelude::*;

use crate::pages::agents::types::{ADAPTER_LABELS, ROLE_LABELS};

const INPUT_CLS: &str = "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]";
const SELECT_CLS: &str = "w-full border border-[var(--outline-variant)] bg-[var(--surface-container)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)]";

#[component]
pub fn StepAgent(
  agent_name: Signal<String>,
  agent_role: Signal<String>,
  agent_description: Signal<String>,
  agent_adapter: Signal<String>,
) -> Element {
  rsx! {
    div { class: "space-y-5",
      div { class: "flex items-center gap-3 mb-1",
        div { class: "bg-[var(--surface-container-highest)] p-2",
          span { class: "material-symbols-outlined text-xl text-[var(--outline)]",
            "smart_toy"
          }
        }
        div {
          h3 { class: "text-sm font-medium text-[var(--on-surface)]",
            "Create your first agent"
          }
          p { class: "text-xs text-[var(--outline)]",
            "Configure the agent that will handle your first task."
          }
        }
      }
      div { class: "grid grid-cols-2 gap-3",
        div { class: "space-y-1",
          label { class: "text-xs text-[var(--outline)] block", "Agent name" }
          input {
            class: INPUT_CLS,
            placeholder: "CEO",
            value: "{agent_name}",
            oninput: move |e| agent_name.set(e.value()),
            autofocus: true,
          }
        }
        div { class: "space-y-1",
          label { class: "text-xs text-[var(--outline)] block", "Role" }
          select {
            class: SELECT_CLS,
            value: "{agent_role}",
            onchange: move |e| agent_role.set(e.value()),
            for (key , label) in ROLE_LABELS {
              option { value: *key, "{label}" }
            }
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Adapter" }
        select {
          class: SELECT_CLS,
          value: "{agent_adapter}",
          onchange: move |e| agent_adapter.set(e.value()),
          for (key , label) in ADAPTER_LABELS {
            option { value: *key, "{label}" }
          }
        }
      }
      div { class: "space-y-1",
        label { class: "text-xs text-[var(--outline)] block", "Description (optional)" }
        textarea {
          class: "{INPUT_CLS} resize-none min-h-[80px]",
          placeholder: "What should this agent focus on?",
          value: "{agent_description}",
          oninput: move |e| agent_description.set(e.value()),
        }
      }
    }
  }
}
```

## Step 2: Edit `crates/lx-desktop/src/components/onboarding/wizard.rs`

### 2a: Add three new signals after line 57

Find:
```rust
  let mut agent_name = use_signal(|| "CEO".to_string());
```

Replace with:
```rust
  let mut agent_name = use_signal(|| "CEO".to_string());
  let mut agent_role = use_signal(|| "ceo".to_string());
  let mut agent_description = use_signal(String::new);
  let mut agent_adapter = use_signal(|| "claude_local".to_string());
```

### 2b: Add resets for the new signals in the use_effect block

Find:
```rust
      agent_name.set("CEO".to_string());
```

Replace with:
```rust
      agent_name.set("CEO".to_string());
      agent_role.set("ceo".to_string());
      agent_description.set(String::new());
      agent_adapter.set("claude_local".to_string());
```

### 2c: Pass new props to StepAgent

Find:
```rust
            WizardStep::Agent => rsx! {
              StepAgent { agent_name }
            },
```

Replace with:
```rust
            WizardStep::Agent => rsx! {
              StepAgent { agent_name, agent_role, agent_description, agent_adapter }
            },
```

### 2d: Pass new data to StepLaunch

Find:
```rust
            WizardStep::Launch => rsx! {
              StepLaunch {
                company_name: company_name.read().clone(),
                agent_name: agent_name.read().clone(),
                task_title: task_title.read().clone(),
              }
            },
```

Replace with:
```rust
            WizardStep::Launch => rsx! {
              StepLaunch {
                company_name: company_name.read().clone(),
                agent_name: agent_name.read().clone(),
                agent_role: agent_role.read().clone(),
                agent_adapter: agent_adapter.read().clone(),
                task_title: task_title.read().clone(),
              }
            },
```

## Step 3: Edit `crates/lx-desktop/src/components/onboarding/step_launch.rs`

### 3a: Update StepLaunch component signature and add role/adapter rows

Find:
```rust
#[component]
pub fn StepLaunch(company_name: String, agent_name: String, task_title: String) -> Element {
```

Replace with:
```rust
#[component]
pub fn StepLaunch(company_name: String, agent_name: String, agent_role: String, agent_adapter: String, task_title: String) -> Element {
```

### 3b: Add summary rows for role and adapter

Find:
```rust
        SummaryRow { icon: "smart_toy", label: "Agent", value: agent_name }
        SummaryRow { icon: "checklist", label: "Task", value: task_title }
```

Replace with:
```rust
        SummaryRow { icon: "smart_toy", label: "Agent", value: agent_name }
        SummaryRow { icon: "badge", label: "Role", value: agent_role }
        SummaryRow { icon: "memory", label: "Adapter", value: agent_adapter }
        SummaryRow { icon: "checklist", label: "Task", value: task_title }
```

## Verification

Run `just diagnose` and confirm no compiler errors in `crates/lx-desktop`.
