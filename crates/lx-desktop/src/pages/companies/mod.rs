mod company_card;

use self::company_card::{CompanyCard, CompanyData};
use dioxus::prelude::*;

#[component]
pub fn Companies() -> Element {
  let mut selected_id: Signal<Option<String>> = use_signal(|| None);

  let companies: Vec<CompanyData> = vec![];

  rsx! {
    div { class: "space-y-6 p-4 overflow-auto",
      div { class: "flex items-center justify-end",
        button { class: "flex items-center gap-1.5 bg-[var(--primary)] text-[var(--on-primary)] rounded px-3 py-1.5 text-xs font-semibold",
          span { class: "material-symbols-outlined text-sm", "add" }
          "New Company"
        }
      }
      if companies.is_empty() {
        div { class: "flex flex-col items-center justify-center py-16 text-[var(--outline)]",
          span { class: "material-symbols-outlined text-4xl mb-4", "business" }
          p { class: "text-sm", "No companies yet." }
        }
      }
      div { class: "grid gap-4",
        for company in companies.iter() {
          CompanyCard {
            key: "{company.id}",
            company: company.clone(),
            selected: selected_id() == Some(company.id.clone()),
            on_select: move |id: String| selected_id.set(Some(id)),
            on_rename: move |(_id, _name): (String, String)| {},
            on_delete: move |_id: String| {},
          }
        }
      }
    }
  }
}
