use dioxus::prelude::*;

use crate::routes::Route;

static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css", AssetOptions::css().with_static_head(true));
static _ECHARTS_JS: Asset = asset!("/assets/echarts-5.5.1.min.js", AssetOptions::js().with_static_head(true));
static _WIDGET_BRIDGE_JS: Asset = asset!("/assets/widget-bridge.js", AssetOptions::js().with_static_head(true));

#[component]
pub fn App() -> Element {
  #[cfg(feature = "desktop")]
  use_hook(|| {
    let desktop = dioxus::desktop::window();
    crate::webview_permissions::enable_media_permissions(&desktop);
  });

  rsx! {
    document::Link {
      rel: "stylesheet",
      href: "https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@300;400;500;600;700&family=Inter:wght@300;400;500;600;700&family=JetBrains+Mono:wght@400;500&family=Material+Symbols+Outlined:opsz,wght,FILL,GRAD@24,400,0,0&display=swap",
    }
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
        Router::<Route> {}
      }
    }
  }
}
