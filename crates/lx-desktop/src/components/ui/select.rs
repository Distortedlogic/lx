use dioxus::prelude::*;

use super::cn;

const BASE_SELECT_TRIGGER_CLASS: &str = "border-input data-[placeholder]:text-muted-foreground [&_svg:not([class*='text-'])]:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 dark:hover:bg-input/50 flex w-fit items-center justify-between gap-2 rounded-md border bg-transparent px-3 py-2 text-sm whitespace-nowrap shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 h-9";

#[component]
pub fn Select(
  #[props(default)] class: String,
  #[props(default)] value: String,
  #[props(default)] disabled: bool,
  #[props(default)] onchange: EventHandler<FormEvent>,
  children: Element,
) -> Element {
  let classes = cn(&[BASE_SELECT_TRIGGER_CLASS, &class]);
  rsx! {
    div { "data-slot": "select",
      select {
        "data-slot": "select-trigger",
        class: "{classes}",
        disabled,
        value: "{value}",
        onchange: move |evt| onchange.call(evt),
        {children}
      }
    }
  }
}

#[component]
pub fn SelectItem(value: String, #[props(default)] disabled: bool, children: Element) -> Element {
  rsx! {
    option { value: "{value}", disabled, {children} }
  }
}
