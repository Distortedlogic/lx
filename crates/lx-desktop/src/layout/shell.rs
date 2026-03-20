use std::sync::Arc;

use dioxus::prelude::*;
use lx_dx::event::EventBus;

use super::sidebar::Sidebar;
use crate::routes::Route;

#[component]
pub fn Shell() -> Element {
    let bus = use_context_provider(|| Signal::new(Arc::new(EventBus::new())));
    let _bus_ref = bus.read().clone();
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
