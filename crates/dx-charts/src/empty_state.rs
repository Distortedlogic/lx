use dioxus::prelude::*;

#[component]
pub fn EmptyState(message: String) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center h-full text-gray-500 text-sm",
            "{message}"
        }
    }
}
