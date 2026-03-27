use dioxus::prelude::*;
use lx_api::run_api::{get_prompts, post_respond};
use lx_api::types::{PendingPrompt, PromptResponse};

#[component]
pub fn Approvals() -> Element {
  let mut action = use_action(get_prompts);

  use_future(move || async move {
    loop {
      action.call();
      tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
  });

  let prompts: Vec<PendingPrompt> = action.value().and_then(|r| r.ok()).map(|s| s.read().clone()).unwrap_or_default();

  rsx! {
    div { class: "space-y-4",
      h2 { class: "text-lg font-bold", "Approvals" }
      if prompts.is_empty() {
        p { class: "text-[var(--outline)] text-sm", "No pending approvals" }
      }
      for prompt in prompts.iter() {
        {render_prompt(prompt)}
      }
    }
  }
}

fn send_response(prompt_id: u64, value: serde_json::Value) {
  spawn(async move {
    let _ = post_respond(PromptResponse { prompt_id, response: value }).await;
  });
}

fn render_prompt(prompt: &PendingPrompt) -> Element {
  match prompt.kind.as_str() {
    "confirm" => {
      let pid = prompt.prompt_id;
      let message = prompt.message.clone();
      rsx! {
        div { class: "p-3 bg-[var(--surface-container)] rounded space-y-2",
          p { class: "text-sm", "{message}" }
          div { class: "flex gap-2",
            button {
              class: "px-3 py-1 bg-[var(--success)] rounded text-sm",
              onclick: move |_| send_response(pid, serde_json::json!(true)),
              "Yes"
            }
            button {
              class: "px-3 py-1 bg-[var(--error)] rounded text-sm",
              onclick: move |_| send_response(pid, serde_json::json!(false)),
              "No"
            }
          }
        }
      }
    },
    "choose" => {
      let pid = prompt.prompt_id;
      let message = prompt.message.clone();
      let options = prompt.options.clone().unwrap_or_default();
      rsx! {
        div { class: "p-3 bg-[var(--surface-container)] rounded space-y-2",
          p { class: "text-sm", "{message}" }
          for (i , opt) in options.iter().enumerate() {
            button {
              class: "block w-full text-left px-3 py-1 bg-[var(--surface-container-high)] rounded text-sm hover:bg-[var(--surface-bright)]",
              onclick: move |_| send_response(pid, serde_json::json!(i)),
              "{opt}"
            }
          }
        }
      }
    },
    "ask" => {
      let pid = prompt.prompt_id;
      let message = prompt.message.clone();
      let mut input_text = use_signal(String::new);
      rsx! {
        div { class: "p-3 bg-[var(--surface-container)] rounded space-y-2",
          p { class: "text-sm", "{message}" }
          div { class: "flex gap-2",
            input {
              r#type: "text",
              class: "flex-1 bg-[var(--surface-container-high)] border border-[var(--outline)] rounded px-2 py-1 text-sm text-[var(--on-surface)]",
              placeholder: "Type your response...",
              value: "{input_text}",
              oninput: move |e| input_text.set(e.value()),
            }
            button {
              class: "px-3 py-1 bg-[var(--primary)] rounded text-sm",
              disabled: input_text.read().is_empty(),
              onclick: move |_| {
                  let val = input_text.read().clone();
                  send_response(pid, serde_json::json!(val));
                  input_text.set(String::new());
              },
              "Send"
            }
          }
        }
      }
    },
    _ => rsx! {},
  }
}
