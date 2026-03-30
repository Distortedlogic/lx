use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct MentionCandidate {
  pub id: String,
  pub name: String,
}

#[component]
pub fn MentionPopup(
  candidates: Vec<MentionCandidate>,
  query: String,
  visible: bool,
  top: f64,
  left: f64,
  selected_index: usize,
  on_select: EventHandler<MentionCandidate>,
) -> Element {
  if !visible || candidates.is_empty() {
    return rsx! {};
  }

  let filtered: Vec<&MentionCandidate> = candidates
    .iter()
    .filter(|c| {
      query.is_empty() || c.name.to_lowercase().contains(&query.to_lowercase())
    })
    .collect();

  if filtered.is_empty() {
    return rsx! {};
  }

  rsx! {
    div {
      class: "fixed z-[100] bg-[var(--surface-container-high)] border border-[var(--outline-variant)] rounded-lg shadow-lg py-1 min-w-[180px] max-h-48 overflow-y-auto",
      style: "top: {top}px; left: {left}px;",
      for (i, candidate) in filtered.iter().enumerate() {
        {
          let c = (*candidate).clone();
          let is_selected = i == selected_index;
          let bg = if is_selected { "bg-[var(--surface-container-highest)]" } else { "" };
          rsx! {
            button {
              key: "{c.id}",
              class: "w-full text-left px-3 py-1.5 text-sm text-[var(--on-surface)] hover:bg-[var(--surface-container-highest)] flex items-center gap-2 {bg}",
              onmousedown: {
                let c = c.clone();
                move |evt: MouseEvent| {
                  evt.prevent_default();
                  on_select.call(c.clone());
                }
              },
              span { class: "w-5 h-5 rounded-full bg-[var(--primary)]/20 text-[var(--primary)] text-xs flex items-center justify-center font-semibold shrink-0",
                "{c.name.chars().next().unwrap_or('?')}"
              }
              span { "{c.name}" }
            }
          }
        }
      }
    }
  }
}
