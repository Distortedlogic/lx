use dioxus::prelude::*;
use pane_tree::{NotificationLevel, PaneNode, TabsState};
use uuid::Uuid;

use super::add_tab;
use crate::panes::{DesktopPane, PaneKind};

#[component]
pub fn TabBar(tabs_state: Signal<TabsState<DesktopPane>>, on_new_tab: EventHandler<()>) -> Element {
  let state = tabs_state.read();
  let active_id = state.active_tab_id.clone();
  let tabs = state.tabs.clone();
  drop(state);

  let mut dropdown_open = use_signal(|| false);
  let mut dropdown_input = use_signal(|| None::<(PaneKind, String)>);

  rsx! {
    div { class: "flex items-center bg-gray-950",
      div { class: "flex flex-1 overflow-x-auto",
        for tab in tabs.iter() {
          {
              let tab_id_click = tab.id.clone();
              let tab_id_close = tab.id.clone();
              let is_active = active_id.as_deref() == Some(tab.id.as_str());
              let bg = if is_active {
                  "bg-gray-800 border-b-2 border-blue-400"
              } else {
                  "bg-gray-950 hover:bg-gray-800"
              };
              let highest_level = highest_notification_level(&tabs_state.read(), &tab.id);
              let dot_class = highest_level
                  .map(|level| match level {
                      NotificationLevel::Error => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-red-500"
                      }
                      NotificationLevel::Attention => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-amber-500 animate-pulse"
                      }
                      NotificationLevel::Warning => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-amber-400"
                      }
                      NotificationLevel::Success => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-emerald-500"
                      }
                      NotificationLevel::Info => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-blue-400"
                      }
                  });
              rsx! {
                button {
                  class: "flex items-center gap-1.5 px-4 py-2 text-sm font-medium whitespace-nowrap",
                  class: "{bg}",
                  onclick: move |_| {
                      let mut s = tabs_state.write();
                      s.set_active_and_focus(&tab_id_click);
                      s.clear_tab_notifications(&tab_id_click);
                  },
                  "{tab.title}"
                  if let Some(dc) = dot_class {
                    span { class: "{dc}" }
                  }
                  span {
                    class: "ml-1 text-xs opacity-50 hover:opacity-100",
                    onclick: move |evt| {
                        evt.stop_propagation();
                        tabs_state.write().close_tab(&tab_id_close);
                    },
                    "\u{00D7}"
                  }
                }
              }
          }
        }
      }
      div { class: "relative",
        button {
          class: "px-3 py-2 text-sm font-medium text-gray-400 hover:text-white hover:bg-gray-800",
          onclick: move |_| on_new_tab.call(()),
          oncontextmenu: move |evt| {
              evt.prevent_default();
              dropdown_open.set(true);
              dropdown_input.set(None);
          },
          "+"
        }
        if dropdown_open() {
          div {
            class: "fixed inset-0 z-20",
            onclick: move |_| {
                dropdown_open.set(false);
                dropdown_input.set(None);
            },
          }
          div { class: "absolute top-full right-0 z-30 mt-1 py-1 bg-gray-800 border border-gray-600 rounded-md shadow-lg min-w-52",
            button {
              class: "w-full text-left px-3 py-1.5 text-sm hover:bg-gray-700",
              onclick: move |_| {
                  on_new_tab.call(());
                  dropdown_open.set(false);
                  dropdown_input.set(None);
              },
              "\u{25B8} Terminal"
            }
            button {
              class: "w-full text-left px-3 py-1.5 text-sm hover:bg-gray-700",
              onclick: move |_| dropdown_input.set(Some((PaneKind::Browser, String::new()))),
              "\u{1F310} Browser"
            }
            button {
              class: "w-full text-left px-3 py-1.5 text-sm hover:bg-gray-700",
              onclick: move |_| dropdown_input.set(Some((PaneKind::Editor, String::new()))),
              "\u{25C7} Editor"
            }
            button {
              class: "w-full text-left px-3 py-1.5 text-sm hover:bg-gray-700",
              onclick: move |_| dropdown_input.set(Some((PaneKind::Agent, String::new()))),
              "\u{25CF} Agent"
            }
            button {
              class: "w-full text-left px-3 py-1.5 text-sm hover:bg-gray-700",
              onclick: move |_| dropdown_input.set(Some((PaneKind::Canvas, String::new()))),
              "\u{25FB} Canvas"
            }
            button {
              class: "w-full text-left px-3 py-1.5 text-sm hover:bg-gray-700",
              onclick: move |_| {
                  let id = Uuid::new_v4().to_string();
                  let pane = DesktopPane::Voice {
                      id: id.clone(),
                  };
                  add_tab(tabs_state, id, "Voice".into(), PaneNode::Leaf(pane));
                  dropdown_open.set(false);
                  dropdown_input.set(None);
              },
              "\u{1F3A4} Voice"
            }
            {render_dropdown_input(tabs_state, &dropdown_input, &dropdown_open)}
          }
        }
      }
    }
  }
}

