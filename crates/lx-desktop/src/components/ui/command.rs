use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Command(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "command",
      class: cn(
          &[
              "bg-popover text-popover-foreground flex h-full w-full flex-col overflow-hidden rounded-md",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn CommandInput(
  #[props(default)] class: String,
  #[props(default)] placeholder: String,
  #[props(default)] value: String,
  #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element {
  rsx! {
    div {
      "data-slot": "command-input-wrapper",
      class: "flex h-9 items-center gap-2 border-b px-3",
      svg { view_box: "0 0 24 24", class: "size-4 shrink-0 opacity-50",
        circle {
          cx: "11",
          cy: "11",
          r: "8",
          fill: "none",
          stroke: "currentColor",
          stroke_width: "2",
        }
        line {
          x1: "21",
          y1: "21",
          x2: "16.65",
          y2: "16.65",
          stroke: "currentColor",
          stroke_width: "2",
          stroke_linecap: "round",
        }
      }
      input {
        "data-slot": "command-input",
        class: cn(
            &[
                "placeholder:text-muted-foreground flex h-10 w-full rounded-md bg-transparent py-3 text-sm outline-hidden disabled:cursor-not-allowed disabled:opacity-50",
                &class,
            ],
        ),
        placeholder,
        value,
        oninput: move |evt| oninput.call(evt),
      }
    }
  }
}

#[component]
pub fn CommandList(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "command-list",
      class: cn(&["max-h-[300px] scroll-py-1 overflow-x-hidden overflow-y-auto", &class]),
      {children}
    }
  }
}

#[component]
pub fn CommandEmpty(children: Element) -> Element {
  rsx! {
    div { "data-slot": "command-empty", class: "py-6 text-center text-sm", {children} }
  }
}

#[component]
pub fn CommandGroup(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "command-group",
      class: cn(&["text-foreground overflow-hidden p-1", &class]),
      {children}
    }
  }
}

#[component]
pub fn CommandItem(#[props(default)] class: String, #[props(default)] onclick: EventHandler<MouseEvent>, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "command-item",
      onclick: move |evt| onclick.call(evt),
      class: cn(
          &[
              "data-[selected=true]:bg-accent data-[selected=true]:text-accent-foreground [&_svg:not([class*='text-'])]:text-muted-foreground relative flex cursor-default items-center gap-2 rounded-sm px-2 py-1.5 text-sm outline-hidden select-none data-[disabled=true]:pointer-events-none data-[disabled=true]:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn CommandSeparator() -> Element {
  rsx! {
    div { "data-slot": "command-separator", class: "bg-border -mx-1 h-px" }
  }
}
