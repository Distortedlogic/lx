use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum BadgeVariant {
  #[default]
  Default,
  Secondary,
  Destructive,
  Outline,
  Ghost,
  Link,
}

fn variant_name(variant: BadgeVariant) -> &'static str {
  match variant {
    BadgeVariant::Default => "default",
    BadgeVariant::Secondary => "secondary",
    BadgeVariant::Destructive => "destructive",
    BadgeVariant::Outline => "outline",
    BadgeVariant::Ghost => "ghost",
    BadgeVariant::Link => "link",
  }
}

#[component]
pub fn Badge(#[props(default)] variant: BadgeVariant, #[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&["badge", &class]);
  rsx! {
    span {
      "data-slot": "badge",
      "data-variant": variant_name(variant),
      class: "{classes}",
      {children}
    }
  }
}
