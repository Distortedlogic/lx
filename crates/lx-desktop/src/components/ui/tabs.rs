use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Tabs(active_tab: Signal<String>, #[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "tabs",
      "data-orientation": "horizontal",
      class: cn(&["group/tabs flex gap-2 flex-col", &class]),
      {children}
    }
  }
}

#[component]
pub fn TabsList(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "tabs-list",
      role: "tablist",
      class: cn(
          &[
              "bg-muted p-[3px] h-9 group/tabs-list text-muted-foreground inline-flex w-fit items-center justify-center",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn TabsTrigger(value: String, active_tab: Signal<String>, #[props(default)] class: String, children: Element) -> Element {
  let is_active = active_tab() == value;
  let aria_selected = if is_active { "true" } else { "false" };
  let data_state = if is_active { "active" } else { "inactive" };
  let mut active_tab = active_tab;
  let value_clone = value.clone();
  rsx! {
    button {
      "data-slot": "tabs-trigger",
      role: "tab",
      "aria-selected": aria_selected,
      "data-state": data_state,
      onclick: move |_| active_tab.set(value_clone.clone()),
      class: cn(
          &[
              "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:outline-ring text-foreground/60 hover:text-foreground dark:text-muted-foreground dark:hover:text-foreground relative inline-flex h-[calc(100%-1px)] flex-1 items-center justify-center gap-1.5 border border-transparent px-2 py-1 text-sm font-medium whitespace-nowrap transition-[color,background-color,border-color,box-shadow] focus-visible:ring-[3px] focus-visible:outline-1 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 data-[state=active]:bg-background dark:data-[state=active]:text-foreground dark:data-[state=active]:border-input dark:data-[state=active]:bg-input/30 data-[state=active]:text-foreground data-[state=active]:shadow-sm",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn TabsContent(value: String, active_tab: Signal<String>, #[props(default)] class: String, children: Element) -> Element {
  if active_tab() != value {
    return rsx! {};
  }
  rsx! {
    div {
      "data-slot": "tabs-content",
      role: "tabpanel",
      class: cn(&["flex-1 outline-none", &class]),
      {children}
    }
  }
}
