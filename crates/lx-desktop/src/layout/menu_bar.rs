use std::env;

use common_pane_tree::{PaneNode, SplitDirection, TabsState};
use dioxus::prelude::*;

use crate::panes::DesktopPane;
use crate::terminal::add_terminal_tab;

struct MenuItem {
  label: &'static str,
  shortcut: Option<&'static str>,
  action: Option<EventHandler<()>>,
}

struct Menu {
  label: &'static str,
  items: Vec<MenuItem>,
}

fn make_new_terminal_action(tabs_state: Signal<TabsState<DesktopPane>>) -> EventHandler<()> {
  EventHandler::new(move |_| {
    let id = uuid::Uuid::new_v4().to_string();
    let wd = env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into());
    add_terminal_tab(tabs_state, id, "Terminal".into(), wd, None);
  })
}

fn build_menus(mut tabs_state: Signal<TabsState<DesktopPane>>) -> Vec<Menu> {
  let close_tab_action = EventHandler::new(move |_| {
    let active_id = tabs_state.read().active_tab_id.clone();
    if let Some(id) = active_id {
      tabs_state.write().close_tab(&id);
    }
  });

  let quit_action = EventHandler::new(move |_| {
    #[cfg(feature = "desktop")]
    dioxus::desktop::window().close();
  });

  let split_right_action = EventHandler::new(move |_| {
    let focused = tabs_state.read().focused_pane_id.clone();
    if let Some(fid) = focused {
      let new_id = uuid::Uuid::new_v4().to_string();
      let wd = env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into());
      let new_pane = DesktopPane::Terminal { id: new_id, working_dir: wd, command: None, name: None };
      tabs_state.write().split_pane(&fid, SplitDirection::Horizontal, PaneNode::Leaf(new_pane));
    }
  });

  let split_down_action = EventHandler::new(move |_| {
    let focused = tabs_state.read().focused_pane_id.clone();
    if let Some(fid) = focused {
      let new_id = uuid::Uuid::new_v4().to_string();
      let wd = env::current_dir().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| ".".into());
      let new_pane = DesktopPane::Terminal { id: new_id, working_dir: wd, command: None, name: None };
      tabs_state.write().split_pane(&fid, SplitDirection::Vertical, PaneNode::Leaf(new_pane));
    }
  });

  vec![
    Menu {
      label: "FILE",
      items: vec![
        MenuItem { label: "New Tab", shortcut: Some("Ctrl+T"), action: Some(make_new_terminal_action(tabs_state)) },
        MenuItem { label: "Close Tab", shortcut: Some("Ctrl+W"), action: Some(close_tab_action) },
        MenuItem { label: "-", shortcut: None, action: None },
        MenuItem { label: "Quit", shortcut: Some("Ctrl+Q"), action: Some(quit_action) },
      ],
    },
    Menu {
      label: "EDIT",
      items: vec![
        MenuItem { label: "Undo", shortcut: Some("Ctrl+Z"), action: None },
        MenuItem { label: "Redo", shortcut: Some("Ctrl+Shift+Z"), action: None },
        MenuItem { label: "Cut", shortcut: Some("Ctrl+X"), action: None },
        MenuItem { label: "Copy", shortcut: Some("Ctrl+C"), action: None },
        MenuItem { label: "Paste", shortcut: Some("Ctrl+V"), action: None },
      ],
    },
    Menu { label: "SELECTION", items: vec![MenuItem { label: "No actions available", shortcut: None, action: None }] },
    Menu { label: "VIEW", items: vec![MenuItem { label: "Toggle Status Bar", shortcut: None, action: None }] },
    Menu { label: "GO", items: vec![MenuItem { label: "No actions available", shortcut: None, action: None }] },
    Menu { label: "RUN", items: vec![MenuItem { label: "No actions available", shortcut: None, action: None }] },
    Menu {
      label: "TERMINAL",
      items: vec![
        MenuItem { label: "New Terminal", shortcut: None, action: Some(make_new_terminal_action(tabs_state)) },
        MenuItem { label: "Split Right", shortcut: None, action: Some(split_right_action) },
        MenuItem { label: "Split Down", shortcut: None, action: Some(split_down_action) },
      ],
    },
    Menu { label: "HELP", items: vec![MenuItem { label: "No actions available", shortcut: None, action: None }] },
  ]
}

