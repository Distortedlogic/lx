use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Textarea(
  #[props(default)] class: String,
  #[props(default)] placeholder: String,
  #[props(default)] value: String,
  #[props(default)] disabled: bool,
  #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element {
  let classes = cn(&["textarea", &class]);
  rsx! {
    textarea {
      "data-slot": "textarea",
      class: "{classes}",
      disabled,
      placeholder: "{placeholder}",
      value: "{value}",
      oninput: move |evt| oninput.call(evt),
    }
  }
}
