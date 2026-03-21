use std::sync::{Arc, Mutex};

use dioxus::prelude::*;
use lx_dx::event::EventBus;
use lx_ui::pane_tree::PaneNode;
use lx_ui::tab_state::TabsState;
use tokio::sync::mpsc;

use super::sidebar::Sidebar;
use crate::routes::Route;
use crate::terminal::{add_tab, use_provide_tabs};

pub struct TerminalSpawnRequest {
  pub id: String,
  pub title: String,
  pub pane: PaneNode,
}

pub type SpawnSender = mpsc::UnboundedSender<TerminalSpawnRequest>;
type SpawnReceiver = mpsc::UnboundedReceiver<TerminalSpawnRequest>;

#[component]
pub fn Shell() -> Element {
  let bus = use_context_provider(|| Signal::new(Arc::new(EventBus::new())));
  let _bus_ref = bus.read().clone();
  let tabs_state = use_provide_tabs();
  let spawn_channel = use_hook(|| {
    let (tx, rx) = mpsc::unbounded_channel::<TerminalSpawnRequest>();
    (tx, Arc::new(Mutex::new(Some(rx))))
  });
  use_context_provider(|| spawn_channel.0.clone());
  spawn_terminal_listener(tabs_state, &spawn_channel.1);
  let collapsed = use_signal(|| false);
  rsx! {
      div { class: "min-h-screen bg-gray-900 text-gray-100 flex",
          Sidebar { collapsed }
          main { class: "flex-1 flex flex-col p-4 min-h-0",
              div { class: "flex-1 min-h-0 overflow-auto",
                  ErrorBoundary {
                      handle_error: |errors: ErrorContext| {
                          let msg = errors
                              .error()
                              .map_or_else(|| "Page error".to_owned(), |e| e.to_string());
                          rsx! {
                              div { class: "p-4 text-red-400", "{msg}" }
                          }
                      },
                      SuspenseBoundary {
                          fallback: |_| rsx! {
                              div { class: "flex items-center justify-center h-full text-gray-500", "Loading..." }
                          },
                          Outlet::<Route> {}
                      }
                  }
              }
          }
      }
  }
}

fn spawn_terminal_listener(tabs_state: Signal<TabsState>, rx_holder: &Arc<Mutex<Option<SpawnReceiver>>>) {
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
