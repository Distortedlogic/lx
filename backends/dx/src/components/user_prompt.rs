use dioxus::prelude::*;

use crate::event::UserPromptKind;

#[derive(Props, Clone, PartialEq)]
pub struct UserPromptProps {
    pub prompt_id: u64,
    pub kind: UserPromptKind,
    pub on_respond: EventHandler<serde_json::Value>,
}

#[component]
pub fn UserPromptWidget(props: UserPromptProps) -> Element {
    let default_text = match &props.kind {
        UserPromptKind::Ask { default, .. } => default.clone().unwrap_or_default(),
        _ => String::new(),
    };
    let mut input_val: Signal<String> = use_signal(|| default_text);

    match &props.kind {
        UserPromptKind::Confirm { message } => {
            let msg = message.clone();
            let on_yes = props.on_respond.clone();
            let on_no = props.on_respond.clone();
            rsx! {
                div {
                    class: "user-prompt prompt-confirm",
                    p { "{msg}" }
                    div {
                        class: "prompt-buttons",
                        button {
                            class: "btn btn-yes",
                            onclick: move |_| on_yes.call(serde_json::Value::Bool(true)),
                            "Yes"
                        }
                        button {
                            class: "btn btn-no",
                            onclick: move |_| on_no.call(serde_json::Value::Bool(false)),
                            "No"
                        }
                    }
                }
            }
        }
        UserPromptKind::Choose { message, options } => {
            let msg = message.clone();
            let opts = options.clone();
            rsx! {
                div {
                    class: "user-prompt prompt-choose",
                    p { "{msg}" }
                    div {
                        class: "prompt-options",
                        for (i, opt) in opts.iter().enumerate() {
                            {
                                let handler = props.on_respond.clone();
                                let idx = i;
                                rsx! {
                                    button {
                                        class: "btn btn-option",
                                        onclick: move |_| {
                                            handler.call(serde_json::Value::Number(
                                                serde_json::Number::from(idx as u64),
                                            ))
                                        },
                                        "{opt}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        UserPromptKind::Ask { message, default } => {
            let msg = message.clone();
            let placeholder = default.clone().unwrap_or_default();
            let on_respond = props.on_respond.clone();
            rsx! {
                div {
                    class: "user-prompt prompt-ask",
                    p { "{msg}" }
                    div {
                        class: "prompt-input",
                        input {
                            r#type: "text",
                            value: "{input_val}",
                            placeholder: "{placeholder}",
                            oninput: move |evt| input_val.set(evt.value().clone()),
                        }
                        button {
                            class: "btn btn-submit",
                            onclick: move |_| {
                                let val = input_val.read().clone();
                                on_respond.call(serde_json::Value::String(val));
                            },
                            "Submit"
                        }
                    }
                }
            }
        }
    }
}

pub fn render_prompt_inline(kind: &UserPromptKind, key: usize) -> Element {
    match kind {
        UserPromptKind::Confirm { message } => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-prompt",
                    span { class: "prompt-tag", "[CONFIRM]" }
                    span { " {message}" }
                }
            }
        }
        UserPromptKind::Choose {
            message, options, ..
        } => {
            let opts_str = options.join(", ");
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-prompt",
                    span { class: "prompt-tag", "[CHOOSE]" }
                    span { " {message}: {opts_str}" }
                }
            }
        }
        UserPromptKind::Ask { message, .. } => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-prompt",
                    span { class: "prompt-tag", "[ASK]" }
                    span { " {message}" }
                }
            }
        }
    }
}
