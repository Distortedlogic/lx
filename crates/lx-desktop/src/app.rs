use dioxus::prelude::*;

use crate::routes::Route;

static TAILWIND_CSS: Asset = asset!("/assets/tailwind.css", AssetOptions::css().with_static_head(true));
static _ECHARTS_JS: Asset = asset!("/assets/echarts-5.5.1.min.js", AssetOptions::js().with_static_head(true));
static _DX_CHARTS_JS: Asset = asset!("/assets/dx-charts.js", AssetOptions::js().with_static_head(true));
static _WIDGET_BRIDGE_JS: Asset = asset!("/assets/widget-bridge.js", AssetOptions::js().with_static_head(true));

#[component]
pub fn App() -> Element {
  rsx! {
    document::Stylesheet { href: TAILWIND_CSS }
    document::Style {
      r#"
            :root {{
                --foreground: #e5e7eb;
                --color-chart-axis: #404040;
                --color-chart-split: #333333;
                --color-chart-tooltip: #171717;
            }}
            "#
    }
    ErrorBoundary {
      handle_error: |errors: ErrorContext| {
          let msg = errors
              .error()
              .map_or_else(|| "An unknown error occurred".to_owned(), |e| e.to_string());
          rsx! {
            div { class: "flex items-center justify-center h-screen text-red-500", "{msg}" }
          }
      },
      SuspenseBoundary {
        fallback: |_| rsx! {
          div { class: "flex items-center justify-center h-screen text-gray-500", "Loading..." }
        },
        Router::<Route> {}
      }
    }
  }
}
