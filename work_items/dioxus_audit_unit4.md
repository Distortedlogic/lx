# Unit 4: approvals.rs — spawn in event handlers → use_action

## Violation

Rule: "All Dioxus components that call server functions or async operations from event handlers must use use_action instead of spawning tasks manually" (Hooks: use_action for event handlers).

File: `crates/lx-mobile/src/pages/approvals.rs`

4 onclick handlers in `render_prompt` use `spawn(async move { post_respond(...).await })` instead of `use_action`.

## Locations

- Line 34-41: "Yes" button in `confirm` prompt — `spawn(async move { let _ = post_respond(...).await; })`
- Line 47-54: "No" button in `confirm` prompt — same pattern
- Line 72-79: Option button in `choose` prompt — same pattern
- Line 105-113: "Send" button in `ask` prompt — same pattern, also has `input_text.read().clone()` on line 106

## Problem

`render_prompt` is a regular function (not a `#[component]`) called inside a `for` loop in the `Approvals` component. It already uses `use_signal` conditionally (line 90, inside the `"ask"` match arm). Using hooks inside a for loop is fragile if the list length changes between renders.

## Required Changes

### Step 1: Convert `render_prompt` match arms into proper `#[component]`s

Each match arm should become its own component so hooks (including `use_action`) have a stable scope. This eliminates the hooks-in-a-loop problem that already exists with `use_signal` on line 90.

Create 3 components to replace the 3 match arms:

**ConfirmPrompt component:**
```rust
#[component]
fn ConfirmPrompt(prompt_id: u64, message: String) -> Element {
  let respond = use_action(post_respond);
  rsx! {
    div { class: "p-3 bg-[var(--surface-container)] rounded space-y-2",
      p { class: "text-sm", "{message}" }
      div { class: "flex gap-2",
        button {
          class: "px-3 py-1 bg-[var(--success)] rounded text-sm",
          onclick: move |_| {
              respond.call(PromptResponse {
                  prompt_id,
                  response: serde_json::json!(true),
              });
          },
          "Yes"
        }
        button {
          class: "px-3 py-1 bg-[var(--error)] rounded text-sm",
          onclick: move |_| {
              respond.call(PromptResponse {
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
```

**ChoosePrompt component:**
```rust
#[component]
fn ChoosePrompt(prompt_id: u64, message: String, options: Vec<String>) -> Element {
  let respond = use_action(post_respond);
  rsx! {
    div { class: "p-3 bg-[var(--surface-container)] rounded space-y-2",
      p { class: "text-sm", "{message}" }
      for (i, opt) in options.iter().enumerate() {
        button {
          class: "block w-full text-left px-3 py-1 bg-[var(--surface-container-high)] rounded text-sm hover:bg-[var(--surface-bright)]",
          onclick: move |_| {
              respond.call(PromptResponse {
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
```

**AskPrompt component:**
```rust
#[component]
fn AskPrompt(prompt_id: u64, message: String) -> Element {
  let respond = use_action(post_respond);
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
              respond.call(PromptResponse {
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
```

### Step 2: Update the Approvals component to use the new components

Replace the `render_prompt` call in the for loop:

Current (lines 16-18):
```rust
      for prompt in prompts_ref.iter() {
        {render_prompt(prompt)}
      }
```

Replace with:
```rust
      for prompt in prompts_ref.iter() {
        match prompt.kind.as_str() {
          "confirm" => rsx! { ConfirmPrompt { prompt_id: prompt.prompt_id, message: prompt.message.clone() } },
          "choose" => rsx! { ChoosePrompt { prompt_id: prompt.prompt_id, message: prompt.message.clone(), options: prompt.options.clone().unwrap_or_default() } },
          "ask" => rsx! { AskPrompt { prompt_id: prompt.prompt_id, message: prompt.message.clone() } },
          _ => rsx! {},
        }
      }
```

### Step 3: Remove the `render_prompt` function

Delete the entire `render_prompt` function (lines 23-124).

### Step 4: Fix signal call syntax in AskPrompt

In the new `AskPrompt`, use `input_text()` instead of `input_text.read().clone()` (was line 106), and `input_text().is_empty()` instead of `input_text.read().is_empty()` (was line 104). These are shown in the code above.

## Verification

Run `just diagnose` and confirm no errors in `approvals.rs`. Verify `use_action` is imported (it's in `dioxus::prelude::*`). Verify `post_respond` is still imported from `lx_api::run_api`.
