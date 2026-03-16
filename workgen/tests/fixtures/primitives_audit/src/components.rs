use dioxus::prelude::*;

#[component]
fn ConfirmDialog(show: Signal<bool>, on_confirm: EventHandler<()>, message: String) -> Element {
    if !show() { return rsx! {} }
    rsx! {
        div {
            class: "fixed inset-0 z-50 bg-black/50",
            onclick: move |_| show.set(false),
            div {
                class: "absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-card p-6 rounded-lg",
                p { "{message}" }
                button { onclick: move |_| on_confirm.call(()), "Confirm" }
                button { onclick: move |_| show.set(false), "Cancel" }
            }
        }
    }
}

#[component]
fn CustomTooltip(text: String, children: Element) -> Element {
    let show = use_signal(|| false);
    rsx! {
        div {
            onmouseenter: move |_| show.set(true),
            onmouseleave: move |_| show.set(false),
            {children}
            if show() {
                div {
                    class: "absolute top-full mt-1 bg-popover text-popover-foreground text-sm px-2 py-1 rounded",
                    "{text}"
                }
            }
        }
    }
}

#[component]
fn CustomSelect(options: Vec<String>, selected: Signal<String>) -> Element {
    let open = use_signal(|| false);
    rsx! {
        div {
            button { onclick: move |_| open.toggle(), "{selected}" }
            if open() {
                div {
                    class: "absolute bg-popover border rounded mt-1",
                    for option in options {
                        div {
                            onclick: move |_| { selected.set(option.clone()); open.set(false); },
                            "{option}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CustomTabs(tabs: Vec<String>, active: Signal<usize>, children: Element) -> Element {
    rsx! {
        div {
            div { class: "flex border-b",
                for (i, tab) in tabs.iter().enumerate() {
                    button {
                        class: if active() == i { "border-b-2 border-primary font-bold" } else { "" },
                        onclick: move |_| active.set(i),
                        "{tab}"
                    }
                }
            }
            {children}
        }
    }
}