fn render_dropdown_input(
  tabs_state: Signal<TabsState<DesktopPane>>,
  dropdown_input: &Signal<Option<(PaneKind, String)>>,
  dropdown_open: &Signal<bool>,
) -> Element {
  let Some((ref kind, ref input_val)) = *dropdown_input.read() else {
    return rsx! {};
  };
  let kind = *kind;
  let input_val = input_val.clone();
  let mut dropdown_input = *dropdown_input;
  let mut dropdown_open = *dropdown_open;

  if kind == PaneKind::Canvas {
    return rsx! {
      div { class: "border-t border-gray-600 mt-1 pt-1",
        for widget_type in ["log-viewer", "markdown", "json-viewer"] {
          {
              let wt = widget_type.to_string();
              let wt2 = wt.clone();
              rsx! {
                button {
                  class: "w-full text-left px-3 py-1.5 text-sm hover:bg-gray-700 pl-6",
                  onclick: move |_| {
                      let id = Uuid::new_v4().to_string();
                      let pane = DesktopPane::Canvas {
                          id: id.clone(),
                          widget_type: wt.clone(),
                          config: serde_json::Value::Object(Default::default()),
                      };
                      add_tab(tabs_state, id, wt.clone(), PaneNode::Leaf(pane));
                      dropdown_open.set(false);
                      dropdown_input.set(None);
                  },
                  "{wt2}"
                }
              }
          }
        }
      }
    };
  }

  let placeholder = match kind {
    PaneKind::Browser => "Enter URL...",
    PaneKind::Editor => "Enter file path...",
    PaneKind::Agent => "Enter model...",
    _ => "",
  };

  rsx! {
    div { class: "border-t border-gray-600 mt-1 pt-1 px-3 py-1.5 flex items-center gap-2",
      input {
        class: "flex-1 bg-gray-900 text-sm px-2 py-1 rounded border border-gray-600 outline-none",
        placeholder: "{placeholder}",
        value: "{input_val}",
        oninput: move |evt: FormEvent| {
            let current = dropdown_input.read().clone();
            if let Some((k, _)) = current {
                dropdown_input.set(Some((k, evt.value().to_string())));
            }
        },
        onkeydown: move |evt: KeyboardEvent| {
            if evt.key() == Key::Enter {
                let current = dropdown_input.read().clone();
                if let Some((k, val)) = current {
                    create_pane_from_input(tabs_state, k, &val);
                    dropdown_open.set(false);
                    dropdown_input.set(None);
                }
            }
        },
      }
      button {
        class: "px-3 py-1 bg-blue-600 text-sm rounded hover:bg-blue-500",
        onclick: move |_| {
            let current = dropdown_input.read().clone();
            if let Some((k, val)) = current {
                create_pane_from_input(tabs_state, k, &val);
                dropdown_open.set(false);
                dropdown_input.set(None);
            }
        },
        "Open"
      }
    }
  }
}

fn create_pane_from_input(tabs_state: Signal<TabsState<DesktopPane>>, kind: PaneKind, input: &str) {
  let id = Uuid::new_v4().to_string();
  let (title, pane) = match kind {
    PaneKind::Browser => {
      let url = input.to_string();
      let title = url.split('/').nth(2).unwrap_or(&url).to_string();
      (title, DesktopPane::Browser { id: id.clone(), url, devtools: false })
    },
    PaneKind::Editor => {
      let file_path = input.to_string();
      let title = file_path.rsplit('/').next().unwrap_or(&file_path).to_string();
      (title, DesktopPane::Editor { id: id.clone(), file_path, language: None })
    },
    PaneKind::Agent => {
      let model = if input.is_empty() { "claude-sonnet-4-6".to_string() } else { input.to_string() };
      let session_id = Uuid::new_v4().to_string();
      let title = model.clone();
      (title, DesktopPane::Agent { id: id.clone(), session_id, model })
    },
    _ => return,
  };
  add_tab(tabs_state, id, title, PaneNode::Leaf(pane));
}

fn highest_notification_level(state: &TabsState<DesktopPane>, tab_id: &str) -> Option<NotificationLevel> {
  let tab = state.tabs.iter().find(|t| t.id == tab_id)?;
  tab.root().all_pane_ids().into_iter().filter_map(|id| state.get_notification(&id).map(|n| n.level)).max()
}