#[component]
pub fn MenuBar() -> Element {
  let open_menu: Signal<Option<usize>> = use_signal(|| None);
  let tabs_state: Signal<TabsState<DesktopPane>> = use_context();
  let menus = build_menus(tabs_state);

  rsx! {
    div {
      class: "flex items-center h-10 bg-[var(--surface-container-lowest)] border-b-2 border-[var(--outline)] text-xs uppercase tracking-wider shrink-0 select-none",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")] dioxus::desktop::window().drag();
      },
      span {
        class: "px-3 font-bold text-[var(--primary)] font-[var(--font-display)]",
        onmousedown: |evt| evt.stop_propagation(),
        "TERMINAL_MONOLITH"
      }
      div {
        class: "flex items-center gap-0.5",
        onmousedown: |evt| evt.stop_propagation(),
        for (idx, menu) in menus.into_iter().enumerate() {
          {render_menu_dropdown(open_menu, idx, menu)}
        }
      }
      div { class: "flex-1" }
      div {
        class: "flex items-center",
        onmousedown: |evt| evt.stop_propagation(),
        button {
          class: "px-3 py-1 hover:bg-[var(--surface-container-high)] text-[var(--on-surface-variant)] transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")] dioxus::desktop::window().set_minimized(true);
          },
          span { class: "material-symbols-outlined text-sm", "remove" }
        }
        button {
          class: "px-3 py-1 hover:bg-[var(--surface-container-high)] text-[var(--on-surface-variant)] transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")] dioxus::desktop::window().toggle_maximized();
          },
          span { class: "material-symbols-outlined text-sm", "content_copy" }
        }
        button {
          class: "px-3 py-1 hover:bg-[var(--error)]/80 text-[var(--on-surface-variant)] hover:text-white transition-colors duration-150",
          onclick: move |_| {
              #[cfg(feature = "desktop")] dioxus::desktop::window().close();
          },
          span { class: "material-symbols-outlined text-sm", "close" }
        }
      }
    }
  }
}

fn render_menu_dropdown(open_menu: Signal<Option<usize>>, idx: usize, menu: Menu) -> Element {
  let label = menu.label;
  let items = menu.items;
  let is_run = label == "RUN";

  rsx! {
    div { class: "relative",
      span {
        class: if is_run { "px-2 py-1 rounded cursor-pointer text-[var(--primary)] hover:bg-[var(--surface-container-high)] transition-colors duration-150" } else { "px-2 py-1 rounded cursor-pointer text-[var(--on-surface-variant)] hover:text-[var(--primary)] hover:bg-[var(--surface-container-high)] transition-colors duration-150" },
        onclick: move |_| {
            let mut sig = open_menu;
            if sig() == Some(idx) {
                sig.set(None);
            } else {
                sig.set(Some(idx));
            }
        },
        "{label}"
      }
      if open_menu() == Some(idx) {
        div {
          class: "fixed inset-0 z-20",
          onclick: move |_| {
              let mut sig = open_menu;
              sig.set(None);
          },
        }
        div { class: "absolute top-full left-0 z-30 mt-1 py-1 bg-[var(--surface-container-high)]/80 backdrop-blur-[12px] rounded-md shadow-ambient min-w-48 normal-case tracking-normal",
          for item in items.iter() {
            if item.label == "-" {
              div { class: "my-1 border-t border-[var(--outline-variant)]" }
            } else {
              {render_menu_item(open_menu, item)}
            }
          }
        }
      }
    }
  }
}

fn render_menu_item(open_menu: Signal<Option<usize>>, item: &MenuItem) -> Element {
  let label = item.label;
  let shortcut = item.shortcut;
  let action = item.action;
  let has_action = action.is_some();

  rsx! {
    button {
      class: "w-full flex items-center justify-between px-3 py-1.5 text-sm transition-colors duration-100",
      class: if has_action { "text-[var(--on-surface)] hover:bg-[var(--surface-container-highest)]" } else { "text-[var(--on-surface-variant)] opacity-50" },
      disabled: !has_action,
      onclick: move |_| {
          if let Some(ref action) = action {
              action.call(());
              let mut sig = open_menu;
              sig.set(None);
          }
      },
      span { "{label}" }
      if let Some(shortcut) = shortcut {
        span { class: "ml-6 text-[var(--on-surface-variant)] text-[10px]", "{shortcut}" }
      }
    }
  }
}
