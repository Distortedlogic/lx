use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use pane_tree::{PaneNode, TabsState};
use tokio::sync::mpsc;

use super::menu_bar::MenuBar;
use super::sidebar::Sidebar;
use super::status_bar::StatusBar;
use crate::panes::DesktopPane;
use crate::routes::Route;
use crate::terminal::{add_tab, use_provide_tabs};

#[cfg(feature = "desktop")]
const ECHARTS_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/echarts-5.5.1.min.js"));
#[cfg(feature = "desktop")]
const DX_CHARTS_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/dx-charts.js"));
#[cfg(feature = "desktop")]
const WIDGET_BRIDGE_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/widget-bridge.js"));

pub struct TerminalSpawnRequest {
  pub id: String,
  pub title: String,
  pub pane: PaneNode<DesktopPane>,
}

type SpawnReceiver = mpsc::UnboundedReceiver<TerminalSpawnRequest>;

#[component]
pub fn Shell() -> Element {
  let tabs_state = use_provide_tabs();
  #[cfg(feature = "desktop")]
  use_hook(|| {
    document::eval(ECHARTS_JS);
    document::eval(DX_CHARTS_JS);
    document::eval(WIDGET_BRIDGE_JS);
  });
  let spawn_channel = use_hook(|| {
    let (tx, rx) = mpsc::unbounded_channel::<TerminalSpawnRequest>();
    (tx, Arc::new(Mutex::new(Some(rx))))
  });
  use_context_provider(|| spawn_channel.0.clone());
  spawn_terminal_listener(tabs_state, &spawn_channel.1);
  rsx! {
    div { class: "relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col",
      ResizeHandles {}
      MenuBar {}
      div { class: "flex flex-1 min-h-0",
        Sidebar {}
        main { class: "flex-1 flex flex-col p-0 min-h-0",
          div { class: "flex-1 min-h-0 overflow-auto",
            ErrorBoundary {
              handle_error: |errors: ErrorContext| {
                  let msg = errors
                      .error()
                      .map_or_else(|| "Page error".to_owned(), |e| e.to_string());
                  rsx! {
                    div { class: "p-4 text-[var(--error)]", "{msg}" }
                  }
              },
              SuspenseBoundary {
                fallback: |_| rsx! {
                  div { class: "flex items-center justify-center h-full text-[var(--outline)]", "Loading..." }
                },
                Outlet::<Route> {}
              }
            }
          }
        }
      }
      StatusBar {}
    }
  }
}

#[component]
fn ResizeHandles() -> Element {
  rsx! {
    div {
      class: "absolute top-0 left-2 right-2 h-1 z-50",
      style: "cursor: ns-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(
                      dioxus::desktop::tao::window::ResizeDirection::North,
                  );
          }
      },
    }
    div {
      class: "absolute bottom-0 left-2 right-2 h-1 z-50",
      style: "cursor: ns-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(
                      dioxus::desktop::tao::window::ResizeDirection::South,
                  );
          }
      },
    }
    div {
      class: "absolute left-0 top-2 bottom-2 w-1 z-50",
      style: "cursor: ew-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::West);
          }
      },
    }
    div {
      class: "absolute right-0 top-2 bottom-2 w-1 z-50",
      style: "cursor: ew-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::East);
          }
      },
    }
    div {
      class: "absolute top-0 left-0 w-2 h-2 z-50",
      style: "cursor: nwse-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(
                      dioxus::desktop::tao::window::ResizeDirection::NorthWest,
                  );
          }
      },
    }
    div {
      class: "absolute top-0 right-0 w-2 h-2 z-50",
      style: "cursor: nesw-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(
                      dioxus::desktop::tao::window::ResizeDirection::NorthEast,
                  );
          }
      },
    }
    div {
      class: "absolute bottom-0 left-0 w-2 h-2 z-50",
      style: "cursor: nesw-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(
                      dioxus::desktop::tao::window::ResizeDirection::SouthWest,
                  );
          }
      },
    }
    div {
      class: "absolute bottom-0 right-0 w-2 h-2 z-50",
      style: "cursor: nwse-resize;",
      onmousedown: move |_| {
          #[cfg(feature = "desktop")]
          {
              let _ = dioxus::desktop::window()
                  .drag_resize_window(
                      dioxus::desktop::tao::window::ResizeDirection::SouthEast,
                  );
          }
      },
    }
  }
}

fn spawn_terminal_listener(tabs_state: Signal<TabsState<DesktopPane>>, rx_holder: &Arc<Mutex<Option<SpawnReceiver>>>) {
  let rx_holder = rx_holder.clone();
  use_hook(move || {
    if let Some(mut rx) = rx_holder.lock().expect("lock poisoned").take() {
      spawn(async move {
        while let Some(req) = rx.recv().await {
          add_tab(tabs_state, req.id, req.title, req.pane);
        }
      });
    }
  });
}
