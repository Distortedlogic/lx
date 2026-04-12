use std::rc::Rc;

use dioxus::html::{FileData, HasFileData, HasFormData};
use dioxus::prelude::*;

use super::cn;

struct EmptyFormData;

impl HasFileData for EmptyFormData {
  fn files(&self) -> Vec<FileData> {
    vec![]
  }
}

impl HasFormData for EmptyFormData {
  fn value(&self) -> String {
    String::new()
  }

  fn valid(&self) -> bool {
    true
  }

  fn values(&self) -> Vec<(String, FormValue)> {
    vec![]
  }

  fn as_any(&self) -> &dyn std::any::Any {
    self
  }
}

#[component]
pub fn Checkbox(
  #[props(default)] class: String,
  #[props(default)] checked: bool,
  #[props(default)] disabled: bool,
  #[props(default)] onchange: EventHandler<FormEvent>,
) -> Element {
  let state = if checked { "checked" } else { "unchecked" };
  let classes = cn(&["checkbox", &class]);
  rsx! {
    button {
      "data-slot": "checkbox",
      "data-state": "{state}",
      role: "checkbox",
      aria_checked: "{checked}",
      class: "{classes}",
      disabled,
      onclick: move |_| {
          let data = FormData::new(EmptyFormData);
          onchange.call(Event::new(Rc::new(data), true));
      },
      if checked {
        div { class: "grid place-content-center text-current transition-none",
          svg { view_box: "0 0 24 24", class: "size-3.5",
            polyline {
              points: "20 6 9 17 4 12",
              fill: "none",
              stroke: "currentColor",
              stroke_width: "3",
              stroke_linecap: "round",
              stroke_linejoin: "round",
            }
          }
        }
      }
    }
  }
}
