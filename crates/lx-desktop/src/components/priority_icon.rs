use dioxus::prelude::*;

use super::status_colors::{priority_color_class, priority_label};

#[component]
pub fn PriorityIcon(priority: String, #[props(optional)] class: Option<String>, #[props(default = false)] show_label: bool) -> Element {
  let color = priority_color_class(&priority);
  let extra = class.as_deref().unwrap_or("");

  let icon_span = rsx! {
    span {
      class: "inline-flex items-center justify-center shrink-0",
      class: "{color}",
      class: "{extra}",
      {render_svg(&priority)}
    }
  };

  if show_label {
    let label = priority_label(&priority);
    rsx! {
      span { class: "inline-flex items-center gap-1.5",
        {icon_span}
        span { class: "text-sm", "{label}" }
      }
    }
  } else {
    icon_span
  }
}

fn render_svg(priority: &str) -> Element {
  match priority {
    "critical" => rsx! {
      svg {
        class: "h-3.5 w-3.5",
        view_box: "0 0 24 24",
        stroke: "currentColor",
        fill: "none",
        stroke_width: "2",
        path { d: "M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" }
        line {
          x1: "12",
          y1: "9",
          x2: "12",
          y2: "13",
        }
        line {
          x1: "12",
          y1: "17",
          x2: "12.01",
          y2: "17",
        }
      }
    },
    "high" => rsx! {
      svg {
        class: "h-3.5 w-3.5",
        view_box: "0 0 24 24",
        stroke: "currentColor",
        fill: "none",
        stroke_width: "2",
        path { d: "M12 19V5" }
        path { d: "M5 12l7-7 7 7" }
      }
    },
    "medium" => rsx! {
      svg {
        class: "h-3.5 w-3.5",
        view_box: "0 0 24 24",
        stroke: "currentColor",
        fill: "none",
        stroke_width: "2",
        path { d: "M5 12h14" }
      }
    },
    "low" => rsx! {
      svg {
        class: "h-3.5 w-3.5",
        view_box: "0 0 24 24",
        stroke: "currentColor",
        fill: "none",
        stroke_width: "2",
        path { d: "M12 5v14" }
        path { d: "M19 12l-7 7-7-7" }
      }
    },
    _ => rsx! {
      svg {
        class: "h-3.5 w-3.5",
        view_box: "0 0 24 24",
        stroke: "currentColor",
        fill: "none",
        stroke_width: "2",
        path { d: "M5 12h14" }
      }
    },
  }
}
