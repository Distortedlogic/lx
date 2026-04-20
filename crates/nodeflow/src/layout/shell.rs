use dioxus::prelude::*;

use super::breadcrumb_bar::BreadcrumbBar;
use super::properties_panel::PropertiesPanel;
use super::status_bar::StatusBar;
use crate::components::toast_viewport::ToastViewport;
use crate::contexts::activity_log::ActivityLog;
use crate::contexts::live_updates::LiveUpdatesProvider;
use crate::contexts::status_bar::StatusBarState;
use crate::hooks::keyboard_shortcuts::{ShortcutRegistry, use_keyboard_shortcuts};
use crate::pages::flows::FlowRouteScope;
use crate::routes::Route;

#[component]
pub fn Shell() -> Element {
  let status_bar_state =
    use_store(|| StatusBarState { branch: "main".into(), line: 1, col: 1, encoding: "UTF-8".into(), notification_count: 0, pane_label: "READY".into() });
  use_context_provider(|| status_bar_state);
  let _activity_log = ActivityLog::provide();
  let _theme = crate::contexts::theme::ThemeState::provide();
  let _toast = crate::contexts::toast::ToastState::provide();
  let _dialog = crate::contexts::dialog::DialogState::provide();
  let _panel = crate::contexts::panel::PanelState::provide();
  let _sidebar_ctx = crate::contexts::sidebar::SidebarState::provide();
  let _breadcrumb = crate::contexts::breadcrumb::BreadcrumbState::provide();
  let _shortcut_registry = ShortcutRegistry::provide();
  let (_registry, key_handler) = use_keyboard_shortcuts();
  let route: Route = use_route();

  rsx! {
    div {
      class: "relative h-screen overflow-hidden bg-[var(--surface)] text-[var(--on-surface)] flex flex-col",
      tabindex: "0",
      onkeydown: move |e| key_handler.call(e),
      ResizeHandles {}
      div { class: "flex flex-1 min-h-0",
        div { class: "flex min-w-0 flex-col flex-1 h-full",
          BreadcrumbBar {}
          match route.clone() {
              Route::Flows {} => rsx! {
                FlowRouteScope { flow_id: None,
                  div { class: "flex flex-1 min-h-0",
                    ShellPageOutlet {}
                    PropertiesPanel {}
                  }
                }
              },
              Route::FlowDetail { flow_id } => rsx! {
                FlowRouteScope { key: "{flow_id}", flow_id: Some(flow_id),
                  div { class: "flex flex-1 min-h-0",
                    ShellPageOutlet {}
                    PropertiesPanel {}
                  }
                }
              },
              _ => rsx! {
                div { class: "flex flex-1 min-h-0",
                  ShellPageOutlet {}
                  PropertiesPanel {}
                }
              },
          }
        }
      }
      StatusBar {}
      ToastViewport {}
    }
  }
}

#[component]
fn ShellPageOutlet() -> Element {
  rsx! {
    main { class: "flex-1 flex flex-col p-0 min-h-0",
      div { class: "flex-1 min-h-0 overflow-auto p-6",
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
              div { class: "p-6", crate::components::page_skeleton::PageSkeleton {} }
            },
            LiveUpdatesProvider {}
          }
        }
      }
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
                  .drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::North);
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
                  .drag_resize_window(dioxus::desktop::tao::window::ResizeDirection::South);
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
  }
}
