# Unit 18: Onboarding Wizard & Polish

## Scope

Port the multi-step onboarding wizard from Paperclip React to Dioxus 0.7.3 in lx-desktop. The wizard guides new users through 4 steps: (1) create company, (2) configure first agent, (3) define first task, (4) review and launch. After the wizard, perform a final integration pass ensuring all Units 1-18 components compose correctly in the Shell layout.

The Paperclip `OnboardingWizard.tsx` is 1403 lines. The Dioxus port must be split across multiple files to respect the 300-line limit. The wizard is simplified from the Paperclip version: adapter type selection is reduced to the lx-relevant adapters (no Codex/Gemini/Cursor/OpenCode/Pi/Hermes/OpenClaw), the adapter environment test is omitted, and the ASCII art animation is omitted.

## Paperclip Source Files

| Paperclip file | Purpose |
|---|---|
| `reference/paperclip/ui/src/components/OnboardingWizard.tsx` | 4-step wizard: company, agent, task, launch (1403 lines) |
| `reference/paperclip/ui/src/lib/onboarding-goal.ts` | Parse goal input into title + description |
| `reference/paperclip/ui/src/lib/onboarding-launch.ts` | Build project/issue payloads, select default goal |
| `reference/paperclip/ui/src/lib/onboarding-route.ts` | Detect /onboarding path, resolve options |
| `reference/paperclip/ui/src/context/DialogContext.tsx` | Provides `openOnboarding` / `closeOnboarding` |

## Preconditions

- Unit 16 complete: `crates/lx-desktop/src/components/mod.rs` exists
- Unit 17 complete: `crates/lx-desktop/src/api/` exists with client, companies, agents, issues, projects, goals modules
- **Unit 3 is complete:** Unit 3 created a stub `pages/onboarding.rs`. This unit replaces it with a real onboarding module. The `routes.rs` Route enum already has an `Onboarding {}` variant importing from `crate::pages::onboarding` -- no changes to `routes.rs` are needed.
- `crates/lx-desktop/src/layout/shell.rs` exists with `Shell` component
- `crates/lx-desktop/src/contexts/mod.rs` exists

## Files Affected

| File | Action |
|---|---|
| `crates/lx-desktop/src/components/onboarding/mod.rs` | Create: module declarations + shared types |
| `crates/lx-desktop/src/components/onboarding/wizard.rs` | Create: OnboardingWizard top-level component |
| `crates/lx-desktop/src/components/onboarding/step_company.rs` | Create: Step 1 - company creation form |
| `crates/lx-desktop/src/components/onboarding/step_agent.rs` | Create: Step 2 - agent configuration form |
| `crates/lx-desktop/src/components/onboarding/step_task.rs` | Create: Step 3 - task definition form |
| `crates/lx-desktop/src/components/onboarding/step_launch.rs` | Create: Step 4 - review and launch |
| `crates/lx-desktop/src/components/onboarding/helpers.rs` | Create: onboarding helper functions (goal parsing, payload builders) |
| `crates/lx-desktop/src/components/mod.rs` | Modify: add `pub mod onboarding;` |
| `crates/lx-desktop/src/contexts/onboarding.rs` | Create: OnboardingState context (open/close, options) |
| `crates/lx-desktop/src/contexts/mod.rs` | Modify: add `pub mod onboarding;` |
| `crates/lx-desktop/src/layout/shell.rs` | Modify: add OnboardingWizard to Shell, provide onboarding context |

## Tasks

### 1. Create `crates/lx-desktop/src/contexts/onboarding.rs`

Provides the onboarding open/close state that the wizard and Shell share. Port from `DialogContext.tsx` (only the onboarding-related subset).

