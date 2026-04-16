use dioxus::prelude::*;

use crate::runtime::use_desktop_runtime;

#[component]
pub fn PiInput(agent_id: String) -> Element {
  let runtime = use_desktop_runtime();
  let mut text = use_signal(String::new);

  let send_prompt = {
    let agent_id = agent_id.clone();
    let runtime = runtime.clone();
    move |_| {
      let value = text.read().trim().to_string();
      if value.is_empty() {
        return;
      }
      runtime.prompt(agent_id.clone(), value);
      text.set(String::new());
    }
  };

  let send_steer = {
    let agent_id = agent_id.clone();
    let runtime = runtime.clone();
    move |_| {
      let value = text.read().trim().to_string();
      if value.is_empty() {
        return;
      }
      runtime.steer(agent_id.clone(), value);
      text.set(String::new());
    }
  };

  let send_follow_up = {
    let agent_id = agent_id.clone();
    let runtime = runtime.clone();
    move |_| {
      let value = text.read().trim().to_string();
      if value.is_empty() {
        return;
      }
      runtime.follow_up(agent_id.clone(), value);
      text.set(String::new());
    }
  };

  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-low)] p-3",
      textarea {
        class: "min-h-28 w-full rounded-xl border border-[var(--outline-variant)]/30 bg-[var(--surface)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
        value: "{text}",
        placeholder: "Send prompt, steer, or follow-up",
        oninput: move |event| text.set(event.value()),
      }
      div { class: "mt-3 flex flex-wrap gap-2",
        button { class: "btn-outline-sm", onclick: send_prompt, "Prompt" }
        button { class: "btn-outline-sm", onclick: send_steer, "Steer" }
        button { class: "btn-outline-sm", onclick: send_follow_up, "Follow-up" }
        button {
          class: "btn-outline-sm border-red-500/30 text-red-300 hover:bg-red-500/10",
          onclick: {
              let runtime = runtime.clone();
              move |_| runtime.abort(agent_id.clone())
          },
          "Abort"
        }
      }
    }
  }
}
