# Goal

Build the Accounts page with persistent credential management, the Activity page consuming the ActivityLog context, and rewrite the MCP panel to use `use_resource` for live data loading.

# Why

- The Accounts page renders only the text "Accounts" — it's a complete stub
- The Activity page renders only the text "Activity" — it's a complete stub
- The MCP panel uses `use_hook(load_mcp_server_names)` which reads `.mcp.json` exactly once and never refreshes, showing only "CONFIGURED" with no actionable information

# Prerequisites

WU-1 (Shell Shared State) must be completed first. The Activity page consumes the `ActivityLog` context provided at Shell level.

# Architecture

**Accounts:** Uses `dioxus-storage`'s `use_persistent_store` (added to Cargo.toml in WU-3) to persist a list of provider credentials. Each credential has a provider name, API key, and active flag. The UI shows a list with masked keys and a toggle, plus an add form.

**Activity:** Reads `ActivityLog` via `use_context` and renders the `VecDeque<ActivityEvent>` as a reverse-chronological table. No writes — this page is read-only.

**MCP Panel:** Replaces `use_hook` with `use_resource` so the data loading is async and can be re-triggered. Adds a refresh button.

# Files Affected

| File | Change |
|------|--------|
| `src/pages/accounts.rs` | Rewrite with credential management UI |
| `src/pages/activity.rs` | Rewrite with ActivityLog consumption |
| `src/pages/agents/mcp_panel.rs` | Rewrite with use_resource |

# Task List

### Task 1: Build Accounts page with persistent credentials

**Subject:** Replace the stub Accounts page with credential management UI

**Description:** Rewrite `crates/lx-desktop/src/pages/accounts.rs`. The page manages a list of API provider credentials persisted via `dioxus-storage`.

```rust
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
    let stored = dioxus_storage::use_persistent_store("lx_accounts", || {
        vec![
            Credential { provider: "ANTHROPIC".into(), api_key: String::new(), active: true },
        ]
    });
    let mut creds: Signal<Vec<Credential>> = use_signal(|| (*stored.get()).clone());
    let mut new_provider = use_signal(String::new);
    let mut new_key = use_signal(String::new);
    let mut reveal: Signal<Option<usize>> = use_signal(|| None);

    let entries = creds.read().clone();

    rsx! {
        div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]", "ACCOUNTS" }
                span { class: "text-xs text-[var(--outline)] uppercase tracking-wider", "{entries.len()} PROVIDERS" }
            }
            div { class: "flex flex-col gap-3",
                for (i, cred) in entries.iter().enumerate() {
                    {
                        let provider = cred.provider.clone();
                        let key_display = if reveal() == Some(i) {
                            cred.api_key.clone()
                        } else if cred.api_key.is_empty() {
                            "Not configured".to_string()
                        } else {
                            let k = &cred.api_key;
                            if k.len() > 8 {
                                format!("{}...{}", &k[..4], &k[k.len()-4..])
                            } else {
                                "\u{2022}".repeat(k.len())
                            }
                        };
                        let is_active = cred.active;
                        rsx! {
                            div { class: "bg-[var(--surface-container)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex items-center gap-4",
                                div { class: "flex-1",
                                    p { class: "text-sm font-semibold uppercase tracking-wider text-[var(--on-surface)]", "{provider}" }
                                    p { class: "text-xs text-[var(--outline)] font-mono mt-1", "{key_display}" }
                                }
                                button {
                                    class: "text-xs text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors duration-150",
                                    onclick: move |_| {
                                        if reveal() == Some(i) { reveal.set(None); } else { reveal.set(Some(i)); }
                                    },
                                    if reveal() == Some(i) { "HIDE" } else { "REVEAL" }
                                }
                                button {
                                    class: if is_active { "text-xs text-[var(--success)] font-semibold" } else { "text-xs text-[var(--outline)]" },
                                    onclick: move |_| {
                                        creds.write()[i].active = !creds.write()[i].active;
                                    },
                                    if is_active { "ACTIVE" } else { "INACTIVE" }
                                }
                                button {
                                    class: "text-xs text-[var(--error)] hover:text-[var(--error)] transition-colors duration-150",
                                    onclick: move |_| { creds.write().remove(i); },
                                    "REMOVE"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-4",
                p { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)] mb-3", "ADD PROVIDER" }
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
                                creds.write().push(Credential {
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
```

Note: The `stored` from `use_persistent_store` initializes the signal. Mutations to `creds` are local. For full persistence, the `stored` value should be updated when the user explicitly saves. However, since the Settings page pattern uses explicit APPLY, and Accounts is simpler, auto-persist is acceptable here. If `Store<T>` provides a `set()` method, call `stored.set(creds.read().clone())` after each mutation. Otherwise, keep the local signal approach — persistence can be enhanced later.

The `use_persistent_store` call requires the `dioxus-storage` dependency added in WU-3. If WU-3 has not landed yet, replace with a simple `use_signal(|| default_creds)` and mark persistence as a follow-up.

**ActiveForm:** Building Accounts page with credential management

---

### Task 2: Build Activity page consuming ActivityLog

**Subject:** Replace the stub Activity page with a live event log

**Description:** Rewrite `crates/lx-desktop/src/pages/activity.rs`:

