use dioxus::prelude::*;

#[component]
pub fn MetricCard(
  icon: String,
  value: String,
  label: String,
  #[props(optional)] description: Option<String>,
  #[props(optional)] to: Option<String>,
  #[props(optional)] onclick: Option<EventHandler<()>>,
) -> Element {
  let clickable = to.is_some() || onclick.is_some();
  let hover_class = if clickable { "hover:bg-[var(--on-surface)]/5 cursor-pointer" } else { "" };

  let desc_text = description.as_deref().unwrap_or("");
  let has_desc = description.is_some();

  let inner = rsx! {
    div { class: "h-full px-5 py-5 rounded-lg transition-colors {hover_class}",
      div { class: "flex items-start justify-between gap-3",
        div { class: "flex-1 min-w-0",
          p { class: "text-3xl font-semibold tracking-tight tabular-nums",
            "{value}"
          }
          p { class: "text-sm font-medium text-[var(--on-surface-variant)] mt-1",
            "{label}"
          }
          if has_desc {
            div { class: "text-xs text-[var(--outline)] mt-1.5", "{desc_text}" }
          }
        }
        span { class: "material-symbols-outlined text-base text-[var(--outline)] shrink-0 mt-1.5",
          "{icon}"
        }
      }
    }
  };

  if let Some(ref href) = to {
    rsx! {
      Link { to: "{href}", {inner} }
    }
  } else if let Some(handler) = onclick {
    rsx! {
      div { onclick: move |_| handler.call(()), {inner} }
    }
  } else {
    inner
  }
}
