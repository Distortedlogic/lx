use dioxus::prelude::*;

#[component]
fn Card(title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "bg-slate-800 rounded-sm border p-4",
            dark: "dark:bg-slate-900",
            h2 { class: "text-gray-300 font-bold", "{title}" }
            {children}
        }
    }
}

#[component]
fn StatusBadge(status: String) -> Element {
    let color = if status == "active" { "bg-green-500" } else { "bg-red-500" };
    rsx! {
        span {
            class: "{color} text-white px-2 py-1 rounded",
            "{status}"
        }
    }
}

#[component]
fn Input(value: String, on_change: EventHandler<String>) -> Element {
    rsx! {
        input {
            class: "border outline-none focus:ring focus:border-blue-500",
            value: "{value}",
        }
    }
}
