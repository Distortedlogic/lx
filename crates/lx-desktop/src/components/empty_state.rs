use dioxus::prelude::*;

#[component]
pub fn EmptyState(icon: String, message: String, #[props(optional)] action: Option<String>, #[props(optional)] on_action: Option<EventHandler<()>>) -> Element {
  rsx! {
    div { class: "flex flex-col items-center justify-center py-16 text-center",
      div { class: "bg-[var(--surface-container)]/50 p-4 mb-4",
        span { class: "material-symbols-outlined text-4xl text-[var(--outline)]",
          "{icon}"
        }
      }
      p { class: "text-sm text-[var(--on-surface-variant)] mb-4", "{message}" }
      if let (Some(act), Some(handler)) = (&action, &on_action) {
        {
            let handler = *handler;
            rsx! {
              button {
                class: "px-4 py-2 bg-[var(--primary)] hover:brightness-110 text-[var(--on-primary)] text-sm rounded transition-colors",
                onclick: move |_| handler.call(()),
                "{act}"
              }
            }
        }
      }
    }
  }
}
