use dioxus::prelude::*;

use super::cn;

#[component]
pub fn DropdownMenu(open: Signal<bool>, children: Element) -> Element {
  rsx! {
    div { "data-slot": "dropdown-menu", class: "relative inline-block", {children} }
  }
}

#[component]
pub fn DropdownMenuTrigger(open: Signal<bool>, children: Element) -> Element {
  let mut open = open;
  rsx! {
    button {
      "data-slot": "dropdown-menu-trigger",
      onclick: move |_| {
          let v = open();
          open.set(!v);
      },
      {children}
    }
  }
}

#[component]
pub fn DropdownMenuContent(open: Signal<bool>, #[props(default)] class: String, children: Element) -> Element {
  if !open() {
    return rsx! {};
  }
  rsx! {
    div {
      "data-slot": "dropdown-menu-content",
      role: "menu",
      class: cn(
          &[
              "bg-popover text-popover-foreground z-50 min-w-[8rem] overflow-hidden rounded-md border p-1 shadow-md absolute top-full mt-1",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn DropdownMenuItem(
  #[props(default)] class: String,
  #[props(default)] disabled: bool,
  #[props(default)] onclick: EventHandler<MouseEvent>,
  children: Element,
) -> Element {
  let data_disabled = if disabled { Some("true") } else { None };
  rsx! {
    div {
      "data-slot": "dropdown-menu-item",
      role: "menuitem",
      "data-disabled": data_disabled,
      onclick: move |evt| onclick.call(evt),
      class: cn(
          &[
              "focus:bg-accent focus:text-accent-foreground [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[disabled]:pointer-events-none data-[disabled]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn DropdownMenuSeparator(#[props(default)] class: String) -> Element {
  rsx! {
    div {
      "data-slot": "dropdown-menu-separator",
      role: "separator",
      class: cn(&["bg-border -mx-1 my-1 h-px", &class]),
    }
  }
}

#[component]
pub fn DropdownMenuLabel(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "dropdown-menu-label",
      class: cn(&["px-2 py-1.5 text-sm font-medium", &class]),
      {children}
    }
  }
}
