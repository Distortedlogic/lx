use dioxus::prelude::*;

use crate::api_client::LxClient;

#[component]
pub fn Approvals() -> Element {
  let client: Signal<LxClient> = use_context();
  let mut prompts: Signal<Vec<PendingPrompt>> = use_signal(Vec::new);

  use_future(move || async move {
    loop {
      let c = client.read();
      if let Ok(raw) = c.fetch_pending_prompts().await {
        let parsed: Vec<PendingPrompt> = raw.into_iter().filter_map(|v| parse_pending_prompt(&v)).collect();
        prompts.set(parsed);
      }
      drop(c);
      tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
  });

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
  Confirm { message: String },
  Choose { message: String, options: Vec<String> },
  Ask { message: String },
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
    },
    PromptKind::Choose { message, options } => {
      let pid = prompt.prompt_id;
      rsx! {
        div { class: "p-3 bg-gray-800 rounded space-y-2",
          p { class: "text-sm", "{message}" }
          for (i , opt) in options.iter().enumerate() {
            button {
              class: "block w-full text-left px-3 py-1 bg-gray-700 rounded text-sm hover:bg-gray-600",
              onclick: move |_| send_response(client, pid, serde_json::json!(i)),
              "{opt}"
            }
          }
        }
      }
    },
    PromptKind::Ask { message } => {
      let pid = prompt.prompt_id;
      let mut input_text = use_signal(String::new);
      rsx! {
        div { class: "p-3 bg-gray-800 rounded space-y-2",
          p { class: "text-sm", "{message}" }
          div { class: "flex gap-2",
            input {
              r#type: "text",
              class: "flex-1 bg-gray-700 border border-gray-600 rounded px-2 py-1 text-sm text-gray-100",
              placeholder: "Type your response...",
              value: "{input_text}",
              oninput: move |e| input_text.set(e.value()),
            }
            button {
              class: "px-3 py-1 bg-blue-600 rounded text-sm",
              disabled: input_text.read().is_empty(),
              onclick: move |_| {
                  let val = input_text.read().clone();
                  send_response(client, pid, serde_json::json!(val));
                  input_text.set(String::new());
              },
              "Send"
            }
          }
        }
      }
    },
  }
}

fn parse_pending_prompt(v: &serde_json::Value) -> Option<PendingPrompt> {
  let prompt_id = v.get("prompt_id")?.as_u64()?;
  let kind_str = v.get("kind")?.as_str()?;
  let message = v.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
  let kind = match kind_str {
    "confirm" => PromptKind::Confirm { message },
    "choose" => {
      let options =
        v.get("options").and_then(|o| o.as_array()).map(|arr| arr.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
      PromptKind::Choose { message, options }
    },
    "ask" => PromptKind::Ask { message },
    _ => return None,
  };
  Some(PendingPrompt { prompt_id, kind })
}
