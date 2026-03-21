use dioxus::prelude::*;

#[component]
pub fn PageHeader(title: String, #[props(default)] subtitle: Option<String>, #[props(default)] actions: Option<Element>) -> Element {
  rsx! {
      div { class: "section-bar",
          div {
              h1 { class: "text-2xl font-bold text-foreground", "{title}" }
              if let Some(s) = &subtitle {
                  p { class: "text-sm text-muted-foreground mt-1", "{s}" }
              }
          }
          if let Some(acts) = actions {
              div { class: "flex items-center gap-2", {acts} }
          }
      }
  }
}

#[component]
pub fn StatusBadge(status: String) -> Element {
  let color_classes = match status.as_str() {
    "running" | "active" => "bg-green-500/20 text-green-400",
    "idle" | "standby" => "bg-blue-500/20 text-blue-400",
    "error" | "failed" => "bg-red-500/20 text-red-400",
    "waiting" | "paused" => "bg-amber-500/20 text-amber-400",
    _ => "bg-gray-500/20 text-gray-400",
  };
  rsx! {
      span {
          class: "rounded-full px-2 py-0.5 text-xs font-medium inline-flex items-center gap-1",
          class: "{color_classes}",
          span { class: "w-1.5 h-1.5 rounded-full bg-current" }
          "{status}"
      }
  }
}

#[component]
pub fn StatsCard(title: String, value: String, #[props(default)] icon: Option<String>) -> Element {
  rsx! {
      div { class: "bg-gray-800 rounded-lg p-4",
          div { class: "flex items-center justify-between",
              span { class: "text-gray-400 text-sm", "{title}" }
              if let Some(ref icon_str) = icon {
                  span { class: "text-lg", "{icon_str}" }
              }
          }
          div { class: "text-2xl font-bold mt-2", "{value}" }
      }
  }
}

#[component]
pub fn Skeleton(#[props(default = "w-full".to_string())] width: String, #[props(default = "h-4".to_string())] height: String) -> Element {
  rsx! {
      div {
          class: "bg-muted animate-pulse rounded",
          class: "{width}",
          class: "{height}",
      }
  }
}
