use dioxus::prelude::*;

use super::types::{AgentRef, PRIORITY_ORDER, STATUS_ORDER};
use crate::components::markdown_editor::MarkdownEditor;
use crate::components::ui::select::{Select, SelectOption};
use crate::styles::{BTN_OUTLINE_SM, BTN_PRIMARY_SM};

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct IssueDraft {
  title: String,
  description: String,
  status: String,
  priority: String,
  assignee: Option<String>,
}

#[derive(Clone, Debug)]
pub struct NewIssuePayload;

#[component]
pub fn NewIssueDialog(open: bool, agents: Vec<AgentRef>, on_close: EventHandler<()>, on_create: EventHandler<NewIssuePayload>) -> Element {
  let mut title = use_signal(String::new);
  let mut description = use_signal(String::new);
  let mut status = use_signal(|| "todo".to_string());
  let mut priority = use_signal(|| "medium".to_string());
  let mut assignee = use_signal(|| Option::<String>::None);

  use_effect(move || {
    if open {
      spawn(async move {
        let result = document::eval(
          r#"
                  let d = localStorage.getItem("lx-new-issue-draft");
                  return d || "";
                  "#,
        )
        .await;
        if let Ok(val) = result {
          if let Some(s) = val.as_str() {
            if let Ok(draft) = serde_json::from_str::<IssueDraft>(s) {
              title.set(draft.title);
              description.set(draft.description);
              status.set(draft.status);
              priority.set(draft.priority);
              assignee.set(draft.assignee);
            }
          }
        }
      });
    }
  });

  use_effect(move || {
    let draft = IssueDraft {
      title: title.read().clone(),
      description: description.read().clone(),
      status: status.read().clone(),
      priority: priority.read().clone(),
      assignee: assignee.read().clone(),
    };
    if let Ok(json) = serde_json::to_string(&draft) {
      let js = format!(r#"localStorage.setItem("lx-new-issue-draft", {})"#, serde_json::json!(json));
      let js = js.clone();
      spawn(async move {
        let _ = document::eval(&js).await;
      });
    }
  });

  if !open {
    return rsx! {};
  }

  rsx! {
    div {
      class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50",
      onclick: move |_| on_close.call(()),
      onkeydown: move |evt: KeyboardEvent| {
          if evt.modifiers().meta() && evt.key() == Key::Enter && !title.read().trim().is_empty() {
              spawn(async move { let _ = document::eval(r#"localStorage.removeItem("lx-new-issue-draft")"#).await; });
              on_create.call(NewIssuePayload);
          }
      },
      div {
        class: "bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg w-full max-w-lg overflow-hidden",
        onclick: move |evt| evt.stop_propagation(),
        div { class: "flex items-center justify-between px-4 py-2.5 border-b border-[var(--outline-variant)]",
          span { class: "text-sm text-[var(--outline)]", "New Issue" }
          button {
            class: "text-[var(--outline)] hover:text-[var(--on-surface)]",
            onclick: move |_| on_close.call(()),
            span { class: "material-symbols-outlined text-lg", "close" }
          }
        }
        div { class: "p-4 space-y-4",
          input {
            class: "w-full text-lg font-semibold bg-transparent outline-none text-[var(--on-surface)] placeholder:text-[var(--outline)]/40",
            placeholder: "Issue title",
            value: "{title}",
            oninput: move |evt| title.set(evt.value().to_string()),
          }
          MarkdownEditor {
              value: description.read().clone(),
              on_change: move |val: String| description.set(val),
              placeholder: "Description (optional)".to_string(),
              class: "min-h-[120px]".to_string(),
          }
          div { class: "grid grid-cols-3 gap-3",
            div {
              label { class: "text-xs text-[var(--outline)] block mb-1",
                "Status"
              }
              Select {
                value: status.read().clone(),
                options: STATUS_ORDER.iter().map(|s| SelectOption::new(*s, *s)).collect::<Vec<_>>(),
                onchange: move |val: String| status.set(val),
              }
            }
            div {
              label { class: "text-xs text-[var(--outline)] block mb-1",
                "Priority"
              }
              Select {
                value: priority.read().clone(),
                options: PRIORITY_ORDER.iter().map(|p| SelectOption::new(*p, *p)).collect::<Vec<_>>(),
                onchange: move |val: String| priority.set(val),
              }
            }
            div {
              label { class: "text-xs text-[var(--outline)] block mb-1",
                "Assignee"
              }
              Select {
                value: assignee.read().as_deref().unwrap_or("").to_string(),
                options: {
                    let mut opts = vec![SelectOption::new("", "Unassigned")];
                    opts.extend(agents.iter().map(|a| SelectOption::new(&a.id, &a.name)));
                    opts
                },
                onchange: move |val: String| {
                    assignee.set(if val.is_empty() { None } else { Some(val) });
                },
              }
            }
          }
        }
        div { class: "border-t border-[var(--outline-variant)] px-4 py-3 flex justify-end gap-2",
          button {
            class: BTN_OUTLINE_SM,
            onclick: move |_| on_close.call(()),
            "Cancel"
          }
          button {
            class: BTN_PRIMARY_SM,
            disabled: title.read().trim().is_empty(),
            onclick: {
                move |_| {
                    spawn(async move { let _ = document::eval(r#"localStorage.removeItem("lx-new-issue-draft")"#).await; });
                    on_create.call(NewIssuePayload);
                }
            },
            "Create Issue"
          }
        }
      }
    }
  }
}
