use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn App() -> Element {
    rsx! {
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
        document::Script { src: asset!("/assets/echarts-5.5.1.min.js") }
        document::Script { src: asset!("/assets/js/formatters.js") }
        document::Script { src: asset!("/assets/js/chart_init.js") }
        document::Script { src: asset!("/assets/js/flamegraph.js") }
        document::Script { src: asset!("/assets/js/flow_graph.js") }
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
