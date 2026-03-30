use dioxus::prelude::*;

use super::step_agent::StepAgent;
use super::step_company::StepCompany;
use super::step_launch::StepLaunch;
use super::step_task::StepTask;
use crate::contexts::onboarding::OnboardingCtx;

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

  pub const ALL: &[WizardStep] = &[Self::Company, Self::Agent, Self::Task, Self::Launch];
}

#[component]
pub fn OnboardingWizard() -> Element {
  let onboarding = use_context::<OnboardingCtx>();
  let mut step = use_signal(|| WizardStep::Company);
  let mut error = use_signal(|| Option::<String>::None);
  let mut loading = use_signal(|| false);
  let mut loading_text = use_signal(|| Option::<String>::None);

  let mut company_name = use_signal(String::new);
  let mut company_goal = use_signal(String::new);
  let mut agent_name = use_signal(|| "CEO".to_string());
  let mut agent_role = use_signal(|| "ceo".to_string());
  let mut agent_description = use_signal(String::new);
  let mut agent_adapter = use_signal(|| "claude_local".to_string());
  let mut agent_model_id = use_signal(|| "claude-sonnet-4-20250514".to_string());
  let mut task_title = use_signal(|| "Create a hiring plan".to_string());
  let mut task_description = use_signal(String::new);

  let is_open = *onboarding.open.read();

  use_effect(move || {
    if !is_open {
      step.set(WizardStep::Company);
      error.set(None);
      loading.set(false);
      loading_text.set(None);
      company_name.set(String::new());
      company_goal.set(String::new());
      agent_name.set("CEO".to_string());
      agent_role.set("ceo".to_string());
      agent_description.set(String::new());
      agent_adapter.set("claude_local".to_string());
      agent_model_id.set("claude-sonnet-4-20250514".to_string());
      task_title.set("Create a hiring plan".to_string());
      task_description.set(String::new());
    }
  });

  if !is_open {
    return rsx! {};
  }

  rsx! {
    div {
      class: "fixed inset-0 z-50 bg-black/60",
      onclick: move |_| onboarding.close_wizard(),
      div {
        class: "fixed top-[10%] left-1/2 -translate-x-1/2 w-full max-w-lg bg-[var(--surface-container)] border border-[var(--outline)] shadow-2xl z-50",
        onclick: move |e| e.stop_propagation(),
        onkeydown: move |evt: KeyboardEvent| {
            if evt.modifiers().meta() && evt.key() == Key::Enter {
                let current = *step.read();
                match current {
                    WizardStep::Company => step.set(WizardStep::Agent),
                    WizardStep::Agent => step.set(WizardStep::Task),
                    WizardStep::Task => step.set(WizardStep::Launch),
                    WizardStep::Launch => {
                        onboarding.close_wizard();
                    }
                }
            }
        },
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
        StepTabs { current: step, on_select: move |s| step.set(s) }
        div { class: "px-6 py-5",
          match *step.read() {
              WizardStep::Company => rsx! {
                StepCompany { company_name, company_goal }
              },
              WizardStep::Agent => rsx! {
                StepAgent { agent_name, agent_role, agent_description, agent_adapter, agent_model_id }
              },
              WizardStep::Task => rsx! {
                StepTask { task_title, task_description }
              },
              WizardStep::Launch => rsx! {
                StepLaunch {
                  company_name: company_name.read().clone(),
                  agent_name: agent_name.read().clone(),
                  agent_role: agent_role.read().clone(),
                  agent_adapter: agent_adapter.read().clone(),
                  agent_model_id: agent_model_id.read().clone(),
                  task_title: task_title.read().clone(),
                }
              },
          }
          if let Some(err) = error.read().as_ref() {
            div { class: "mt-3",
              p { class: "text-xs text-[var(--error)]", "{err}" }
            }
          }
          WizardFooter { step, loading, loading_text, onboarding }
        }
      }
    }
  }
}

#[component]
fn StepTabs(current: Signal<WizardStep>, on_select: EventHandler<WizardStep>) -> Element {
  rsx! {
    div { class: "flex items-center gap-0 border-b border-[var(--outline-variant)] px-6",
      for s in WizardStep::ALL.iter() {
        button {
          key: "{s.index()}",
          class: if *current.read() == *s { "flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 border-[var(--on-surface)] text-[var(--on-surface)] -mb-px" } else { "flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 border-transparent text-[var(--outline)] -mb-px hover:text-[var(--on-surface-variant)]" },
          onclick: {
              let s = *s;
              move |_| on_select.call(s)
          },
          span { class: "material-symbols-outlined text-sm", "{s.icon()}" }
          "{s.label()}"
        }
      }
    }
  }
}

#[component]
fn WizardFooter(step: Signal<WizardStep>, loading: Signal<bool>, loading_text: Signal<Option<String>>, onboarding: OnboardingCtx) -> Element {
  let step_val = *step.read();
  rsx! {
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
        onclick: move |_| {
            match step_val {
                WizardStep::Company => step.set(WizardStep::Agent),
                WizardStep::Agent => step.set(WizardStep::Task),
                WizardStep::Task => step.set(WizardStep::Launch),
                WizardStep::Launch => {
                    onboarding.close_wizard();
                }
            }
        },
        if *loading.read() {
          div { class: "flex items-center gap-2",
              span { class: "material-symbols-outlined text-sm animate-spin", "progress_activity" }
              {
                  let text = loading_text.read().as_ref().cloned().unwrap_or_else(|| "Working...".into());
                  rsx! { span { "{text}" } }
              }
          }
        } else if *step.read() == WizardStep::Launch {
          "Create & Launch"
        } else {
          "Next"
        }
      }
    }
  }
}
