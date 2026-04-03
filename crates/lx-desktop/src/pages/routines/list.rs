use dioxus::prelude::*;

use super::types::{CATCH_UP_POLICIES, CONCURRENCY_POLICIES, Routine};
use crate::routes::Route;
use crate::styles::{FLEX_BETWEEN, PAGE_HEADING};

#[component]
pub fn Routines() -> Element {
  let routines = dioxus_storage::use_persistent("lx_routines", Vec::<Routine>::new);
  let mut show_composer = use_signal(|| false);
  let draft_title = use_signal(String::new);
  let draft_description = use_signal(String::new);
  let draft_priority = use_signal(|| "medium".to_string());
  let draft_concurrency = use_signal(|| "coalesce_if_active".to_string());
  let draft_catch_up = use_signal(|| "skip_missed".to_string());
  let show_advanced = use_signal(|| false);

  let entries = routines();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: FLEX_BETWEEN,
        h1 { class: PAGE_HEADING, "ROUTINES" }
        button {
          class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
          onclick: move |_| show_composer.set(true),
          "CREATE ROUTINE"
        }
      }
      if entries.is_empty() {
        div { class: "flex-1 flex items-center justify-center",
          p { class: "text-sm text-[var(--outline)]", "No routines yet" }
        }
      } else {
        div { class: "overflow-x-auto",
          table { class: "min-w-full text-sm",
            thead {
              tr { class: "text-left text-xs text-[var(--outline)] border-b border-[var(--outline-variant)]/30 uppercase tracking-wider",
                th { class: "px-3 py-2 font-medium", "NAME" }
                th { class: "px-3 py-2 font-medium", "LAST RUN" }
                th { class: "px-3 py-2 font-medium", "ENABLED" }
              }
            }
            tbody {
              for routine in entries.iter() {
                {routine_row(routine, routines)}
              }
            }
          }
        }
      }
      if show_composer() {
        {
            create_dialog(
                routines,
                show_composer,
                &RoutineDraft {
                    title: draft_title,
                    description: draft_description,
                    priority: draft_priority,
                    concurrency: draft_concurrency,
                    catch_up: draft_catch_up,
                },
                show_advanced,
            )
        }
      }
    }
  }
}