```rust
use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardingInitialStep {
    Company,
    Agent,
}

impl OnboardingInitialStep {
    pub fn index(&self) -> u8 {
        match self {
            Self::Company => 1,
            Self::Agent => 2,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct OnboardingOptions {
    pub initial_step: Option<OnboardingInitialStep>,
    pub company_id: Option<String>,
}

#[derive(Clone, Copy)]
pub struct OnboardingCtx {
    pub open: Signal<bool>,
    pub options: Signal<OnboardingOptions>,
}

impl OnboardingCtx {
    pub fn provide() -> Self {
        let ctx = Self {
            open: Signal::new(false),
            options: Signal::new(OnboardingOptions::default()),
        };
        use_context_provider(|| ctx);
        ctx
    }

    pub fn open_wizard(&self, opts: OnboardingOptions) {
        self.options.set(opts);
        self.open.set(true);
    }

    pub fn close_wizard(&self) {
        self.open.set(false);
        self.options.set(OnboardingOptions::default());
    }
}
```

### 2. Create `crates/lx-desktop/src/components/onboarding/mod.rs`

```rust
pub mod helpers;
pub mod step_agent;
pub mod step_company;
pub mod step_launch;
pub mod step_task;
pub mod wizard;

pub use wizard::OnboardingWizard;
```

### 3. Create `crates/lx-desktop/src/components/onboarding/helpers.rs`

Port helper functions from `onboarding-goal.ts` and `onboarding-launch.ts`.

```rust
pub fn parse_goal_input(raw: &str) -> (String, Option<String>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }
    let mut lines = trimmed.lines();
    let title = lines.next().unwrap_or("").trim().to_string();
    let description: String = lines.collect::<Vec<_>>().join("\n").trim().to_string();
    if description.is_empty() {
        (title, None)
    } else {
        (title, Some(description))
    }
}

pub fn build_project_payload(goal_id: Option<&str>) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "name": "Onboarding",
        "status": "in_progress"
    });
    if let Some(gid) = goal_id {
        payload["goalIds"] = serde_json::json!([gid]);
    }
    payload
}

pub fn build_issue_payload(
    title: &str,
    description: &str,
    assignee_agent_id: &str,
    project_id: &str,
    goal_id: Option<&str>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "title": title.trim(),
        "assigneeAgentId": assignee_agent_id,
        "projectId": project_id,
        "status": "todo"
    });
    let desc = description.trim();
    if !desc.is_empty() {
        payload["description"] = serde_json::json!(desc);
    }
    if let Some(gid) = goal_id {
        payload["goalId"] = serde_json::json!(gid);
    }
    payload
}
```

### 4. Create `crates/lx-desktop/src/components/onboarding/wizard.rs`

Top-level wizard component managing step state and the modal overlay. Port from `OnboardingWizard.tsx` lines 79-649 (state + handlers) and 616-1349 (render).

