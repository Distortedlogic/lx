use common_pane_tree::{Pane, PaneNode};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::panes::{DesktopPane, PaneKind};
use crate::terminal::status_badge::{BadgeVariant, StatusBadge};

#[component]
pub fn PaneToolbar(
  pane: DesktopPane,
  on_split_h: EventHandler,
  on_split_v: EventHandler,
  on_close: EventHandler,
  on_navigate: Option<EventHandler<String>>,
  on_convert: EventHandler<PaneNode<DesktopPane>>,
  current_url: ReadSignal<String>,
) -> Element {
  let initial_url = match &pane {
    DesktopPane::Browser { url, .. } => url.clone(),
    _ => String::new(),
  };
  let mut url_input = use_signal(|| initial_url);
  use_effect(move || {
    let val = current_url.read().clone();
    if !val.is_empty() {
      url_input.set(val);
    }
  });
  let mut conversion_open = use_signal(|| false);
  let current_kind = pane.kind();
  let icon = pane.icon();
  let pane_title = derive_pane_title(&pane);

  let left_section = match &pane {
    DesktopPane::Browser { .. } => {
      let nav = on_navigate;
      let nav2 = on_navigate;
      let nav3 = on_navigate;
      let nav4 = on_navigate;
      rsx! {
        button {
          class: "px-1.5 py-0.5 bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)] transition-colors duration-150",
          onclick: move |_| {
              if let Some(ref h) = nav {
                  h.call("back".into());
              }
          },
          "\u{2190}"
        }
        button {
          class: "px-1.5 py-0.5 bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)] transition-colors duration-150",
          onclick: move |_| {
              if let Some(ref h) = nav2 {
                  h.call("forward".into());
              }
          },
          "\u{2192}"
        }
        button {
          class: "px-1.5 py-0.5 bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)] transition-colors duration-150",
          onclick: move |_| {
              if let Some(ref h) = nav3 {
                  h.call("refresh".into());
              }
          },
          "\u{21BB}"
        }
        input {
          class: "flex-1 bg-[var(--surface-container-lowest)] rounded text-xs px-1.5 py-0.5 outline-none focus:bg-[var(--surface-container-low)] focus:border-b focus:border-[var(--primary)] transition-colors duration-150",
          value: "{url_input}",
          oninput: move |evt| url_input.set(evt.value()),
          onkeydown: move |evt: KeyboardEvent| {
              if evt.key() == Key::Enter && let Some(ref h) = nav4 {
                  h.call(url_input());
              }
          },
        }
      }
    },
    DesktopPane::Editor { file_path, .. } => {
      let mut path_input = use_signal(|| file_path.clone());
      rsx! {
        span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider mr-1",
          "FILE"
        }
        input {
          class: "flex-1 bg-[var(--surface-container-lowest)] rounded text-xs px-1.5 py-0.5 outline-none focus:bg-[var(--surface-container-low)] focus:border-b focus:border-[var(--primary)] transition-colors duration-150 font-mono",
          value: "{path_input}",
          placeholder: "Enter file path...",
          oninput: move |evt| path_input.set(evt.value()),
          onkeydown: move |evt: KeyboardEvent| {
              if evt.key() == Key::Enter {
                  let new_id = uuid::Uuid::new_v4().to_string();
                  let new_pane = PaneNode::Leaf(DesktopPane::Editor {
                      id: new_id,
                      file_path: path_input(),
                      language: None,
                      name: None,
                  });
                  on_convert.call(new_pane);
              }
          },
        }
      }
    },
    _ => {
      rsx! {
        span { class: "text-xs text-[var(--primary)] uppercase font-semibold tracking-[0.05em] truncate",
          "{pane_title}"
        }
      }
    },
  };

  rsx! {
    div { class: "flex items-center h-8 px-2 gap-1 bg-[var(--surface-container-high)] text-xs shrink-0",
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
          div { class: "absolute top-full left-0 z-30 mt-1 py-1 bg-[var(--surface-container-high)]/80 backdrop-blur-[12px] rounded-md shadow-ambient min-w-36",
            for kind in PaneKind::ALL {
              if *kind != current_kind {
                {
                    let kind = *kind;
                    rsx! {
                      button {
                        class: "flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-[var(--surface-bright)] transition-colors duration-150",
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
            div { class: "h-px bg-[var(--outline-variant)]/15 my-1" }
            button {
              class: "flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-[var(--surface-bright)] transition-colors duration-150",
              onclick: move |evt| {
                  evt.stop_propagation();
                  on_split_h.call(());
                  conversion_open.set(false);
              },
              span { "\u{21E5}" }
              span { "Split Right" }
            }
            button {
              class: "flex items-center gap-2 w-full px-3 py-1.5 text-left hover:bg-[var(--surface-bright)] transition-colors duration-150",
              onclick: move |evt| {
                  evt.stop_propagation();
                  on_split_v.call(());
                  conversion_open.set(false);
              },
              span { "\u{21E4}" }
              span { "Split Down" }
            }
          }
        }
      }
      {left_section}
      div { class: "flex-1" }
      if pane.kind() == PaneKind::Terminal {
        {
            let tabs = crate::terminal::use_tabs_state();
            let notification = tabs.read().get_notification(pane.pane_id()).cloned();
            let (label, variant) = match notification.as_ref().map(|n| n.level) {
                Some(common_pane_tree::NotificationLevel::Success) => {
                    ("EXITED".to_string(), BadgeVariant::Idle)
                }
                Some(common_pane_tree::NotificationLevel::Error) => {
                    ("ERROR".to_string(), BadgeVariant::Idle)
                }
                _ => ("ACTIVE".to_string(), BadgeVariant::Active),
            };
            rsx! {
              StatusBadge { label, variant }
            }
        }
      }
      button {
        class: "px-1.5 py-0.5 bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)] transition-colors duration-150",
        onclick: move |evt| {
            evt.stop_propagation();
            on_split_h.call(());
        },
        "\u{229F}"
      }
      button {
        class: "px-1.5 py-0.5 bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)] transition-colors duration-150",
        onclick: move |evt| {
            evt.stop_propagation();
            on_split_v.call(());
        },
        "\u{229E}"
      }
      button {
        class: "px-1.5 py-0.5 bg-[var(--surface-container-highest)]/80 rounded hover:bg-[var(--surface-bright)] transition-colors duration-150",
        onclick: move |evt| {
            evt.stop_propagation();
            on_close.call(());
        },
        "\u{00D7}"
      }
    }
  }
}

fn derive_pane_title(pane: &DesktopPane) -> String {
  if let Some(n) = pane.name() {
    return n.to_string();
  }
  match pane {
    DesktopPane::Terminal { working_dir, command, .. } => {
      if let Some(cmd) = command {
        cmd.split_whitespace().next().unwrap_or("terminal").to_string()
      } else {
        working_dir.clone()
      }
    },
    DesktopPane::Browser { url, .. } => url.split('/').nth(2).unwrap_or("browser").to_string(),
    DesktopPane::Editor { file_path, .. } => file_path.rsplit('/').next().unwrap_or("editor").to_string(),
    DesktopPane::Agent { model, .. } => model.clone(),
    DesktopPane::Canvas { widget_type, .. } => widget_type.clone(),
    DesktopPane::Chart { title, .. } => title.as_deref().unwrap_or("chart").to_string(),
  }
}
