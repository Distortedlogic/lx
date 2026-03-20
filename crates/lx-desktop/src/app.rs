use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn App() -> Element {
    rsx! {
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