```rust
use dioxus::prelude::*;
use crate::contexts::onboarding::OnboardingCtx;
use super::step_company::StepCompany;
use super::step_agent::StepAgent;
use super::step_task::StepTask;
use super::step_launch::StepLaunch;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardStep {
    Company,
    Agent,
    Task,
    Launch,
}

impl WizardStep {
    pub fn index(&self) -> u8 {
        match self {
            Self::Company => 1,
            Self::Agent => 2,
            Self::Task => 3,
            Self::Launch => 4,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Company => "Company",
            Self::Agent => "Agent",
            Self::Task => "Task",
            Self::Launch => "Launch",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            Self::Company => "apartment",
            Self::Agent => "smart_toy",
            Self::Task => "checklist",
            Self::Launch => "rocket_launch",
        }
    }

    pub const ALL: &[WizardStep] = &[
        Self::Company,
        Self::Agent,
        Self::Task,
        Self::Launch,
    ];
}

#[component]
pub fn OnboardingWizard() -> Element {
    let onboarding = use_context::<OnboardingCtx>();
    let mut step = use_signal(|| WizardStep::Company);
    let mut error = use_signal(|| Option::<String>::None);
    let mut loading = use_signal(|| false);

    let mut created_company_id = use_signal(|| Option::<String>::None);
    let mut created_agent_id = use_signal(|| Option::<String>::None);
    let mut created_project_id = use_signal(|| Option::<String>::None);
    let mut created_issue_ref = use_signal(|| Option::<String>::None);
    let mut created_goal_id = use_signal(|| Option::<String>::None);

    let mut company_name = use_signal(|| String::new());
    let mut company_goal = use_signal(|| String::new());
    let mut agent_name = use_signal(|| "CEO".to_string());
    let mut task_title = use_signal(|| "Create a hiring plan".to_string());
    let mut task_description = use_signal(|| String::new());

    let is_open = *onboarding.open.read();

    use_effect(move || {
        if !is_open {
            step.set(WizardStep::Company);
            error.set(None);
            loading.set(false);
            created_company_id.set(None);
            created_agent_id.set(None);
            created_project_id.set(None);
            created_issue_ref.set(None);
            created_goal_id.set(None);
            company_name.set(String::new());
            company_goal.set(String::new());
            agent_name.set("CEO".to_string());
            task_title.set("Create a hiring plan".to_string());
            task_description.set(String::new());
        }
    });

    if !is_open {
        return None;
    }

    rsx! {
        div { class: "fixed inset-0 z-50 bg-black/60",
            onclick: move |_| onboarding.close_wizard(),
            div {
                class: "fixed top-[10%] left-1/2 -translate-x-1/2 w-full max-w-lg bg-[var(--surface-container)] border border-[var(--outline)] shadow-2xl z-50",
                onclick: move |e| e.stop_propagation(),

                div { class: "flex items-center justify-between px-6 pt-5 pb-3",
                    h2 { class: "text-sm font-bold uppercase tracking-wider text-[var(--on-surface)]",
                        "SETUP WIZARD"
                    }
                    button {
                        class: "text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
                        onclick: move |_| onboarding.close_wizard(),
                        span { class: "material-symbols-outlined text-lg", "close" }
                    }
                }

                // Step tabs
                div { class: "flex items-center gap-0 border-b border-[var(--outline-variant)] px-6",
                    for s in WizardStep::ALL.iter() {
                        button {
                            key: "{s.index()}",
                            class: if *step.read() == *s {
                                "flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 border-[var(--on-surface)] text-[var(--on-surface)] -mb-px"
                            } else {
                                "flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 border-transparent text-[var(--outline)] -mb-px hover:text-[var(--on-surface-variant)]"
                            },
                            onclick: move |_| step.set(*s),
                            span { class: "material-symbols-outlined text-sm", "{s.icon()}" }
                            "{s.label()}"
                        }
                    }
                }

                div { class: "px-6 py-5",
                    match *step.read() {
                        WizardStep::Company => rsx! {
                            StepCompany {
                                company_name,
                                company_goal,
                            }
                        },
                        WizardStep::Agent => rsx! {
                            StepAgent {
                                agent_name,
                            }
                        },
                        WizardStep::Task => rsx! {
                            StepTask {
                                task_title,
                                task_description,
                            }
                        },
                        WizardStep::Launch => rsx! {
                            StepLaunch {
                                company_name: company_name.read().clone(),
                                agent_name: agent_name.read().clone(),
                                task_title: task_title.read().clone(),
                            }
                        },
                    }

                    if let Some(err) = error.read().as_ref() {
                        div { class: "mt-3",
                            p { class: "text-xs text-[var(--error)]", "{err}" }
                        }
                    }

                    // Footer navigation
                    div { class: "flex items-center justify-between mt-6",
                        div {
                            if step.read().index() > 1 {
                                button {
                                    class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors flex items-center gap-1",
                                    disabled: *loading.read(),
                                    onclick: move |_| {
                                        let prev = match *step.read() {
                                            WizardStep::Agent => WizardStep::Company,
                                            WizardStep::Task => WizardStep::Agent,
                                            WizardStep::Launch => WizardStep::Task,
                                            WizardStep::Company => WizardStep::Company,
                                        };
                                        step.set(prev);
                                    },
                                    span { class: "material-symbols-outlined text-sm", "arrow_back" }
                                    "Back"
                                }
                            }
                        }
                        button {
                            class: "px-4 py-1.5 text-xs font-medium bg-[var(--primary)] text-[var(--on-primary)] hover:opacity-90 transition-opacity",
                            disabled: *loading.read(),
                            onclick: {
                                let step_val = *step.read();
                                move |_| {
                                    match step_val {
                                        WizardStep::Company => step.set(WizardStep::Agent),
                                        WizardStep::Agent => step.set(WizardStep::Task),
                                        WizardStep::Task => step.set(WizardStep::Launch),
                                        WizardStep::Launch => {
                                            onboarding.close_wizard();
                                        }
                                    }
                                }
                            },
                            if *loading.read() {
                                "Working..."
                            } else if *step.read() == WizardStep::Launch {
                                "Create & Launch"
                            } else {
                                "Next"
                            }
                        }
                    }
                }
            }
        }
    }
}
```

