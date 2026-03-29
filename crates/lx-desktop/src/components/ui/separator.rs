use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Orientation {
  #[default]
  Horizontal,
  Vertical,
}

const BASE_SEPARATOR_CLASS: &str = "bg-border shrink-0";

#[component]
pub fn Separator(#[props(default)] orientation: Orientation, #[props(default = true)] decorative: bool, #[props(default)] class: String) -> Element {
  let orientation_str = match orientation {
    Orientation::Horizontal => "horizontal",
    Orientation::Vertical => "vertical",
  };
  let orientation_class = match orientation {
    Orientation::Horizontal => "h-px w-full",
    Orientation::Vertical => "h-full w-px",
  };
  let role = if decorative { "none" } else { "separator" };
  let classes = cn(&[BASE_SEPARATOR_CLASS, orientation_class, &class]);
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
