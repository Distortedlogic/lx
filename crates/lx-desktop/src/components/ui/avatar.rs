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

const BASE_AVATAR_CLASS: &str =
  "group/avatar relative flex size-8 shrink-0 overflow-hidden rounded-full select-none data-[size=lg]:size-10 data-[size=sm]:size-6 data-[size=xs]:size-5";

const AVATAR_FALLBACK_CLASS: &str = "bg-muted text-muted-foreground flex size-full items-center justify-center rounded-full text-sm group-data-[size=sm]/avatar:text-xs group-data-[size=xs]/avatar:text-[10px]";

const AVATAR_BADGE_CLASS: &str = "bg-primary text-primary-foreground ring-background absolute right-0 bottom-0 z-10 inline-flex items-center justify-center rounded-full ring-2 select-none group-data-[size=sm]/avatar:size-2 group-data-[size=sm]/avatar:[&>svg]:hidden group-data-[size=default]/avatar:size-2.5 group-data-[size=default]/avatar:[&>svg]:size-2 group-data-[size=lg]/avatar:size-3 group-data-[size=lg]/avatar:[&>svg]:size-2";

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
  let classes = cn(&[BASE_AVATAR_CLASS, &class]);
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
  let classes = cn(&[AVATAR_FALLBACK_CLASS, &class]);
  rsx! {
    span { "data-slot": "avatar-fallback", class: "{classes}", {children} }
  }
}

#[component]
pub fn AvatarBadge(#[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&[AVATAR_BADGE_CLASS, &class]);
  rsx! {
    span { "data-slot": "avatar-badge", class: "{classes}", {children} }
  }
}

#[component]
pub fn AvatarGroup(#[props(default)] class: String, children: Element) -> Element {
  let classes = cn(&["*:data-[slot=avatar]:ring-background group/avatar-group flex -space-x-2 *:data-[slot=avatar]:ring-2", &class]);
  rsx! {
    div { "data-slot": "avatar-group", class: "{classes}", {children} }
  }
}
