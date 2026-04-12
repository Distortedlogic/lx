use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum AvatarSize {
  #[default]
  Default,
  Xs,
  Sm,
  Lg,
}

fn size_name(size: AvatarSize) -> &'static str {
  match size {
    AvatarSize::Default => "default",
    AvatarSize::Xs => "xs",
    AvatarSize::Sm => "sm",
    AvatarSize::Lg => "lg",
  }
}

#[component]
pub fn Avatar(#[props(default)] size: AvatarSize, #[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&["avatar", &class]);
  rsx! {
    span {
      "data-slot": "avatar",
      "data-size": size_name(size),
      class: "{classes}",
      {children}
    }
  }
}

#[component]
pub fn AvatarImage(src: String, #[props(default)] alt: String, #[props(default)] class: String) -> Element {
  let classes = cn(&["aspect-square size-full", &class]);
  rsx! {
    img {
      "data-slot": "avatar-image",
      src: "{src}",
      alt: "{alt}",
      class: "{classes}",
    }
  }
}

#[component]
pub fn AvatarFallback(#[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&["avatar-fallback", &class]);
  rsx! {
    span { "data-slot": "avatar-fallback", class: "{classes}", {children} }
  }
}

#[component]
pub fn AvatarBadge(#[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&["avatar-badge", &class]);
  rsx! {
    span { "data-slot": "avatar-badge", class: "{classes}", {children} }
  }
}

#[component]
pub fn AvatarGroup(#[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&["avatar-group", &class]);
  rsx! {
    div { "data-slot": "avatar-group", class: "{classes}", {children} }
  }
}
