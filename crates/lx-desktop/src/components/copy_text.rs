use dioxus::prelude::*;

#[component]
pub fn CopyText(text: String, #[props(optional)] children: Option<Element>, #[props(optional)] class: Option<String>) -> Element {
  let mut copied = use_signal(|| false);
  let extra = class.as_deref().unwrap_or("");

  let handle_click = {
    let text = text.clone();
    move |_| {
      let text = text.clone();
      document::eval(&format!("navigator.clipboard.writeText(\"{text}\")"));
      copied.set(true);
      spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
        copied.set(false);
      });
    }
  };

  let opacity_class = if copied() { "opacity-100" } else { "opacity-0" };

  rsx! {
    span { class: "relative inline-flex",
      button {
        class: "cursor-copy hover:text-white transition-colors",
        class: "{extra}",
        onclick: handle_click,
        if let Some(ch) = children {
          {ch}
        } else {
          "{text}"
        }
      }
      span {
        class: "pointer-events-none absolute left-1/2 -translate-x-1/2 bottom-full mb-1.5 rounded-md bg-white text-black px-2 py-1 text-xs whitespace-nowrap transition-opacity duration-300",
        class: "{opacity_class}",
        if copied() {
          "Copied!"
        } else {
          ""
        }
      }
    }
  }
}
