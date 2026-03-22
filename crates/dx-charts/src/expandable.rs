use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct ChartExpanded(pub bool);

#[component]
pub fn ExpandableChart(#[props(default)] header: Option<Element>, children: Element) -> Element {
  let mut expanded = use_signal(|| false);
  provide_context(ChartExpanded(expanded()));

  if expanded() {
    rsx! {
      div { class: "card-sm",
        div {
          class: "chart-height flex items-center justify-center text-muted-foreground text-sm cursor-pointer",
          onclick: move |_| expanded.set(false),
          "Chart expanded \u{2014} click to collapse"
        }
      }
      div {
        class: "fixed inset-0 z-50 bg-black/80 flex items-center justify-center",
        onclick: move |_| expanded.set(false),
        div {
          class: "bg-gray-900 border border-gray-700 rounded-lg p-4 max-w-6xl w-full mx-4",
          onclick: move |e| e.stop_propagation(),
          div { class: "flex items-center justify-between mb-4",
            div { class: "flex items-center gap-2",
              if let Some(h) = header {
                div { class: "text-xs font-mono flex flex-row flex-wrap gap-x-3 gap-y-0.5",
                  {h}
                }
              }
            }
            button {
              class: "text-gray-400 hover:text-white p-1 text-lg",
              onclick: move |_| expanded.set(false),
              "\u{2715}"
            }
          }
          div { class: "h-[70vh]", {children} }
        }
      }
    }
  } else {
    rsx! {
      div { class: "card-sm group relative",
        if header.is_some() {
          div { class: "flex items-center justify-between",
            div { class: "flex items-center gap-2",
              if let Some(h) = header {
                div { class: "bg-background/80 backdrop-blur-xs rounded px-2 py-1 text-xs font-mono flex flex-row flex-wrap gap-x-3 gap-y-0.5",
                  {h}
                }
              }
              button {
                class: "text-gray-400 hover:text-white p-0.5 opacity-0 group-hover:opacity-100 transition-opacity text-lg",
                onclick: move |_| expanded.set(true),
                "\u{2922}"
              }
            }
          }
        } else {
          div { class: "flex justify-end",
            button {
              class: "text-gray-400 hover:text-white p-0.5 opacity-0 group-hover:opacity-100 transition-opacity text-lg",
              onclick: move |_| expanded.set(true),
              "\u{2922}"
            }
          }
        }
        div { class: "chart-height", {children} }
      }
    }
  }
}