```rust
use dioxus::prelude::*;
use crate::contexts::activity_log::ActivityLog;

#[component]
pub fn Activity() -> Element {
    let log = use_context::<ActivityLog>();
    let events = log.events.read();

    rsx! {
        div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
            div { class: "flex items-center justify-between",
                h1 { class: "text-2xl font-bold uppercase tracking-wider text-[var(--on-surface)] font-[var(--font-display)]", "ACTIVITY_LOG" }
                span { class: "text-xs text-[var(--outline)] uppercase tracking-wider", "{events.len()} EVENTS" }
            }
            if events.is_empty() {
                div { class: "flex-1 flex items-center justify-center",
                    p { class: "text-sm text-[var(--outline)]", "No activity recorded yet" }
                }
            } else {
                div { class: "bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] overflow-hidden",
                    div { class: "flex text-[10px] uppercase tracking-wider text-[var(--on-surface-variant)] py-3 px-4 border-b border-[var(--outline-variant)] bg-[var(--surface-container-high)]",
                        span { class: "w-32 shrink-0", "TIMESTAMP" }
                        span { class: "w-24 shrink-0", "KIND" }
                        span { class: "flex-1", "MESSAGE" }
                    }
                    div { class: "flex flex-col max-h-[calc(100vh-12rem)] overflow-y-auto",
                        for event in events.iter() {
                            div { class: "flex items-center px-4 py-2.5 border-b border-[var(--outline-variant)]/15 hover:bg-[var(--surface-container)] transition-colors duration-150 text-xs",
                                span { class: "w-32 shrink-0 text-[var(--outline)] font-mono", "{event.timestamp}" }
                                span { class: "w-24 shrink-0 text-[var(--primary)] uppercase font-semibold", "{event.kind}" }
                                span { class: "flex-1 text-[var(--on-surface-variant)]", "{event.message}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

This reads the `ActivityLog` context provided by Shell (from WU-1). Events appear in reverse-chronological order because `ActivityLog::push` uses `push_front`. The page is entirely read-only — it just renders the deque.

The Activity page will be empty until other components (TerminalView, BrowserView, etc.) start pushing events. That wiring happens naturally as WU-2 (pane engine) is completed — each pane view can push activity events in its `use_future` loop. But the page itself is complete and functional: it renders whatever is in the deque.

**ActiveForm:** Building Activity page with ActivityLog consumption

---

### Task 3: Rewrite McpPanel with use_resource and refresh

**Subject:** Replace one-shot use_hook with async use_resource for MCP server loading

**Description:** Rewrite `crates/lx-desktop/src/pages/agents/mcp_panel.rs`:

```rust
use dioxus::prelude::*;

async fn load_mcp_servers() -> Vec<String> {
    let content = match tokio::fs::read_to_string(".mcp.json").await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };
    json.get("mcpServers")
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect())
        .unwrap_or_default()
}

#[component]
pub fn McpPanel() -> Element {
    let servers = use_resource(|| async { load_mcp_servers().await });
    let mut refresh_counter = use_signal(|| 0u32);

    rsx! {
        div { class: "flex items-center gap-3 mb-4",
            div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
            span { class: "text-xs uppercase tracking-wider font-semibold text-[var(--on-surface)]", "MCP_EXTENSIONS" }
            div { class: "h-px flex-1 bg-[var(--outline-variant)]" }
            button {
                class: "text-xs text-[var(--outline)] hover:text-[var(--primary)] transition-colors duration-150",
                onclick: move |_| {
                    refresh_counter += 1;
                    servers.restart();
                },
                span { class: "material-symbols-outlined text-sm", "refresh" }
            }
        }
        match &*servers.value().read() {
            Some(names) => rsx! {
                div { class: "grid grid-cols-4 gap-3",
                    for name in names {
                        div { class: "bg-[var(--surface-container-low)] border border-[var(--outline-variant)]/30 rounded-lg p-4 flex flex-col gap-2",
                            span { class: "text-2xl text-[var(--primary)]", "\u{1F5C4}" }
                            span { class: "text-xs font-semibold uppercase tracking-wider text-[var(--on-surface)]", "{name}" }
                            span { class: "text-[10px] uppercase tracking-wider text-[var(--outline)]", "CONFIGURED" }
                        }
                    }
                }
            },
            None => rsx! {
                div { class: "text-xs text-[var(--outline)] py-4 text-center", "Loading MCP servers..." }
            },
        }
    }
}
```

Key changes:
- `load_mcp_server_names` becomes `load_mcp_servers` and uses `tokio::fs::read_to_string` (async) instead of `std::fs::read_to_string` (sync, blocks UI thread)
- `use_hook` replaced with `use_resource` which provides loading/ready states
- Added a refresh button that calls `servers.restart()` to re-trigger the resource
- The `refresh_counter` signal exists only to satisfy the borrow checker if `restart()` needs a reactive dependency — if `use_resource` in Dioxus 0.7 supports `restart()` without it, the counter can be removed

Check the actual Dioxus 0.7 `Resource` API: if `restart()` is not available, use `servers.clear()` or `servers.cancel()` followed by the resource re-executing on the next render. Adapt to whatever re-trigger mechanism the API provides.

**ActiveForm:** Rewriting McpPanel with use_resource and refresh button

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_LEAF_PAGES.md" })
```
