use dioxus::prelude::*;

use crate::api_client::LxClient;
use crate::ws_client::EventWsClient;

#[component]
pub fn Events() -> Element {
    let client: Signal<LxClient> = use_context();
    let mut events: Signal<Vec<serde_json::Value>> = use_signal(Vec::new);
    let mut filter = use_signal(|| "all".to_string());
    let expanded: Signal<std::collections::HashSet<usize>> = use_signal(Default::default);

    use_future(move || {
        let base_url = client.read().base_url_for_spawn();
        async move {
            let ws_client = EventWsClient::new(&base_url);
            let (tx, mut rx) = tokio::sync::mpsc::channel::<serde_json::Value>(256);
            tokio::spawn(async move {
                ws_client.connect_and_stream(tx).await;
            });
            while let Some(val) = rx.recv().await {
                let mut evts = events.write();
                evts.push(val);
                if evts.len() > 10_000 {
                    evts.remove(0);
                }
            }
        }
    });

    let current_filter = filter.read().clone();
    let visible: Vec<_> = events
        .read()
        .iter()
        .filter(|e| {
            if current_filter == "all" {
                return true;
            }
            e.get("type")
                .and_then(|t| t.as_str())
                .is_some_and(|t| event_type_matches(t, &current_filter))
        })
        .cloned()
        .collect();

    let total = events.read().len();
    let visible_count = visible.len();

    rsx! {
        div { class: "space-y-4",
            h2 { class: "text-lg font-bold", "Events ({visible_count}/{total})" }
            div { class: "flex gap-2 flex-wrap",
                for f in FILTER_OPTIONS {
                    button {
                        class: "px-2 py-1 text-xs rounded",
                        class: if current_filter == *f { "bg-blue-600 text-white" } else { "bg-gray-700 text-gray-300" },
                        onclick: {
                            let f = f.to_string();
                            move |_| filter.set(f.clone())
                        },
                        "{f}"
                    }
                }
            }
            div { class: "space-y-1",
                for (idx, event) in visible.iter().enumerate().rev().take(100) {
                    {render_mobile_event(idx, event, &expanded)}
                }
                if visible.is_empty() {
                    p { class: "text-gray-500 text-sm", "No events yet" }
                }
            }
        }
    }
}

const FILTER_OPTIONS: &[&str] = &[
    "all", "ai", "emit", "log", "shell", "messages", "agents", "progress", "errors",
];

fn event_type_matches(event_type: &str, filter: &str) -> bool {
    match filter {
        "ai" => event_type.starts_with("ai_"),
        "emit" => event_type == "emit",
        "log" => event_type == "log",
        "shell" => event_type.starts_with("shell_"),
        "messages" => {
            event_type.starts_with("message_")
                || event_type == "user_prompt"
                || event_type == "user_response"
        }
        "agents" => event_type.starts_with("agent_"),
        "progress" => {
            event_type == "progress"
                || event_type == "program_started"
                || event_type == "program_finished"
                || event_type == "trace_span_recorded"
        }
        "errors" => event_type == "error",
        _ => true,
    }
}

fn render_mobile_event(
    idx: usize,
    event: &serde_json::Value,
    expanded: &Signal<std::collections::HashSet<usize>>,
) -> Element {
    let event_type = event
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");
    let agent = event
        .get("agent_id")
        .and_then(|a| a.as_str())
        .unwrap_or("system");
    let is_expanded = expanded.read().contains(&idx);
    let detail = serde_json::to_string_pretty(event).unwrap_or_default();
    let mut expanded = *expanded;
    rsx! {
        div {
            class: "p-2 bg-gray-800 rounded text-xs cursor-pointer",
            onclick: move |_| {
                let mut set = expanded.write();
                if set.contains(&idx) {
                    set.remove(&idx);
                } else {
                    set.insert(idx);
                }
            },
            div {
                span { class: "text-gray-500", "[{agent}] " }
                span { class: "text-gray-300", "{event_type}" }
            }
            if is_expanded {
                pre { class: "mt-1 text-gray-400 whitespace-pre-wrap break-all", "{detail}" }
            }
        }
    }
}
