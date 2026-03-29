use dioxus::prelude::*;

use super::cn;

#[component]
pub fn Breadcrumb(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    nav {
      "data-slot": "breadcrumb",
      "aria-label": "breadcrumb",
      class: cn(&[&class]),
      {children}
    }
  }
}

#[component]
pub fn BreadcrumbList(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    ol {
      "data-slot": "breadcrumb-list",
      class: cn(
          &[
              "text-muted-foreground flex flex-wrap items-center gap-1.5 text-sm break-words sm:gap-2.5",
              &class,
          ],
      ),
      {children}
    }
  }
}

#[component]
pub fn BreadcrumbItem(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    li {
      "data-slot": "breadcrumb-item",
      class: cn(&["inline-flex items-center gap-1.5", &class]),
      {children}
    }
  }
}

#[component]
pub fn BreadcrumbLink(#[props(default)] href: String, #[props(default)] class: String, children: Element) -> Element {
  rsx! {
    a {
      "data-slot": "breadcrumb-link",
      href: "{href}",
      class: cn(&["hover:text-foreground transition-colors", &class]),
      {children}
    }
  }
}

#[component]
pub fn BreadcrumbPage(#[props(default)] class: String, children: Element) -> Element {
  rsx! {
    span {
      "data-slot": "breadcrumb-page",
      role: "link",
      "aria-disabled": "true",
      "aria-current": "page",
      class: cn(&["text-foreground font-normal", &class]),
      {children}
    }
  }
}

#[component]
pub fn BreadcrumbSeparator(#[props(default)] class: String) -> Element {
  rsx! {
    li {
      "data-slot": "breadcrumb-separator",
      role: "presentation",
      "aria-hidden": "true",
      class: cn(&["[&>svg]:size-3.5", &class]),
      svg { view_box: "0 0 24 24", class: "size-3.5",
        polyline {
          points: "9 18 15 12 9 6",
          fill: "none",
          stroke: "currentColor",
          stroke_width: "2",
          stroke_linecap: "round",
          stroke_linejoin: "round",
        }
      }
    }
  }
}

#[component]
pub fn BreadcrumbEllipsis(#[props(default)] class: String) -> Element {
  rsx! {
    span {
      "data-slot": "breadcrumb-ellipsis",
      role: "presentation",
      "aria-hidden": "true",
      class: cn(&["flex size-9 items-center justify-center", &class]),
      svg { view_box: "0 0 24 24", class: "size-4",
        circle {
          cx: "12",
          cy: "12",
          r: "1",
          fill: "currentColor",
        }
        circle {
          cx: "5",
          cy: "12",
          r: "1",
          fill: "currentColor",
        }
        circle {
          cx: "19",
          cy: "12",
          r: "1",
          fill: "currentColor",
        }
      }
      span { class: "sr-only", "More" }
    }
  }
}
