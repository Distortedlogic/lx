use dioxus::prelude::*;

#[component]
pub fn EntityRow(
  title: String,
  #[props(optional)] leading: Option<Element>,
  #[props(optional)] identifier: Option<String>,
  #[props(optional)] subtitle: Option<String>,
  #[props(optional)] trailing: Option<Element>,
  #[props(default = false)] selected: bool,
  #[props(optional)] to: Option<String>,
  #[props(optional)] onclick: Option<EventHandler<()>>,
  #[props(optional)] class: Option<String>,
) -> Element {
  let interactive = to.is_some() || onclick.is_some();
  let extra = class.as_deref().unwrap_or("");
  let base = "flex items-center gap-3 px-4 py-2 text-sm border-b border-gray-700/50 last:border-b-0 transition-colors";
  let hover = if interactive { " cursor-pointer hover:bg-white/5" } else { "" };
  let sel = if selected { " bg-white/[0.03]" } else { "" };
  let cls = format!("{base}{hover}{sel} {extra}");

  let inner = rsx! {
    if let Some(lead) = leading {
      div { class: "flex items-center gap-2 shrink-0", {lead} }
    }
    div { class: "flex-1 min-w-0",
      div { class: "flex items-center gap-2",
        if let Some(ref id) = identifier {
          span { class: "text-xs text-gray-400 font-mono shrink-0", "{id}" }
        }
        span { class: "truncate", "{title}" }
      }
      if let Some(ref sub) = subtitle {
        p { class: "text-xs text-gray-400 truncate mt-0.5", "{sub}" }
      }
    }
    if let Some(trail) = trailing {
      div { class: "flex items-center gap-2 shrink-0", {trail} }
    }
  };

  if let Some(ref href) = to {
    rsx! {
      Link { to: "{href}", class: "{cls}", {inner} }
    }
  } else if let Some(handler) = onclick {
    rsx! {
      div { class: "{cls}", onclick: move |_| handler.call(()), {inner} }
    }
  } else {
    rsx! {
      div { class: "{cls}", {inner} }
    }
  }
}
