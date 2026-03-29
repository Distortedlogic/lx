use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct CompanySwitcherEntry {
  pub id: String,
  pub name: String,
  pub status: String,
}

fn status_dot_color(status: &str) -> &'static str {
  match status {
    "active" => "bg-green-400",
    "paused" => "bg-yellow-400",
    "archived" => "bg-neutral-400",
    _ => "bg-green-400",
  }
}

#[component]
pub fn CompanySwitcher(companies: Vec<CompanySwitcherEntry>, selected_id: Option<String>, on_select: EventHandler<String>) -> Element {
  let mut open = use_signal(|| false);
  let selected = companies.iter().find(|c| Some(&c.id) == selected_id.as_ref());
  let sidebar_companies: Vec<_> = companies.iter().filter(|c| c.status != "archived").collect();

  rsx! {
    div { class: "relative",
      button {
        class: "w-full flex items-center justify-between px-2 py-1.5 text-left hover:bg-[var(--surface-container)] rounded",
        onclick: move |_| {
            let current = open();
            open.set(!current);
        },
        div { class: "flex items-center gap-2 min-w-0",
          if let Some(company) = selected {
            span { class: "h-2 w-2 rounded-full shrink-0 {status_dot_color(&company.status)}" }
          }
          span { class: "text-sm font-medium truncate text-[var(--on-surface)]",
            if let Some(company) = selected {
              "{company.name}"
            } else {
              "Select company"
            }
          }
        }
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
          "unfold_more"
        }
      }
      if open() {
        div { class: "absolute left-0 top-full mt-1 w-[220px] rounded-md border border-[var(--outline-variant)] bg-[var(--surface-container-lowest)] shadow-lg z-50",
          div { class: "px-3 py-1.5 text-xs font-semibold text-[var(--outline)] uppercase",
            "Companies"
          }
          div { class: "border-t border-[var(--outline-variant)]" }
          for company in sidebar_companies.iter() {
            {
                let id = company.id.clone();
                let is_selected = Some(&company.id) == selected_id.as_ref();
                let bg = if is_selected { " bg-[var(--surface-container)]" } else { "" };
                rsx! {
                  button {
                    class: "w-full flex items-center gap-2 px-3 py-1.5 text-sm text-left hover:bg-[var(--surface-container)]{bg}",
                    onclick: move |_| {
                        on_select.call(id.clone());
                        open.set(false);
                    },
                    span { class: "h-2 w-2 rounded-full shrink-0 {status_dot_color(&company.status)}" }
                    span { class: "truncate text-[var(--on-surface)]", "{company.name}" }
                  }
                }
            }
          }
          if sidebar_companies.is_empty() {
            div { class: "px-3 py-1.5 text-sm text-[var(--outline)]", "No companies" }
          }
          div { class: "border-t border-[var(--outline-variant)]" }
          button {
            class: "w-full flex items-center gap-2 px-3 py-1.5 text-sm text-left hover:bg-[var(--surface-container)]",
            onclick: move |_| open.set(false),
            span { class: "material-symbols-outlined text-base", "settings" }
            span { class: "text-[var(--on-surface)]", "Company Settings" }
          }
          button {
            class: "w-full flex items-center gap-2 px-3 py-1.5 text-sm text-left hover:bg-[var(--surface-container)]",
            onclick: move |_| open.set(false),
            span { class: "material-symbols-outlined text-base", "add" }
            span { class: "text-[var(--on-surface)]", "Manage Companies" }
          }
        }
      }
    }
  }
}
