use dioxus::prelude::*;

#[component]
pub fn Terminals() -> Element {
    rsx! {
        div { class: "p-4",
            h2 { class: "text-xl font-bold mb-4", "Terminals" }
            p { class: "text-gray-400", "Terminal pane manager." }
        }
    }
}
