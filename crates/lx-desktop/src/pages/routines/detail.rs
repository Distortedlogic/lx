use dioxus::prelude::*;

use super::schedule_editor::ScheduleEditor;
use super::types::{CATCH_UP_POLICIES, CONCURRENCY_POLICIES, PRIORITIES, Routine};
use crate::components::page_skeleton::PageSkeleton;
#[component]
pub fn RoutineDetail(routine_id: String) -> Element {
  rsx! {
    SuspenseBoundary {
      fallback: |_| rsx! {
        PageSkeleton { variant: "detail".to_string() }
      },
      RoutineDetailInner { routine_id }
    }
  }
}

#[component]
fn RoutineDetailInner(routine_id: String) -> Element {
  let mut routines = dioxus_storage::use_persistent("lx_routines", Vec::<Routine>::new);
  let mut active_tab: Signal<&'static str> = use_signal(|| "triggers");
  let mut draft_cron = use_signal(String::new);

  let entries = routines();
  let routine = entries.iter().find(|r| r.id == routine_id);

  let Some(routine) = routine else {
    return rsx! {
      div { class: "flex-1 flex items-center justify-center p-4",
        p { class: "text-sm text-[var(--outline)]", "Routine not found" }
      }
    };
  };

  let routine = routine.clone();
  let id = routine.id.clone();
  let enabled = routine.status == "active";

  if draft_cron().is_empty()
    && let Some(cron) = &routine.cron_expression
  {
    draft_cron.set(cron.clone());
  }

  let select_cls = "bg-[var(--surface-container)] border border-[var(--outline-variant)] \
                      text-xs px-2 py-1.5 rounded outline-none text-[var(--on-surface)] w-full";
  let tab_base = "px-4 py-2 text-xs uppercase font-semibold tracking-wider transition-colors";

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex items-center justify-between",
        h2 { class: "page-heading", "{routine.title}" }
        div { class: "flex items-center gap-3",
          span { class: "text-xs uppercase tracking-wider px-2 py-1 rounded bg-[var(--surface-container)] text-[var(--outline)]",
            "{routine.status}"
          }
          button {
            role: "switch",
            class: format!(
                "relative inline-flex h-6 w-11 items-center rounded-full transition-colors {}",
                if enabled { "bg-green-500" } else { "bg-[var(--outline-variant)]" },
            ),
            onclick: {
                let id = id.clone();
                move |_| {
                    let new_status = if enabled { "paused" } else { "active" };
                    if let Some(item) = routines.write().iter_mut().find(|r| r.id == id) {
                        item.status = new_status.into();
                    }
                }
            },
            span {
              class: format!(
                  "inline-block h-5 w-5 rounded-full bg-white shadow-sm transition-transform {}",
                  if enabled { "translate-x-5" } else { "translate-x-0.5" },
              ),
            }
          }
        }
      }
      input {
        class: "w-full bg-transparent text-lg font-semibold outline-none text-[var(--on-surface)]",
        value: "{routine.title}",
        onchange: {
            let id = id.clone();
            move |evt: Event<FormData>| {
                if let Some(item) = routines.write().iter_mut().find(|r| r.id == id) {
                    item.title = evt.value();
                }
            }
        },
      }
      textarea {
        class: "w-full bg-[var(--surface-container-lowest)] text-xs px-3 py-2 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)] min-h-[60px] resize-y",
        placeholder: "Add description...",
        value: "{routine.description.as_deref().unwrap_or(\"\")}",
        onchange: {
            let id = id.clone();
            move |evt: Event<FormData>| {
                let v = evt.value().trim().to_string();
                if let Some(item) = routines.write().iter_mut().find(|r| r.id == id) {
                    item.description = if v.is_empty() { None } else { Some(v) };
                }
            }
        },
      }
      div { class: "flex gap-1 border-b border-[var(--outline-variant)]/30",
        button {
          class: format!(
              "{tab_base} {}",
              if active_tab() == "triggers" {
                  "text-[var(--on-surface)] border-b-2 border-[var(--primary)]"
              } else {
                  "text-[var(--outline)]"
              },
          ),
          onclick: move |_| active_tab.set("triggers"),
          "TRIGGERS"
        }
        button {
          class: format!(
              "{tab_base} {}",
              if active_tab() == "settings" {
                  "text-[var(--on-surface)] border-b-2 border-[var(--primary)]"
              } else {
                  "text-[var(--outline)]"
              },
          ),
          onclick: move |_| active_tab.set("settings"),
          "SETTINGS"
        }
      }
      if active_tab() == "triggers" {
        {triggers_tab(&routine, &id, routines, draft_cron)}
      } else {
        {settings_tab(&routine, &id, routines, select_cls)}
      }
    }
  }
}

