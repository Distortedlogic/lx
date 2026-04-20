use dioxus::prelude::*;

use crate::routes::Route;
use crate::runtime::DesktopRuntimeProvider;

static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css", AssetOptions::css().with_static_head(true));
static FONTS_CSS: Asset = asset!("/assets/fonts.css", AssetOptions::css().with_static_head(true));
#[used]
static _FONTS_DIR: Asset = asset!("/assets/fonts", AssetOptions::folder());
static _ECHARTS_JS: Asset = asset!("/assets/echarts-5.5.1.min.js", AssetOptions::js().with_static_head(true));
static _WIDGET_BRIDGE_JS: Asset = asset!("/assets/widget-bridge.js", AssetOptions::js().with_static_head(true));

#[component]
pub fn App() -> Element {
  use_context_provider(|| {
    let credentials = crate::credentials::CredentialStore::file_backed().ok().unwrap_or_else(crate::credentials::CredentialStore::in_memory);
    let runs = crate::engine::FlowRunPersistence::file_backed();
    let flow_persistence = crate::pages::flows::storage::FlowPersistence::file_backed();
    crate::engine::build_and_start_scheduler(credentials, runs, &flow_persistence)
  });
  rsx! {
    document::Stylesheet { href: FONTS_CSS }
    document::Stylesheet { href: TAILWIND_CSS }
    ErrorBoundary {
      handle_error: |errors: ErrorContext| {
          let msg = errors
              .error()
              .map_or_else(|| "An unknown error occurred".to_owned(), |e| e.to_string());
          rsx! {
            div { class: "flex items-center justify-center h-screen text-[var(--error)]", "{msg}" }
          }
      },
      SuspenseBoundary {
        fallback: |_| rsx! {
          div { class: "flex items-center justify-center h-screen p-6",
            crate::components::page_skeleton::PageSkeleton {}
          }
        },
        DesktopRuntimeProvider { Router::<Route> {} }
      }
    }
  }
}
