use dioxus::prelude::*;

use super::types::IssueDocument;

#[component]
pub fn DocumentsSection(documents: Vec<IssueDocument>) -> Element {
  rsx! {
    div { class: "space-y-3",
      h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Documents" }
      div { class: "space-y-2",
        for doc in documents.iter() {
          DocumentCard { document: doc.clone() }
        }
      }
    }
  }
}

#[component]
fn DocumentCard(document: IssueDocument) -> Element {
  let mut expanded = use_signal(|| false);
  let title = document.title.as_deref().unwrap_or(&document.key);
  let has_body = !document.body.is_empty();

  rsx! {
    div { class: "border border-[var(--outline-variant)]/20 rounded-lg overflow-hidden",
      button {
        class: "flex items-center gap-2 w-full px-3 py-2.5 text-left hover:bg-[var(--surface-container)] transition-colors",
        onclick: move |_| {
            if has_body {
                let current = *expanded.read();
                expanded.set(!current);
            }
        },
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]",
          "description"
        }
        span { class: "flex-1 text-sm font-medium text-[var(--on-surface)]",
          "{title}"
        }
        if has_body {
          span { class: "material-symbols-outlined text-xs text-[var(--outline)]",
            if *expanded.read() {
              "expand_less"
            } else {
              "expand_more"
            }
          }
        }
        if let Some(updated) = &document.updated_at {
          span { class: "text-xs text-[var(--outline)]", "{updated}" }
        }
      }
      if *expanded.read() && has_body {
        div { class: "px-3 py-3 border-t border-[var(--outline-variant)]/15 text-sm text-[var(--on-surface-variant)] whitespace-pre-wrap",
          "{document.body}"
        }
      }
    }
  }
}
