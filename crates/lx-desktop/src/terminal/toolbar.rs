use dioxus::prelude::*;
use pane_tree::PaneNode;
use uuid::Uuid;

use crate::panes::{DesktopPane, PaneKind};

#[component]
pub fn PaneToolbar(
  pane: DesktopPane,
  on_split_h: EventHandler,
  on_split_v: EventHandler,
  on_close: EventHandler,
  on_navigate: Option<EventHandler<String>>,
  on_convert: EventHandler<PaneNode<DesktopPane>>,
) -> Element {
  let initial_url = match &pane {
    DesktopPane::Browser { url, .. } => url.clone(),
    _ => String::new(),
  };
  let mut url_input = use_signal(|| initial_url);
  let mut conversion_open = use_signal(|| false);
  let current_kind = pane.kind();
  let icon = pane.icon();

  let left_section = match &pane {
    DesktopPane::Terminal { working_dir, .. } => {
      let truncated = truncate_path(working_dir, 2);
      rsx! {
        span { class: "truncate", "{truncated}" }
      }
    },
    DesktopPane::Browser { .. } => {
      let nav = on_navigate;
      let nav2 = on_navigate;
      let nav3 = on_navigate;
      let nav4 = on_navigate;
      rsx! {
        button {
          class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
          onclick: move |_| {
              if let Some(ref h) = nav {
                  h.call("back".into());
              }
          },
          "\u{2190}"
        }
        button {
          class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
          onclick: move |_| {
              if let Some(ref h) = nav2 {
                  h.call("forward".into());
              }
          },
          "\u{2192}"
        }
        button {
          class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
          onclick: move |_| {
              if let Some(ref h) = nav3 {
                  h.call("refresh".into());
              }
          },
          "\u{21BB}"
        }
        input {
          class: "flex-1 bg-gray-900 border border-gray-600 rounded text-xs px-1.5 py-0.5",
          value: "{url_input}",
          oninput: move |evt| url_input.set(evt.value()),
          onkeypress: move |evt: KeyboardEvent| {
              if evt.key() == Key::Enter && let Some(ref h) = nav4 {
                  h.call(url_input());
              }
          },
        }
      }
    },
    DesktopPane::Editor { file_path, .. } => {
      let basename = file_path.rsplit('/').next().unwrap_or(file_path);
      rsx! {
        span { class: "truncate", "{basename}" }
      }
    },
    DesktopPane::Agent { model, .. } => {
      rsx! {
        span { class: "truncate", "{model}" }
      }
    },
    DesktopPane::Canvas { widget_type, .. } => {
      rsx! {
        span { class: "truncate", "{widget_type}" }
      }
    },
    DesktopPane::Chart { title, .. } => {
      let label = title.as_deref().unwrap_or("Chart");
      rsx! {
        span { class: "truncate", "{label}" }
      }
    },
    DesktopPane::Voice { .. } => {
      rsx! {
        span { class: "truncate", "Voice" }
      }
    },
  };

  rsx! {
    div { class: "flex items-center h-8 px-2 gap-1 bg-gray-800 border-b border-gray-700 opacity-0 group-hover:opacity-100 transition-opacity text-xs shrink-0",
      div { class: "relative",
        span {
          class: "cursor-pointer hover:opacity-70",
          onclick: move |evt| {
              evt.stop_propagation();
              conversion_open.set(!conversion_open());
          },
          "{icon}"
        }
        if conversion_open() {
          div { class: "absolute top-full left-0 z-30 mt-1 py-1 bg-gray-800 border border-gray-600 rounded-md shadow-lg min-w-36",
            for kind in PaneKind::ALL {
              if *kind != current_kind {
                {
                    let kind = *kind;
                    rsx! {
                      button {
                        class: "flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-gray-700",
                        onclick: move |evt| {
                            evt.stop_propagation();
                            let new_id = Uuid::new_v4().to_string();
                            let new_node = PaneNode::Leaf(DesktopPane::make_default(kind, new_id));
                            on_convert.call(new_node);
                            conversion_open.set(false);
                        },
                        span { "{kind.icon()}" }
                        span { "{kind.label()}" }
                      }
                    }
                }
              }
            }
          }
        }
      }
      {left_section}
      div { class: "flex-1" }
      button {
        class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
        onclick: move |evt| {
            evt.stop_propagation();
            on_split_h.call(());
        },
        "\u{21E5}"
      }
      button {
        class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
        onclick: move |evt| {
            evt.stop_propagation();
            on_split_v.call(());
        },
        "\u{21E4}"
      }
      button {
        class: "px-1.5 py-0.5 bg-gray-700 rounded hover:bg-gray-600",
        onclick: move |evt| {
            evt.stop_propagation();
            on_close.call(());
        },
        "\u{00D7}"
      }
    }
  }
}

fn truncate_path(path: &str, components: usize) -> String {
  let parts: Vec<&str> = path.rsplitn(components + 1, '/').collect();
  if parts.len() <= components { path.to_string() } else { parts[..components].iter().rev().copied().collect::<Vec<_>>().join("/") }
}
