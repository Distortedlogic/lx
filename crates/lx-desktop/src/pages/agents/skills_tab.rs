use dioxus::prelude::*;

use super::run_types::{SkillEntry, SkillSnapshot};

#[component]
pub fn SkillsTab(snapshot: SkillSnapshot) -> Element {
  let mut desired = use_signal(|| snapshot.desired_skills.clone());
  let required: Vec<&SkillEntry> = snapshot.entries.iter().filter(|e| e.required).collect();
  let optional: Vec<&SkillEntry> = snapshot.entries.iter().filter(|e| !e.required).collect();

  rsx! {
    div { class: "max-w-3xl space-y-6",
      if !required.is_empty() {
        SkillSection {
          title: "Required Skills",
          description: "These skills are always enabled for this agent.",
          skills: required.iter().map(|e| (*e).clone()).collect(),
          desired_keys: desired.read().clone(),
          read_only: true,
          on_toggle: move |_: String| {},
        }
      }
      if !optional.is_empty() {
        SkillSection {
          title: "Optional Skills",
          description: "Toggle skills on or off for this agent.",
          skills: optional.iter().map(|e| (*e).clone()).collect(),
          desired_keys: desired.read().clone(),
          read_only: false,
          on_toggle: move |key: String| {
              let mut current = desired.read().clone();
              if current.contains(&key) {
                  current.retain(|k| k != &key);
              } else {
                  current.push(key);
              }
              desired.set(current);
          },
        }
      }
      if required.is_empty() && optional.is_empty() {
        p { class: "text-sm text-[var(--outline)]", "No skills configured." }
      }
    }
  }
}

#[component]
fn SkillSection(
  title: &'static str,
  description: &'static str,
  skills: Vec<SkillEntry>,
  desired_keys: Vec<String>,
  read_only: bool,
  on_toggle: EventHandler<String>,
) -> Element {
  rsx! {
    div { class: "space-y-3",
      div {
        h3 { class: "text-sm font-medium text-[var(--on-surface)]", "{title}" }
        p { class: "text-xs text-[var(--outline)] mt-1", "{description}" }
      }
      div { class: "border border-[var(--outline-variant)]/30 rounded-lg divide-y divide-[var(--outline-variant)]/15",
        for skill in skills.iter() {
          SkillRow {
            skill: skill.clone(),
            checked: skill.required || desired_keys.contains(&skill.key),
            read_only: read_only || skill.required,
            on_toggle: {
                let key = skill.key.clone();
                move |_| on_toggle.call(key.clone())
            },
          }
        }
      }
    }
  }
}

#[component]
fn SkillRow(skill: SkillEntry, checked: bool, read_only: bool, on_toggle: EventHandler<()>) -> Element {
  let opacity = if read_only { "opacity-60" } else { "" };
  rsx! {
    button {
      class: "flex items-start gap-3 w-full px-4 py-3 text-left hover:bg-[var(--surface-container)] transition-colors {opacity}",
      disabled: read_only,
      onclick: move |_| on_toggle.call(()),
      div {
        class: "flex items-center justify-center h-4 w-4 shrink-0 mt-0.5 border border-[var(--outline-variant)] rounded-sm",
        class: if checked { "bg-[var(--primary)]" } else { "" },
        if checked {
          span { class: "text-[10px] text-[var(--on-primary)] leading-none",
            "\u{2713}"
          }
        }
      }
      div { class: "flex-1 min-w-0",
        span { class: "text-sm font-medium text-[var(--on-surface)]", "{skill.name}" }
        if let Some(desc) = &skill.description {
          p { class: "text-xs text-[var(--outline)] mt-0.5", "{desc}" }
        }
        if let Some(detail) = &skill.detail {
          p { class: "text-xs text-[var(--outline)] font-mono mt-0.5", "{detail}" }
        }
      }
    }
  }
}