The `onclick` for the "Next" / "Create & Launch" button on Step 4 (Launch) will eventually call async functions via `use_api_client` (from Unit 17) to create the company, agent, project, and issue. For now, it closes the wizard. The API integration is wired inline: each step's Next handler spawns an async task that calls the API, sets `created_*_id` signals on success, sets `error` on failure, and advances the step.

Specifically for each step transition when API calls are wired:
- Step 1 -> 2: call `companies::create`, then optionally `goals::create` if goal text is non-empty. Store `created_company_id` and `created_goal_id`.
- Step 2 -> 3: call `agents::create` with `created_company_id`. Store `created_agent_id`.
- Step 3 -> 4: no API call, just advance.
- Step 4 launch: call `projects::create`, then `issues::create` with all stored IDs. Navigate to the new issue route.

For the initial implementation, the Next button on each step simply advances the step signal without API calls. The Launch button closes the wizard. Wire the actual API calls using `spawn(async move { ... })` inside the onclick closures, referencing the `ApiClient` from context.

### 5. Create `crates/lx-desktop/src/components/onboarding/step_company.rs`

Step 1: company name and optional goal.

```rust
use dioxus::prelude::*;

#[component]
pub fn StepCompany(
    company_name: Signal<String>,
    company_goal: Signal<String>,
) -> Element {
    rsx! {
        div { class: "space-y-5",
            div { class: "flex items-center gap-3 mb-1",
                div { class: "bg-[var(--surface-container-highest)] p-2",
                    span { class: "material-symbols-outlined text-xl text-[var(--outline)]", "apartment" }
                }
                div {
                    h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Name your company" }
                    p { class: "text-xs text-[var(--outline)]",
                        "This is the organization your agents will work for."
                    }
                }
            }
            div { class: "space-y-1",
                label { class: "text-xs text-[var(--outline)] block", "Company name" }
                input {
                    class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]",
                    placeholder: "Acme Corp",
                    value: "{company_name}",
                    oninput: move |e| company_name.set(e.value()),
                    autofocus: true,
                }
            }
            div { class: "space-y-1",
                label { class: "text-xs text-[var(--outline)] block", "Mission / goal (optional)" }
                textarea {
                    class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)] resize-none min-h-[60px]",
                    placeholder: "What is this company trying to achieve?",
                    value: "{company_goal}",
                    oninput: move |e| company_goal.set(e.value()),
                }
            }
        }
    }
}
```

### 6. Create `crates/lx-desktop/src/components/onboarding/step_agent.rs`

Step 2: agent name. Simplified from Paperclip (no adapter type selection, no model picker, no environment test).

```rust
use dioxus::prelude::*;

#[component]
pub fn StepAgent(
    agent_name: Signal<String>,
) -> Element {
    rsx! {
        div { class: "space-y-5",
            div { class: "flex items-center gap-3 mb-1",
                div { class: "bg-[var(--surface-container-highest)] p-2",
                    span { class: "material-symbols-outlined text-xl text-[var(--outline)]", "smart_toy" }
                }
                div {
                    h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Create your first agent" }
                    p { class: "text-xs text-[var(--outline)]",
                        "Name the agent that will handle your first task."
                    }
                }
            }
            div { class: "space-y-1",
                label { class: "text-xs text-[var(--outline)] block", "Agent name" }
                input {
                    class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]",
                    placeholder: "CEO",
                    value: "{agent_name}",
                    oninput: move |e| agent_name.set(e.value()),
                    autofocus: true,
                }
            }
        }
    }
}
```

### 7. Create `crates/lx-desktop/src/components/onboarding/step_task.rs`

Step 3: task title and description.