fn routine_row(routine: &Routine, routines: Signal<Vec<Routine>>) -> Element {
  let id = routine.id.clone();
  let enabled = routine.status == "active";
  let is_paused_or_archived = routine.status == "paused" || routine.status == "archived";

  rsx! {
    tr {
      class: "border-b border-[var(--outline-variant)]/30 hover:bg-white/5 cursor-pointer transition-colors",
      onclick: {
          let id = id.clone();
          move |_| {
              let nav = navigator();
              nav.push(Route::RoutineDetail {
                  routine_id: id.clone(),
              });
          }
      },
      td { class: "px-3 py-2.5",
        div {
          span { class: "font-medium text-[var(--on-surface)]", "{routine.title}" }
          if is_paused_or_archived {
            div { class: "mt-1 text-xs text-[var(--outline)]", "{routine.status}" }
          }
        }
      }
      td { class: "px-3 py-2.5 text-[var(--outline)]",
        div { "{routine.last_run_at.as_deref().unwrap_or(\"Never\")}" }
        if let Some(status) = &routine.last_run_status {
          div { class: "mt-1 text-xs", "{status}" }
        }
      }
      td {
        class: "px-3 py-2.5",
        onclick: move |evt| evt.stop_propagation(),
        div { class: "flex items-center gap-3",
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
                    let mut r = routines;
                    if let Some(item) = r.write().iter_mut().find(|r| r.id == id) {
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
          span { class: "text-xs text-[var(--outline)]",
            if enabled {
              "On"
            } else {
              "Off"
            }
          }
        }
      }
    }
  }
}

struct RoutineDraft {
  title: Signal<String>,
  description: Signal<String>,
  priority: Signal<String>,
  concurrency: Signal<String>,
  catch_up: Signal<String>,
}

fn create_dialog(mut routines: Signal<Vec<Routine>>, mut show_composer: Signal<bool>, draft: &RoutineDraft, mut show_advanced: Signal<bool>) -> Element {
  let mut draft_title = draft.title;
  let mut draft_description = draft.description;
  let mut draft_priority = draft.priority;
  let mut draft_concurrency = draft.concurrency;
  let mut draft_catch_up = draft.catch_up;
  let select_cls = "bg-[var(--surface-container)] border border-[var(--outline-variant)] \
                      text-xs px-2 py-1.5 rounded outline-none text-[var(--on-surface)] w-full";

  rsx! {
    div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
      div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg w-[480px] max-h-[90vh] overflow-y-auto",
        div { class: "px-5 py-3 border-b border-[var(--outline-variant)]/30",
          p { class: "text-xs font-medium uppercase tracking-[0.2em] text-[var(--outline)]",
            "NEW ROUTINE"
          }
        }
        div { class: "px-5 py-4 flex flex-col gap-3",
          input {
            class: "w-full bg-transparent text-lg font-semibold outline-none placeholder-[var(--outline)]/50 text-[var(--on-surface)]",
            placeholder: "Routine title",
            value: "{draft_title}",
            oninput: move |evt| draft_title.set(evt.value()),
          }
          textarea {
            class: "w-full bg-[var(--surface-container-lowest)] text-xs px-3 py-2 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)] min-h-[80px] resize-y",
            placeholder: "Add instructions...",
            value: "{draft_description}",
            oninput: move |evt| draft_description.set(evt.value()),
          }
        }
        div { class: "px-5 py-3 border-t border-[var(--outline-variant)]/30",
          button {
            class: "flex w-full items-center justify-between text-left",
            onclick: move |_| show_advanced.set(!show_advanced()),
            span { class: "text-sm font-medium text-[var(--on-surface)]",
              "Advanced settings"
            }
            span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
              if show_advanced() {
                "expand_less"
              } else {
                "expand_more"
              }
            }
          }
          if show_advanced() {
            div { class: "mt-3 flex flex-col gap-4",
              div { class: "flex flex-col gap-1",
                p { class: "text-xs font-medium uppercase tracking-[0.18em] text-[var(--outline)]",
                  "CONCURRENCY"
                }
                select {
                  class: select_cls,
                  value: "{draft_concurrency}",
                  onchange: move |evt| draft_concurrency.set(evt.value()),
                  for (val , _desc) in CONCURRENCY_POLICIES {
                    option { value: *val, "{val}" }
                  }
                }
                {policy_description(CONCURRENCY_POLICIES, &draft_concurrency())}
              }
              div { class: "flex flex-col gap-1",
                p { class: "text-xs font-medium uppercase tracking-[0.18em] text-[var(--outline)]",
                  "CATCH-UP"
                }
                select {
                  class: select_cls,
                  value: "{draft_catch_up}",
                  onchange: move |evt| draft_catch_up.set(evt.value()),
                  for (val , _desc) in CATCH_UP_POLICIES {
                    option { value: *val, "{val}" }
                  }
                }
                {policy_description(CATCH_UP_POLICIES, &draft_catch_up())}
              }
            }
          }
        }
        div { class: "flex items-center justify-end gap-3 px-5 py-3 border-t border-[var(--outline-variant)]/30",
          button {
            class: "text-xs uppercase text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
            onclick: move |_| {
                show_composer.set(false);
                show_advanced.set(false);
            },
            "CANCEL"
          }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
            onclick: move |_| {
                let title = draft_title().trim().to_string();
                if title.is_empty() {
                    return;
                }
                let new_routine = Routine {
                    id: uuid::Uuid::new_v4().to_string(),
                    title,
                    description: {
                        let d = draft_description().trim().to_string();
                        if d.is_empty() { None } else { Some(d) }
                    },
                    status: "active".into(),
                    project_id: None,
                    assignee_agent_id: None,
                    priority: draft_priority().clone(),
                    concurrency_policy: draft_concurrency().clone(),
                    catch_up_policy: draft_catch_up().clone(),
                    cron_expression: None,
                    last_run_at: None,
                    last_run_status: None,
                };
                routines.write().push(new_routine);
                draft_title.set(String::new());
                draft_description.set(String::new());
                draft_priority.set("medium".into());
                draft_concurrency.set("coalesce_if_active".into());
                draft_catch_up.set("skip_missed".into());
                show_composer.set(false);
                show_advanced.set(false);
            },
            "CREATE ROUTINE"
          }
        }
      }
    }
  }
}

fn policy_description(policies: &[(&str, &str)], current: &str) -> Element {
  let desc = policies.iter().find(|(k, _)| *k == current).map(|(_, v)| *v).unwrap_or("");
  rsx! {
    p { class: "text-xs text-[var(--outline)]", "{desc}" }
  }
}
