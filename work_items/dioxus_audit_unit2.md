# Unit 2: Replace polling with use_loader in Approvals page

## Violation

Rules 14 (`use_loader` for data loading) and 18 (no polling loops) in `crates/lx-mobile/src/pages/approvals.rs`.

The `Approvals` component uses `use_action` + `use_future` with a 2-second polling loop to call `get_prompts`. This must be replaced with `use_loader`. The mutation handlers (`post_respond` calls via `spawn`) are correct and must be preserved as-is.

## Prerequisite: Add PartialEq to PendingPrompt

File: `crates/lx-api/src/types.rs`, line 17.

Current:
```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PendingPrompt {
```

Replace with:
```rust
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PendingPrompt {
```

`use_loader` requires `T: PartialEq + Serialize + DeserializeOwned`. Since `T = Vec<PendingPrompt>`, `PendingPrompt` must derive `PartialEq`. All fields (`u64`, `String`, `Option<Vec<String>>`) implement `PartialEq`.

## Server function signatures

From `crates/lx-api/src/run_api.rs`:
- `get_prompts` (line 32): `pub async fn get_prompts() -> Result<Vec<PendingPrompt>>` — returns `Result<Vec<PendingPrompt>, ServerFnError>`
- `post_respond` (line 37): `pub async fn post_respond(data: PromptResponse) -> Result<serde_json::Value>` — mutation, stays with `spawn`

## Step 1: Replace imports

File: `crates/lx-mobile/src/pages/approvals.rs`, lines 1-3.

Current:
```rust
use dioxus::prelude::*;
use lx_api::run_api::{get_prompts, post_respond};
use lx_api::types::{PendingPrompt, PromptResponse};
```

Replace with:
```rust
use dioxus::prelude::*;
use lx_api::run_api::{get_prompts, post_respond};
use lx_api::types::{PendingPrompt, PromptResponse};
```

No import changes needed. `use_loader` is in `dioxus::prelude::*`. Removing `use_action` and `use_future` is done by removing their call sites (neither had explicit imports).

## Step 2: Replace the Approvals component body

File: `crates/lx-mobile/src/pages/approvals.rs`, lines 5-29.

### Remove polling infrastructure (lines 7-16)

Current:
```rust
pub fn Approvals() -> Element {
  let mut action = use_action(get_prompts);

  use_future(move || async move {
    loop {
      action.call();
      tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
  });

  let prompts: Vec<PendingPrompt> = action.value().and_then(|r| r.ok()).map(|s| s.read().clone()).unwrap_or_default();
```

Replace with:
```rust
pub fn Approvals() -> Element {
  let prompts = use_loader(|| get_prompts())?;
```

This replaces:
- The `use_action` call (line 7)
- The entire `use_future` polling loop (lines 9-14)
- The `prompts` extraction chain (line 16)

The `?` operator propagates `Loading::Pending` (suspends) and `Loading::Failed` (error) via `From<Loading> for RenderError`.

`prompts` is now a `Loader<Vec<PendingPrompt>>`, which implements `Readable`. It can be read via `.read()` to get a `&Vec<PendingPrompt>`.

### Update RSX block (lines 18-28)

Current:
```rust
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
```

Replace with:
```rust
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
```

Changes:
- Add `let prompts_ref = prompts.read();` before the RSX block to get a `&Vec<PendingPrompt>` reference
- `prompts.is_empty()` becomes `prompts_ref.is_empty()`
- `prompts.iter()` becomes `prompts_ref.iter()`

### Keep render_prompt function unchanged

The `render_prompt` function (lines 31-132) uses `spawn` for `post_respond` mutation calls. This is correct per the audit rules (use_action for event handlers, spawn for fire-and-forget mutations). Do not modify this function.

## Complete final file

```rust
use dioxus::prelude::*;
use lx_api::run_api::{get_prompts, post_respond};
use lx_api::types::{PendingPrompt, PromptResponse};

#[component]
pub fn Approvals() -> Element {
  let prompts = use_loader(|| get_prompts())?;

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
```

## Verification

After making the changes, run `just diagnose` and confirm no errors related to `approvals.rs`.
