use std::rc::Rc;

use dioxus::prelude::*;

use super::cn;

const BASE_CHECKBOX_CLASS: &str = "peer border-input dark:bg-input/30 data-[state=checked]:bg-primary data-[state=checked]:text-primary-foreground dark:data-[state=checked]:bg-primary data-[state=checked]:border-primary focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive size-4 shrink-0 rounded-[4px] border shadow-xs transition-shadow outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50";

#[component]
pub fn Checkbox(
  #[props(default)] class: String,
  #[props(default)] checked: bool,
  #[props(default)] disabled: bool,
  #[props(default)] onchange: EventHandler<FormEvent>,
) -> Element {
  let state = if checked { "checked" } else { "unchecked" };
  let classes = cn(&[BASE_CHECKBOX_CLASS, &class]);
  rsx! {
    button {
      "data-slot": "checkbox",
      "data-state": "{state}",
      role: "checkbox",
      aria_checked: "{checked}",
      class: "{classes}",
      disabled,
      onclick: move |_| {
          let data = FormData::new(SerializedFormData::new(String::new(), vec![]));
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