```rust
use dioxus::prelude::*;

#[component]
pub fn StepTask(
    task_title: Signal<String>,
    task_description: Signal<String>,
) -> Element {
    rsx! {
        div { class: "space-y-5",
            div { class: "flex items-center gap-3 mb-1",
                div { class: "bg-[var(--surface-container-highest)] p-2",
                    span { class: "material-symbols-outlined text-xl text-[var(--outline)]", "checklist" }
                }
                div {
                    h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Give it something to do" }
                    p { class: "text-xs text-[var(--outline)]",
                        "Give your agent a small task to start with."
                    }
                }
            }
            div { class: "space-y-1",
                label { class: "text-xs text-[var(--outline)] block", "Task title" }
                input {
                    class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)]",
                    placeholder: "e.g. Research competitor pricing",
                    value: "{task_title}",
                    oninput: move |e| task_title.set(e.value()),
                    autofocus: true,
                }
            }
            div { class: "space-y-1",
                label { class: "text-xs text-[var(--outline)] block", "Description (optional)" }
                textarea {
                    class: "w-full border border-[var(--outline-variant)] bg-transparent px-3 py-2 text-sm text-[var(--on-surface)] outline-none focus:border-[var(--primary)] placeholder:text-[var(--outline)] resize-none min-h-[120px]",
                    placeholder: "Add more detail about what the agent should do...",
                    value: "{task_description}",
                    oninput: move |e| task_description.set(e.value()),
                }
            }
        }
    }
}
```

### 8. Create `crates/lx-desktop/src/components/onboarding/step_launch.rs`

Step 4: review summary of company, agent, and task before launching.

```rust
use dioxus::prelude::*;

#[component]
pub fn StepLaunch(
    company_name: String,
    agent_name: String,
    task_title: String,
) -> Element {
    rsx! {
        div { class: "space-y-5",
            div { class: "flex items-center gap-3 mb-1",
                div { class: "bg-[var(--surface-container-highest)] p-2",
                    span { class: "material-symbols-outlined text-xl text-[var(--outline)]", "rocket_launch" }
                }
                div {
                    h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Ready to launch" }
                    p { class: "text-xs text-[var(--outline)]",
                        "Everything is set up. Launch will create the task and open it."
                    }
                }
            }
            div { class: "border border-[var(--outline-variant)] divide-y divide-[var(--outline-variant)]",
                SummaryRow { icon: "apartment", label: "Company", value: company_name }
                SummaryRow { icon: "smart_toy", label: "Agent", value: agent_name }
                SummaryRow { icon: "checklist", label: "Task", value: task_title }
            }
        }
    }
}

#[component]
fn SummaryRow(icon: &'static str, label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "flex items-center gap-3 px-3 py-2.5",
            span { class: "material-symbols-outlined text-base text-[var(--outline)] shrink-0", "{icon}" }
            div { class: "flex-1 min-w-0",
                p { class: "text-sm font-medium text-[var(--on-surface)] truncate", "{value}" }
                p { class: "text-xs text-[var(--outline)]", "{label}" }
            }
            span { class: "material-symbols-outlined text-base text-green-500 shrink-0", "check_circle" }
        }
    }
}
```

### 9. Edit `crates/lx-desktop/src/components/mod.rs` (already exists)

Add `pub mod onboarding;` to the existing `components/mod.rs`. Do NOT recreate the file or remove existing module declarations.

### 10. Modify `crates/lx-desktop/src/contexts/mod.rs`

Edit `contexts/mod.rs` -- add `pub mod onboarding;` to the existing module declarations. Do NOT replace the file contents (it already has modules from Units 3 and 17).

### 11. Modify `crates/lx-desktop/src/layout/shell.rs`

Add onboarding context provision and wizard rendering.

**Add import** (after existing imports):

```rust
use crate::contexts::onboarding::OnboardingCtx;
use crate::components::onboarding::OnboardingWizard;
```

**Inside the `Shell` component**, add the onboarding context provider after the existing `use_context_provider` calls (after `use_context_provider(|| status_bar_state);`):

```rust
let _onboarding = OnboardingCtx::provide();
```