fn triggers_tab(routine: &Routine, id: &str, mut routines: Signal<Vec<Routine>>, mut draft_cron: Signal<String>) -> Element {
  let id_owned = id.to_string();
  let id_run = id.to_string();
  rsx! {
    div { class: "flex flex-col gap-4",
      if routine.cron_expression.is_some() {
        ScheduleEditor {
          value: draft_cron(),
          on_change: move |cron: String| {
              draft_cron.set(cron.clone());
              if let Some(item) = routines.write().iter_mut().find(|r| r.id == id_owned) {
                  item.cron_expression = Some(cron);
              }
          },
        }
      } else {
        div { class: "flex flex-col items-center gap-3 py-8",
          p { class: "text-sm text-[var(--outline)]", "No schedule configured" }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
            onclick: {
                let id = id_owned.clone();
                move |_| {
                    let default_cron = "0 10 * * *".to_string();
                    draft_cron.set(default_cron.clone());
                    if let Some(item) = routines.write().iter_mut().find(|r| r.id == id) {
                        item.cron_expression = Some(default_cron);
                    }
                }
            },
            "ADD SCHEDULE"
          }
        }
      }
      button {
        class: "self-start bg-[var(--surface-container)] border border-[var(--outline-variant)] text-xs uppercase font-semibold px-4 py-2 rounded text-[var(--on-surface)] hover:brightness-110 transition-all",
        onclick: move |_| {
            let now = chrono_now_iso();
            if let Some(item) = routines.write().iter_mut().find(|r| r.id == id_run) {
                item.last_run_at = Some(now);
                item.last_run_status = Some("completed".into());
            }
        },
        "RUN NOW"
      }
    }
  }
}

fn settings_tab(routine: &Routine, id: &str, mut routines: Signal<Vec<Routine>>, select_cls: &str) -> Element {
  let id_conc = id.to_string();
  let id_catch = id.to_string();
  let id_prio = id.to_string();
  let id_archive = id.to_string();
  let is_archived = routine.status == "archived";
  rsx! {
    div { class: "flex flex-col gap-4",
      div { class: "flex flex-col gap-1",
        p { class: "text-xs font-medium uppercase tracking-[0.18em] text-[var(--outline)]",
          "CONCURRENCY"
        }
        select {
          class: select_cls,
          value: "{routine.concurrency_policy}",
          onchange: move |evt| {
              if let Some(item) = routines.write().iter_mut().find(|r| r.id == id_conc) {
                  item.concurrency_policy = evt.value();
              }
          },
          for (val, desc) in CONCURRENCY_POLICIES {
            option { value: *val, "{val} - {desc}" }
          }
        }
      }
      div { class: "flex flex-col gap-1",
        p { class: "text-xs font-medium uppercase tracking-[0.18em] text-[var(--outline)]",
          "CATCH-UP"
        }
        select {
          class: select_cls,
          value: "{routine.catch_up_policy}",
          onchange: move |evt| {
              if let Some(item) = routines.write().iter_mut().find(|r| r.id == id_catch) {
                  item.catch_up_policy = evt.value();
              }
          },
          for (val, desc) in CATCH_UP_POLICIES {
            option { value: *val, "{val} - {desc}" }
          }
        }
      }
      div { class: "flex flex-col gap-1",
        p { class: "text-xs font-medium uppercase tracking-[0.18em] text-[var(--outline)]",
          "PRIORITY"
        }
        select {
          class: select_cls,
          value: "{routine.priority}",
          onchange: move |evt| {
              if let Some(item) = routines.write().iter_mut().find(|r| r.id == id_prio) {
                  item.priority = evt.value();
              }
          },
          for p in PRIORITIES {
            option { value: *p, "{p}" }
          }
        }
      }
      button {
        class: "self-start text-xs uppercase font-semibold px-4 py-2 rounded border border-[var(--outline-variant)] text-[var(--error)] hover:bg-[var(--error)]/10 transition-all",
        onclick: move |_| {
            let new_status = if is_archived { "active" } else { "archived" };
            if let Some(item) = routines.write().iter_mut().find(|r| r.id == id_archive) {
                item.status = new_status.into();
            }
        },
        if is_archived {
          "RESTORE"
        } else {
          "ARCHIVE"
        }
      }
    }
  }
}

fn chrono_now_iso() -> String {
  let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
  let secs = now.as_secs();
  let hours = (secs % 86400) / 3600;
  let mins = (secs % 3600) / 60;
  format!("run at {hours:02}:{mins:02} UTC")
}
