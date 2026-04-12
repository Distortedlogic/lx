use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
  #[default]
  Horizontal,
  Vertical,
}

#[component]
pub fn Separator(#[props(default)] orientation: Orientation, #[props(default = true)] decorative: bool, #[props(default)] class: String) -> Element {
  let orientation_str = match orientation {
    Orientation::Horizontal => "horizontal",
    Orientation::Vertical => "vertical",
  };
  let role = if decorative { "none" } else { "separator" };
  let classes = cn(&["separator", &class]);
  rsx! {
    div {
      "data-slot": "separator",
      role: "{role}",
      aria_orientation: "{orientation_str}",
      "data-orientation": "{orientation_str}",
      class: "{classes}",
    }
  }
}
