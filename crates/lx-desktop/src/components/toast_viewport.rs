use dioxus::prelude::*;

use crate::contexts::toast::{ToastItem, ToastState, ToastTone};

fn tone_class(tone: ToastTone) -> &'static str {
  match tone {
    ToastTone::Info => "border-sky-500/25 bg-sky-950/60 text-sky-100",
    ToastTone::Success => "border-emerald-500/25 bg-emerald-950/60 text-emerald-100",
    ToastTone::Warn => "border-amber-500/25 bg-amber-950/60 text-amber-100",
    ToastTone::Error => "border-red-500/30 bg-red-950/60 text-red-100",
  }
}

fn dot_class(tone: ToastTone) -> &'static str {
  match tone {
    ToastTone::Info => "bg-sky-400",
    ToastTone::Success => "bg-emerald-400",
    ToastTone::Warn => "bg-amber-400",
    ToastTone::Error => "bg-red-400",
  }
}

fn render_toast(toast: &ToastItem, state: ToastState) -> Element {
  let tc = tone_class(toast.tone);
  let dc = dot_class(toast.tone);
  let id = toast.id.clone();
  let title = toast.title.clone();
  let body = toast.body.clone();
  let action = toast.action.clone();

  rsx! {
    li { class: "pointer-events-auto rounded-sm border shadow-lg backdrop-blur-xl {tc}",
      div { class: "flex items-start gap-3 px-3 py-2.5",
        span { class: "mt-1 h-2 w-2 shrink-0 rounded-full {dc}" }
        div { class: "min-w-0 flex-1",
          p { class: "text-sm font-semibold leading-5", "{title}" }
          if let Some(ref b) = body {
            p { class: "mt-1 text-xs leading-4 opacity-70", "{b}" }
          }
          if let Some(ref act) = action {
            Link {
              to: "{act.href}",
              class: "mt-2 inline-flex text-xs font-medium underline underline-offset-4 hover:opacity-90",
              "{act.label}"
            }
          }
        }
        button {
          class: "mt-0.5 shrink-0 rounded p-1 opacity-50 hover:bg-white/10 hover:opacity-100",
          onclick: move |_| state.dismiss(&id),
          span { class: "material-symbols-outlined text-sm", "close" }
        }
      }
    }
  }
}

fn timestamp_ms() -> u64 {
  std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

#[component]
pub fn ToastViewport() -> Element {
  let state = use_context::<ToastState>();
  let toasts = state.toasts;

  use_future(move || async move {
    loop {
      tokio::time::sleep(std::time::Duration::from_millis(500)).await;
      let now = timestamp_ms();
      let expired: Vec<String> = toasts.read().iter().filter(|t| now.saturating_sub(t.created_at) >= t.ttl_ms).map(|t| t.id.clone()).collect();
      for id in expired {
        state.dismiss(&id);
      }
    }
  });

  if toasts.read().is_empty() {
    return rsx! {};
  }

  rsx! {
    aside {
      class: "pointer-events-none fixed bottom-3 left-3 z-[120] w-full max-w-sm px-1",
      "aria-live": "polite",
      ol { class: "flex w-full flex-col-reverse gap-2",
        for toast in toasts.read().iter() {
          {render_toast(toast, state)}
        }
      }
    }
  }
}
