use dioxus::prelude::*;
use lx_api::run_api::{get_prompts, post_respond};
use lx_api::types::{PromptKind, PromptResponse};

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
        match prompt.kind {
            PromptKind::Confirm => rsx! {
              ConfirmPrompt { prompt_id: prompt.prompt_id, message: prompt.message.clone() }
            },
            PromptKind::Choose => rsx! {
              ChoosePrompt {
                prompt_id: prompt.prompt_id,
                message: prompt.message.clone(),
                options: prompt.options.clone().unwrap_or_default(),
              }
            },
            PromptKind::Ask => rsx! {
              AskPrompt { prompt_id: prompt.prompt_id, message: prompt.message.clone() }
            },
        }
      }
    }
  }
}

#[component]
fn ConfirmPrompt(prompt_id: u64, message: String) -> Element {
  let mut respond = use_action(post_respond);
  rsx! {
    div { class: "p-3 bg-[var(--surface-container)] rounded space-y-2",
      p { class: "text-sm", "{message}" }
      div { class: "flex gap-2",
        button {
          class: "px-3 py-1 bg-[var(--success)] rounded text-sm",
          onclick: move |_| {
              respond
                  .call(PromptResponse {
                      prompt_id,
                      response: serde_json::json!(true),
                  });
          },
          "Yes"
        }
        button {
          class: "px-3 py-1 bg-[var(--error)] rounded text-sm",
          onclick: move |_| {
              respond
                  .call(PromptResponse {
                      prompt_id,
                      response: serde_json::json!(false),
                  });
          },
          "No"
        }
      }
    }
  }
}

#[component]
fn ChoosePrompt(prompt_id: u64, message: String, options: Vec<String>) -> Element {
  let mut respond = use_action(post_respond);
  rsx! {
    div { class: "p-3 bg-[var(--surface-container)] rounded space-y-2",
      p { class: "text-sm", "{message}" }
      for (i, opt) in options.iter().enumerate() {
        button {
          class: "block w-full text-left px-3 py-1 bg-[var(--surface-container-high)] rounded text-sm hover:bg-[var(--surface-bright)]",
          onclick: move |_| {
              respond
                  .call(PromptResponse {
                      prompt_id,
                      response: serde_json::json!(i),
                  });
          },
          "{opt}"
        }
      }
    }
  }
}

#[component]
fn AskPrompt(prompt_id: u64, message: String) -> Element {
  let mut respond = use_action(post_respond);
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
          disabled: input_text().is_empty(),
          onclick: move |_| {
              let val = input_text();
              respond
                  .call(PromptResponse {
                      prompt_id,
                      response: serde_json::json!(val),
                  });
              input_text.set(String::new());
          },
          "Send"
        }
      }
    }
  }
}
