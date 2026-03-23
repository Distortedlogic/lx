use dioxus::prelude::*;
use pane_tree::{NotificationLevel, PaneNode, Rect, TabsState};
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
  let mut canvas_submenu = use_signal(|| false);

  rsx! {
    div { class: "flex items-center bg-[var(--surface-container)]",
      div { class: "flex flex-1 overflow-x-auto",
        for tab in tabs.iter() {
          {
              let tab_id_click = tab.id.clone();
              let tab_id_close = tab.id.clone();
              let is_active = active_id.as_deref() == Some(tab.id.as_str());
              let bg = if is_active {
                  "bg-[var(--surface-container-high)] border-b-2 border-[var(--primary)]"
              } else {
                  "bg-transparent hover:bg-[var(--primary)]/10 hover:backdrop-blur-sm"
              };
              let tab_icon = {
                  let rects = tab.root().compute_pane_rects(Rect::default());
                  rects.first().map(|(p, _)| p.icon()).unwrap_or("\u{25B8}")
              };
              let icon_color = if is_active {
                  "text-[var(--primary)]"
              } else {
                  "text-[var(--outline)]"
              };
              let highest_level = highest_notification_level(&tabs_state.read(), &tab.id);
              let dot_class = highest_level
                  .map(|level| match level {
                      NotificationLevel::Error => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-[var(--error)]"
                      }
                      NotificationLevel::Attention => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-[var(--warning)] animate-pulse"
                      }
                      NotificationLevel::Warning => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-[var(--warning)]"
                      }
                      NotificationLevel::Success => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-[var(--success)]"
                      }
                      NotificationLevel::Info => {
                          "w-1.5 h-1.5 rounded-full inline-block ml-1.5 bg-[var(--primary)]"
                      }
                  });
              rsx! {
                button {
                  class: "flex items-center gap-1.5 px-4 py-2 text-sm font-medium whitespace-nowrap transition-colors duration-150",
                  class: "{bg}",
                  onclick: move |_| {
                      let mut s = tabs_state.write();
                      s.set_active_and_focus(&tab_id_click);
                      s.clear_tab_notifications(&tab_id_click);
                  },
                  span { class: "text-xs {icon_color}", "{tab_icon}" }
                  "{tab.title}"
                  if let Some(dc) = dot_class {
                    span { class: "{dc}" }
                  }
                  span {
                    class: "ml-1 text-xs text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors duration-150",
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
          class: "px-3 py-2 text-sm font-medium text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)] transition-colors duration-150",
          onclick: move |_| on_new_tab.call(()),
          oncontextmenu: move |evt| {
              evt.prevent_default();
              dropdown_open.set(true);
              canvas_submenu.set(false);
          },
          "+"
        }
        if dropdown_open() {
          div {
            class: "fixed inset-0 z-20",
            onclick: move |_| {
                dropdown_open.set(false);
                canvas_submenu.set(false);
            },
          }
          div { class: "absolute top-full right-0 z-30 mt-1 py-1 bg-[var(--surface-container-high)]/80 backdrop-blur-[12px] rounded-md shadow-ambient min-w-52",
            for kind in PaneKind::ALL {
              {
                  let label = kind.label();
                  let icon = kind.icon();
                  let kind = *kind;
                  let is_canvas = kind == PaneKind::Canvas;
                  rsx! {
                    button {
                      class: "w-full text-left px-3 py-1.5 text-sm hover:bg-[var(--surface-bright)] transition-colors duration-150",
                      onclick: move |_| {
                          if is_canvas {
                              canvas_submenu.set(!canvas_submenu());
                          } else {
                              let id = Uuid::new_v4().to_string();
                              let pane = DesktopPane::make_default(kind, id.clone());
                              add_tab(tabs_state, id, label.to_string(), PaneNode::Leaf(pane));
                              dropdown_open.set(false);
                          }
                      },
                      span { class: "mr-2", "{icon}" }
                      "{label}"
                    }
                  }
              }
            }
            if canvas_submenu() {
              div { class: "border-t border-[var(--outline-variant)]/15 mt-1 pt-1",
                for widget_type in ["log-viewer", "markdown", "json-viewer"] {
                  {
                      let wt = widget_type.to_string();
                      let wt2 = wt.clone();
                      rsx! {
                        button {
                          class: "w-full text-left px-3 py-1.5 text-sm hover:bg-[var(--surface-bright)] pl-6 transition-colors duration-150",
                          onclick: move |_| {
                              let id = Uuid::new_v4().to_string();
                              let pane = DesktopPane::Canvas {
                                  id: id.clone(),
                                  widget_type: wt.clone(),
                                  config: serde_json::Value::Object(Default::default()),
                                  name: None,
                              };
                              add_tab(tabs_state, id, wt.clone(), PaneNode::Leaf(pane));
                              dropdown_open.set(false);
                              canvas_submenu.set(false);
                          },
                          "{wt2}"
                        }
                      }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}

fn highest_notification_level(state: &TabsState<DesktopPane>, tab_id: &str) -> Option<NotificationLevel> {
  let tab = state.tabs.iter().find(|t| t.id == tab_id)?;
  tab.root().all_pane_ids().into_iter().filter_map(|id| state.get_notification(&id).map(|n| n.level)).max()
}
