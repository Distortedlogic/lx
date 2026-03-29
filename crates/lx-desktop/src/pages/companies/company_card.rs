use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CompanyData {
  pub id: String,
  pub name: String,
  pub description: Option<String>,
  pub status: String,
  pub agent_count: u32,
  pub issue_count: u32,
  pub spent_monthly_cents: u64,
  pub budget_monthly_cents: u64,
  pub created_at: String,
}

fn format_cents(cents: u64) -> String {
  format!("${:.2}", cents as f64 / 100.0)
}

#[component]
pub fn CompanyCard(
  company: CompanyData,
  selected: bool,
  on_select: EventHandler<String>,
  on_rename: EventHandler<(String, String)>,
  on_delete: EventHandler<String>,
) -> Element {
  let mut editing = use_signal(|| false);
  let mut edit_name = use_signal(|| company.name.clone());
  let mut confirming_delete = use_signal(|| false);

  let border = if selected { "border-[var(--primary)] ring-1 ring-[var(--primary)]" } else { "border-[var(--outline-variant)] hover:border-[var(--outline)]" };
  let status_class = match company.status.as_str() {
    "active" => "bg-green-500/10 text-green-600",
    "paused" => "bg-yellow-500/10 text-yellow-600",
    _ => "bg-[var(--surface-container)] text-[var(--outline)]",
  };
  let budget_pct = if company.budget_monthly_cents > 0 { (company.spent_monthly_cents as f64 / company.budget_monthly_cents as f64 * 100.0) as u32 } else { 0 };
  let id = company.id.clone();
  let id2 = company.id.clone();
  let id3 = company.id.clone();

  rsx! {
    div {
      class: "group text-left bg-[var(--surface-container-lowest)] border rounded-lg p-5 cursor-pointer {border}",
      onclick: move |_| on_select.call(id.clone()),
      div { class: "flex items-start justify-between gap-3",
        div { class: "flex-1 min-w-0",
          if editing() {
            div {
              class: "flex items-center gap-2",
              onclick: move |evt| evt.stop_propagation(),
              input {
                class: "h-7 text-sm border border-[var(--outline-variant)] rounded px-2 bg-transparent text-[var(--on-surface)]",
                value: "{edit_name}",
                oninput: move |evt| edit_name.set(evt.value()),
              }
              button {
                class: "text-green-500 text-sm",
                onclick: move |_| {
                    on_rename.call((id2.clone(), edit_name().trim().to_string()));
                    editing.set(false);
                },
                span { class: "material-symbols-outlined text-sm", "check" }
              }
              button {
                class: "text-[var(--outline)] text-sm",
                onclick: move |_| editing.set(false),
                span { class: "material-symbols-outlined text-sm", "close" }
              }
            }
          } else {
            div { class: "flex items-center gap-2",
              h3 { class: "font-semibold text-base text-[var(--on-surface)]",
                "{company.name}"
              }
              span { class: "inline-flex items-center rounded-full px-2 py-0.5 text-[11px] font-medium {status_class}",
                "{company.status}"
              }
            }
          }
          if let Some(ref desc) = company.description {
            if !editing() {
              p { class: "text-sm text-[var(--outline)] mt-1 line-clamp-2",
                "{desc}"
              }
            }
          }
        }
      }
      div { class: "flex items-center gap-3 mt-4 text-sm text-[var(--outline)] flex-wrap",
        div { class: "flex items-center gap-1.5",
          span { class: "material-symbols-outlined text-sm", "group" }
          span { "{company.agent_count} agents" }
        }
        div { class: "flex items-center gap-1.5",
          span { class: "material-symbols-outlined text-sm", "radio_button_checked" }
          span { "{company.issue_count} issues" }
        }
        div { class: "flex items-center gap-1.5 tabular-nums",
          span { class: "material-symbols-outlined text-sm", "attach_money" }
          span {
            "{format_cents(company.spent_monthly_cents)}"
            if company.budget_monthly_cents > 0 {
              " / {format_cents(company.budget_monthly_cents)} ({budget_pct}%)"
            } else {
              " Unlimited"
            }
          }
        }
        div { class: "flex items-center gap-1.5 ml-auto",
          span { class: "material-symbols-outlined text-sm", "calendar_today" }
          span { "Created {company.created_at}" }
        }
      }
      if confirming_delete() {
        div {
          class: "mt-4 flex items-center justify-between bg-red-500/5 border border-red-500/20 rounded-md px-4 py-3",
          onclick: move |evt| evt.stop_propagation(),
          p { class: "text-sm text-red-500 font-medium",
            "Delete this company? This cannot be undone."
          }
          div { class: "flex items-center gap-2 ml-4 shrink-0",
            button {
              class: "px-3 py-1 text-sm rounded hover:bg-[var(--surface-container)]",
              onclick: move |_| confirming_delete.set(false),
              "Cancel"
            }
            button {
              class: "bg-red-600 text-white px-3 py-1 text-sm rounded",
              onclick: move |_| {
                  on_delete.call(id3.clone());
                  confirming_delete.set(false);
              },
              "Delete"
            }
          }
        }
      }
    }
  }
}
