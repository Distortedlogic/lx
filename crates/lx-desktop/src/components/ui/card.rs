use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Card(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "card",
      class: cn(
          &[
              "bg-card text-card-foreground flex flex-col gap-6 border py-6 shadow-sm",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn CardHeader(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "card-header",
      class: cn(
          &[
              "@container/card-header grid auto-rows-min grid-rows-[auto_auto] items-start gap-2 px-6 has-data-[slot=card-action]:grid-cols-[1fr_auto] [.border-b]:pb-6",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn CardTitle(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "card-title",
      class: cn(&["leading-none font-semibold", &class]),
      {children}
    }
  }
}

#[component]
pub fn CardDescription(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "card-description",
      class: cn(&["text-muted-foreground text-sm", &class]),
      {children}
    }
  }
}

#[component]
pub fn CardAction(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "card-action",
      class: cn(&["col-start-2 row-span-2 row-start-1 self-start justify-self-end", &class]),
      {children}
    }
  }
}

#[component]
pub fn CardContent(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div { "data-slot": "card-content", class: cn(&["px-6", &class]), {children} }
  }
}

#[component]
pub fn CardFooter(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    div {
      "data-slot": "card-footer",
      class: cn(&["flex items-center px-6 [.border-t]:pt-6", &class]),
      {children}
    }
  }
}
