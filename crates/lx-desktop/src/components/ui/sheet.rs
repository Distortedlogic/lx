use dioxus::prelude::*;

use super::cn;

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum SheetSide {
  Top,
  #[default]
  Right,
  Bottom,
  Left,
}

fn side_class(side: SheetSide) -> &'static str {
  match side {
    SheetSide::Right => "inset-y-0 right-0 h-full w-3/4 border-l sm:max-w-sm",
    SheetSide::Left => "inset-y-0 left-0 h-full w-3/4 border-r sm:max-w-sm",
    SheetSide::Top => "inset-x-0 top-0 h-auto border-b",
    SheetSide::Bottom => "inset-x-0 bottom-0 h-auto border-t",
  }
}

#[component]
pub fn SheetContent(
  open: Signal<bool>,
  #[props(default)] side: SheetSide,
  #[props(default)] class: String,
  #[props(default = true)] show_close_button: bool,
  children: Element,
) -> Element {
  if !open() {
    return rsx! {};
  }
  let mut open = open;
  rsx! {
    div {
      "data-slot": "sheet-overlay",
      class: "fixed inset-0 z-50 bg-black/50",
      onclick: move |_| open.set(false),
    }
    div {
      "data-slot": "sheet-content",
      class: cn(
          &[
              "bg-background fixed z-50 flex flex-col gap-4 shadow-lg",
              side_class(side),
              &class,
          ],
      ),
      if show_close_button {
        button {
          "data-slot": "sheet-close",
          class: "ring-offset-background focus:ring-ring absolute top-4 right-4 rounded-xs opacity-70 transition-opacity hover:opacity-100 focus:ring-2 focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none",
          onclick: move |_| open.set(false),
          svg { view_box: "0 0 24 24", class: "size-4",
            line {
              x1: "18",
              y1: "6",
              x2: "6",
              y2: "18",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
            }
            line {
              x1: "6",
              y1: "6",
              x2: "18",
              y2: "18",
              stroke: "currentColor",
              stroke_width: "2",
              stroke_linecap: "round",
            }
          }
          span { class: "sr-only", "Close" }
        }
      }
      {children}
    }
  }
}

#[component]
pub fn SheetHeader(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "sheet-header",
      class: cn(&["flex flex-col gap-1.5 p-4", &class]),
      {children}
    }
  }
}

#[component]
pub fn SheetFooter(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "sheet-footer",
      class: cn(&["mt-auto flex flex-col gap-2 p-4", &class]),
      {children}
    }
  }
}

#[component]
pub fn SheetTitle(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    h2 {
      "data-slot": "sheet-title",
      class: cn(&["text-foreground font-semibold", &class]),
      {children}
    }
  }
}

#[component]
pub fn SheetDescription(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    p {
      "data-slot": "sheet-description",
      class: cn(&["text-muted-foreground text-sm", &class]),
      {children}
    }
  }
}
