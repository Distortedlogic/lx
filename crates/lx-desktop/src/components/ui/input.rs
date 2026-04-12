use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Input(
  #[props(default)] class: String,
  #[props(default = "text".to_string())] r#type: String,
  #[props(default)] placeholder: String,
  #[props(default)] value: String,
  #[props(default)] disabled: bool,
  #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element {
  let classes = cn(&["input", &class]);
  rsx! {
    input {
      "data-slot": "input",
      r#type: "{r#type}",
      class: "{classes}",
      disabled,
      placeholder: "{placeholder}",
      value: "{value}",
      oninput: move |evt| oninput.call(evt),
    }
  }
}
