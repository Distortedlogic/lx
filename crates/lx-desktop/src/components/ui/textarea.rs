use dioxus::prelude::*;

use super::cn;

const BASE_TEXTAREA_CLASS: &str = "border-input placeholder:text-muted-foreground focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive dark:bg-input/30 flex field-sizing-content min-h-16 w-full rounded-md border bg-transparent px-3 py-2 text-base shadow-xs transition-[color,box-shadow] outline-none focus-visible:ring-[3px] disabled:cursor-not-allowed disabled:opacity-50 md:text-sm";

#[component]
pub fn Textarea(
  #[props(default)] class: String,
  #[props(default)] placeholder: String,
  #[props(default)] value: String,
  #[props(default)] disabled: bool,
  #[props(default)] oninput: EventHandler<FormEvent>,
) -> Element {
  let classes = cn(&[BASE_TEXTAREA_CLASS, &class]);
  rsx! {
    textarea {
      "data-slot": "textarea",
      class: "{classes}",
      disabled,
      placeholder: "{placeholder}",
      value: "{value}",
      oninput: move |evt| oninput.call(evt),
    }
  }
}
