use dioxus::prelude::*;

#[component]
fn Skeleton(class: String) -> Element {
  rsx! {
    div { class: "animate-pulse bg-[var(--outline-variant)]/50 rounded {class}" }
  }
}

#[component]
pub fn PageSkeleton(#[props(default = "list".to_string())] variant: String) -> Element {
  match variant.as_str() {
    "dashboard" => rsx! {
      div { class: "space-y-6",
        Skeleton { class: "h-32 w-full".to_string() }
        div { class: "grid grid-cols-4 gap-4",
          Skeleton { class: "h-24 w-full".to_string() }
          Skeleton { class: "h-24 w-full".to_string() }
          Skeleton { class: "h-24 w-full".to_string() }
          Skeleton { class: "h-24 w-full".to_string() }
        }
        div { class: "grid grid-cols-4 gap-4",
          Skeleton { class: "h-44 w-full".to_string() }
          Skeleton { class: "h-44 w-full".to_string() }
          Skeleton { class: "h-44 w-full".to_string() }
          Skeleton { class: "h-44 w-full".to_string() }
        }
        div { class: "grid grid-cols-2 gap-4",
          Skeleton { class: "h-72 w-full".to_string() }
          Skeleton { class: "h-72 w-full".to_string() }
        }
      }
    },
    "detail" => rsx! {
      div { class: "space-y-6",
        div { class: "space-y-3",
          Skeleton { class: "h-3 w-64".to_string() }
          div { class: "flex gap-2",
            Skeleton { class: "h-5 w-16".to_string() }
            Skeleton { class: "h-5 w-16".to_string() }
            Skeleton { class: "h-5 w-16".to_string() }
          }
          Skeleton { class: "h-4 w-40".to_string() }
        }
        div { class: "space-y-3",
          Skeleton { class: "h-10 w-full".to_string() }
          Skeleton { class: "h-32 w-full".to_string() }
        }
        div { class: "space-y-3",
          div { class: "flex gap-2",
            Skeleton { class: "h-8 w-20".to_string() }
            Skeleton { class: "h-8 w-20".to_string() }
            Skeleton { class: "h-8 w-20".to_string() }
          }
          Skeleton { class: "h-48 w-full".to_string() }
          Skeleton { class: "h-48 w-full".to_string() }
        }
      }
    },
    _ => rsx! {
      div { class: "space-y-4",
        div { class: "flex items-center justify-between",
          Skeleton { class: "h-9 w-44".to_string() }
          div { class: "flex gap-2",
            Skeleton { class: "h-9 w-24".to_string() }
            Skeleton { class: "h-9 w-24".to_string() }
          }
        }
        Skeleton { class: "h-11 w-full".to_string() }
        Skeleton { class: "h-11 w-full".to_string() }
        Skeleton { class: "h-11 w-full".to_string() }
        Skeleton { class: "h-11 w-full".to_string() }
        Skeleton { class: "h-11 w-full".to_string() }
        Skeleton { class: "h-11 w-full".to_string() }
        Skeleton { class: "h-11 w-full".to_string() }
      }
    },
  }
}
