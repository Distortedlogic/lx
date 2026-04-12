use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
  #[default]
  Default,
  Destructive,
  Outline,
  Secondary,
  Ghost,
  Link,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
  #[default]
  Default,
  Xs,
  Sm,
  Lg,
  Icon,
  IconXs,
  IconSm,
  IconLg,
}

fn variant_name(variant: ButtonVariant) -> &'static str {
  match variant {
    ButtonVariant::Default => "default",
    ButtonVariant::Destructive => "destructive",
    ButtonVariant::Outline => "outline",
    ButtonVariant::Secondary => "secondary",
    ButtonVariant::Ghost => "ghost",
    ButtonVariant::Link => "link",
  }
}

fn size_name(size: ButtonSize) -> &'static str {
  match size {
    ButtonSize::Default => "default",
    ButtonSize::Xs => "xs",
    ButtonSize::Sm => "sm",
    ButtonSize::Lg => "lg",
    ButtonSize::Icon => "icon",
    ButtonSize::IconXs => "icon-xs",
    ButtonSize::IconSm => "icon-sm",
    ButtonSize::IconLg => "icon-lg",
  }
}

#[component]
pub fn Button(
  #[props(default)] variant: ButtonVariant,
  #[props(default)] size: ButtonSize,
  #[props(default)] class: String,
  #[props(default)] disabled: bool,
  #[props(default)] r#type: String,
  #[props(default)] onclick: EventHandler<MouseEvent>,
  children: Element,
) -> Element {
  let btn_type = if r#type.is_empty() { "button" } else { &r#type };
  let classes = cn(&["btn", &class]);
  rsx! {
    button {
      "data-slot": "button",
      "data-variant": variant_name(variant),
      "data-size": size_name(size),
      class: "{classes}",
      disabled,
      r#type: "{btn_type}",
      onclick: move |evt| onclick.call(evt),
      {children}
    }
  }
}
