use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Credential {
  provider: String,
  api_key: String,
  active: bool,
}

#[component]
pub fn Accounts() -> Element {
  let creds = dioxus_storage::use_persistent("lx_accounts", || vec![Credential { provider: "ANTHROPIC".into(), api_key: String::new(), active: true }]);
  let mut new_provider = use_signal(String::new);
  let mut new_key = use_signal(String::new);
  let reveal: Signal<Option<usize>> = use_signal(|| None);

  let entries = creds();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: "flex-between",
        h1 { class: "page-heading", "ACCOUNTS" }
        span { class: "text-xs text-[var(--outline)] uppercase tracking-wider",
          "{entries.len()} PROVIDERS"
        }
      }
      div { class: "flex flex-col gap-3",
        for (i , cred) in entries.iter().enumerate() {
          {
              let provider = cred.provider.clone();
              let key_display = if reveal() == Some(i) {
                  cred.api_key.clone()
              } else if cred.api_key.is_empty() {
                  "Not configured".to_string()
              } else {
                  let k = &cred.api_key;
                  if k.len() > 8 {
                      format!("{}...{}", &k[..4], &k[k.len() - 4..])
                  } else {
                      "\u{2022}".repeat(k.len())
                  }
              };
              let is_active = cred.active;
              rsx! {
                div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex items-center gap-4",
                  div { class: "flex-1",
                    p { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]",
                      "{provider}"
                    }
                    p { class: "text-xs text-[var(--outline)] font-mono mt-1", "{key_display}" }
                  }
                  button {
                    class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors duration-150",
                    onclick: move |_| {
                        let mut r = reveal;
                        if r() == Some(i) {
                            r.set(None);
                        } else {
                            r.set(Some(i));
                        }
                    },
                    if reveal() == Some(i) {
                      "HIDE"
                    } else {
                      "REVEAL"
                    }
                  }
                  button {
                    class: if is_active { "text-xs text-[var(--success)] font-semibold" } else { "text-xs text-[var(--outline)]" },
                    onclick: move |_| {
                        let mut c = creds;
                        let current = c.read()[i].active;
                        c.write()[i].active = !current;
                    },
                    if is_active {
                      "ACTIVE"
                    } else {
                      "INACTIVE"
                    }
                  }
                  button {
                    class: "text-xs text-[var(--error)] hover:text-[var(--error)] transition-colors duration-150",
                    onclick: move |_| {
                        let mut c = creds;
                        c.write().remove(i);
                    },
                    "REMOVE"
                  }
                }
              }
          }
        }
      }
      div { class: "bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-4",
        p { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)] mb-3",
          "ADD PROVIDER"
        }
        div { class: "flex gap-3",
          input {
            class: "flex-1 bg-[var(--surface-container-lowest)] text-xs px-3 py-2 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)] uppercase",
            placeholder: "PROVIDER NAME",
            value: "{new_provider}",
            oninput: move |evt| new_provider.set(evt.value()),
          }
          input {
            class: "flex-[2] bg-[var(--surface-container-lowest)] text-xs px-3 py-2 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)] font-mono",
            placeholder: "API Key",
            r#type: "password",
            value: "{new_key}",
            oninput: move |evt| new_key.set(evt.value()),
          }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-6 py-2 text-xs uppercase font-semibold hover:brightness-110 transition-all duration-150 rounded",
            onclick: move |_| {
                let p = new_provider().trim().to_uppercase();
                if !p.is_empty() {
                    let mut c = creds;
                    c.write()
                        .push(Credential {
                            provider: p,
                            api_key: new_key().trim().to_string(),
                            active: true,
                        });
                    new_provider.set(String::new());
                    new_key.set(String::new());
                }
            },
            "ADD"
          }
        }
      }
    }
  }
}
