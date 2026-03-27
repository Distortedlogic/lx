use dioxus::prelude::*;
use lx_api::run_api::{get_prompts, post_respond};
use lx_api::types::{PendingPrompt, PromptResponse};

#[component]
pub fn Approvals() -> Element {
  let prompts = use_loader(get_prompts)?;

  let prompts_ref = prompts.read();
  rsx! {
    div { class: "space-y-4",
      h2 { class: "text-lg font-bold", "Approvals" }
      if prompts_ref.is_empty() {
        p { class: "text-[var(--outline)] text-sm", "No pending approvals" }
      }
      for prompt in prompts_ref.iter() {
        {render_prompt(prompt)}
      }
    }
  }
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
              onclick: move |_| {
                  spawn(async move {
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(true),
                          })
                          .await;
                  });
              },
              "Yes"
            }
            button {
              class: "px-3 py-1 bg-[var(--error)] rounded text-sm",
              onclick: move |_| {
                  spawn(async move {
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(false),
                          })
                          .await;
                  });
              },
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
              onclick: move |_| {
                  spawn(async move {
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(i),
                          })
                          .await;
                  });
              },
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
                  spawn(async move {
                      let _ = post_respond(PromptResponse {
                              prompt_id: pid,
                              response: serde_json::json!(val),
                          })
                          .await;
                  });
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
