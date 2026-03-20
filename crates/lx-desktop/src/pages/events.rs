use dioxus::prelude::*;

#[component]
pub fn Events() -> Element {
    rsx! {
        div { class: "p-4",
            h2 { class: "text-xl font-bold mb-4", "Events" }
            p { class: "text-gray-400", "Runtime event log." }
        }
    }
}
