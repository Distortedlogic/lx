use dioxus::prelude::*;

use crate::api_client::LxClient;

#[component]
pub fn Approvals() -> Element {
    let client: Signal<LxClient> = use_context();
    let prompts: Signal<Vec<PendingPrompt>> = use_signal(Vec::new);

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-lg font-bold", "Approvals" }
            if prompts.read().is_empty() {
                p { class: "text-gray-500 text-sm", "No pending approvals" }
            }
            for prompt in prompts.read().iter() {
                {render_prompt(prompt, client)}
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct PendingPrompt {
    pub prompt_id: u64,
    pub kind: PromptKind,
}

#[derive(Clone, Debug)]
pub enum PromptKind {
    Confirm {
        message: String,
    },
    Choose {
        message: String,
        options: Vec<String>,
    },
    Ask {
        message: String,
    },
}

fn send_response(client: Signal<LxClient>, pid: u64, value: serde_json::Value) {
    let base = client.read().base_url_for_spawn();
    tokio::spawn(async move {
        let c = LxClient::new(&base);
        let _ = c.post_user_response(pid, value).await;
    });
}

fn render_prompt(prompt: &PendingPrompt, client: Signal<LxClient>) -> Element {
    match &prompt.kind {
        PromptKind::Confirm { message } => {
            let pid = prompt.prompt_id;
            rsx! {
                div { class: "p-3 bg-gray-800 rounded space-y-2",
                    p { class: "text-sm", "{message}" }
                    div { class: "flex gap-2",
                        button {
                            class: "px-3 py-1 bg-green-600 rounded text-sm",
                            onclick: move |_| send_response(client, pid, serde_json::json!(true)),
                            "Yes"
                        }
                        button {
                            class: "px-3 py-1 bg-red-600 rounded text-sm",
                            onclick: move |_| send_response(client, pid, serde_json::json!(false)),
                            "No"
                        }
                    }
                }
            }
        }
        PromptKind::Choose { message, options } => {
            let pid = prompt.prompt_id;
            rsx! {
                div { class: "p-3 bg-gray-800 rounded space-y-2",
                    p { class: "text-sm", "{message}" }
                    for (i, opt) in options.iter().enumerate() {
                        button {
                            class: "block w-full text-left px-3 py-1 bg-gray-700 rounded text-sm hover:bg-gray-600",
                            onclick: move |_| send_response(client, pid, serde_json::json!(i)),
                            "{opt}"
                        }
                    }
                }
            }
        }
        PromptKind::Ask { message } => {
            rsx! {
                div { class: "p-3 bg-gray-800 rounded space-y-2",
                    p { class: "text-sm", "{message}" }
                    p { class: "text-xs text-gray-500", "Text input pending" }
                }
            }
        }
    }
}
