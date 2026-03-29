use dioxus::prelude::*;

#[component]
pub fn InlineEditor(
  value: String,
  on_save: EventHandler<String>,
  #[props(default = "Click to edit...".to_string())] placeholder: String,
  #[props(optional)] class: Option<String>,
) -> Element {
  let mut editing = use_signal(|| false);
  let mut draft = use_signal(|| value.clone());
  let extra = class.as_deref().unwrap_or("");

  {
    let value = value.clone();
    use_effect(move || {
      draft.set(value.clone());
    });
  }

  if editing() {
    let original = value.clone();
    rsx! {
      input {
        class: "w-full bg-transparent rounded outline-none px-1 -mx-1 {extra}",
        value: "{draft}",
        oninput: move |evt: Event<FormData>| draft.set(evt.value()),
        onblur: {
            let original = original.clone();
            move |_| {
                let trimmed = draft().trim().to_string();
                if trimmed != original {
                    on_save.call(trimmed);
                }
                editing.set(false);
            }
        },
        onkeydown: {
            let original = original.clone();
            move |evt: Event<KeyboardData>| {
                match evt.key() {
                    Key::Enter => {
                        let trimmed = draft().trim().to_string();
                        if trimmed != original {
                            on_save.call(trimmed);
                        }
                        editing.set(false);
                    }
                    Key::Escape => {
                        draft.set(original.clone());
                        editing.set(false);
                    }
                    _ => {}
                }
            }
        },
        autofocus: true,
      }
    }
  } else {
    let is_empty = value.is_empty();
    rsx! {
      span {
        class: "cursor-pointer rounded hover:bg-white/5 transition-colors px-1 -mx-1 {extra}",
        onclick: move |_| editing.set(true),
        if is_empty {
          span { class: "text-gray-400 italic", "{placeholder}" }
        } else {
          "{value}"
        }
      }
    }
  }
}