**Add `OnboardingWizard {}` as the last child** of the root div, after `CommandPalette {}` (added in Unit 16):

```rust
CommandPalette {}
OnboardingWizard {}
```

### 12. Replace the onboarding stub

Unit 3 created a stub `pages/onboarding.rs`. This unit replaces it with a real `Onboarding` component. The `pub mod onboarding;` declaration already exists in `pages/mod.rs` from Unit 3, and `routes.rs` already imports `Onboarding` from `crate::pages::onboarding` -- no changes to either file are needed.

Replace the contents of `crates/lx-desktop/src/pages/onboarding.rs` with the real Onboarding component:

```rust
use dioxus::prelude::*;
use crate::contexts::onboarding::OnboardingCtx;

#[component]
pub fn Onboarding() -> Element {
    let onboarding = use_context::<OnboardingCtx>();
    use_effect(move || {
        onboarding.open_wizard(crate::contexts::onboarding::OnboardingOptions::default());
    });
    rsx! {
        crate::pages::agents::Agents {}
    }
}
```

### 13. Final Integration Pass

Concrete steps to verify everything composes correctly:

1. **Verify routes.rs compiles with all page imports.** Check that every Route variant's component is importable. Run `just diagnose` and fix any missing imports.
2. **Verify Shell renders sidebar with all nav items.** Confirm `sidebar.rs` has NavItem entries for Dashboard, Agents, Activity, Projects, Goals, Routines, Org, Costs, Approvals, Inbox, Tools, Settings, Accounts.
3. **Verify each page component exists and is importable.** For each Route variant, confirm the component function exists at the import path.
4. **Delete remaining stub files if all stubs are replaced.** Check each stub module file created by Unit 3 (e.g., `pages/not_found.rs`, `pages/agent_detail.rs`). If a stub file has been replaced by a real module (either as a rewritten single file or a directory module), verify it no longer contains stub content. Stub files that have NOT been replaced by any unit (e.g., `not_found.rs`, `agent_detail.rs`) should remain as-is.
5. **Verify z-index layering.** In `command_palette.rs`: change `z-50` to `z-[60]` so it renders above the `OnboardingWizard`'s `z-50`.
6. **Verify context provider order in shell.rs.** `OnboardingCtx::provide()` must be called before `OnboardingWizard {}` is rendered.
7. **Verify Onboarding route access.** The `Onboarding` component accesses `OnboardingCtx` via `use_context` -- this works because Shell provides it via `use_context_provider` and `Onboarding` is rendered inside the Shell layout.

## Line Count Verification

| File | Estimated lines |
|---|---|
| `contexts/onboarding.rs` | ~45 |
| `components/onboarding/mod.rs` | 9 |
| `components/onboarding/helpers.rs` | ~55 |
| `components/onboarding/wizard.rs` | ~180 |
| `components/onboarding/step_company.rs` | ~45 |
| `components/onboarding/step_agent.rs` | ~35 |
| `components/onboarding/step_task.rs` | ~45 |
| `components/onboarding/step_launch.rs` | ~45 |
| `components/mod.rs` (modified) | 3 |
| `contexts/mod.rs` (modified) | 5 |
| `layout/shell.rs` (modified) | ~210 |

All under 300 lines.

## Definition of Done

1. `just diagnose` passes with zero warnings
2. Navigating to `/onboarding` opens the wizard overlay
3. Step 1 renders company name and goal inputs; typing updates state
4. Step 2 renders agent name input
5. Step 3 renders task title and description inputs
6. Step 4 renders a summary table with company name, agent name, and task title
7. The Next button advances through steps 1 -> 2 -> 3 -> 4
8. The Back button goes backward through steps 4 -> 3 -> 2 -> 1
9. The close button and backdrop click dismiss the wizard
10. The wizard resets all state when closed and reopened
11. `CommandPalette` (Cmd+K) still works when the wizard is closed
12. The step tab buttons allow jumping directly to any step
13. All new files are under 300 lines
14. No code comments or doc strings in new files
15. `helpers.rs` functions `parse_goal_input`, `build_project_payload`, `build_issue_payload` return correct JSON structures
