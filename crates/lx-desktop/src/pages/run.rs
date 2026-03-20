use dioxus::prelude::*;

#[component]
pub fn Run() -> Element {
    rsx! {
        div { class: "p-4",
            h2 { class: "text-xl font-bold mb-4", "Run" }
            p { class: "text-gray-400", "Select a .lx file to execute." }
        }
    }
}
