use std::sync::Arc;

use dioxus::prelude::*;
use lx_dx::adapters::ansi;
use lx_dx::event::{EventBus, RuntimeEvent};
use lx_ui::components::PageHeader;

#[component]
pub fn Events() -> Element {
    let bus: Signal<Arc<EventBus>> = use_context();
    let mut events: Signal<Vec<RuntimeEvent>> = use_signal(Vec::new);
    let mut filter = use_signal(|| "all".to_string());

    use_future(move || async move {
        let mut rx = bus.read().subscribe();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let mut evts = events.write();
                    evts.push(event);
                    if evts.len() > 10_000 {
                        evts.remove(0);
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
            }
        }
    });

    let current_filter = filter.read().clone();
    let visible: Vec<_> = events
        .read()
        .iter()
        .filter(|e| matches_filter(e, &current_filter))
        .cloned()
        .collect();

    let total = events.read().len();
    let visible_count = visible.len();

    rsx! {
        PageHeader {
            title: "Events".to_string(),
            subtitle: Some(format!("{visible_count}/{total} events")),
        }
        div { class: "p-4",
            div { class: "flex gap-2 mb-4 flex-wrap",
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
            div { class: "space-y-1 max-h-96 overflow-auto",
                for event in visible.iter().rev().take(200) {
                    {render_event(event)}
                }
            }
        }
    }
}

const FILTER_OPTIONS: &[&str] = &[
    "all", "ai", "emit", "log", "shell", "messages", "agents", "progress", "errors",
];

fn matches_filter(event: &RuntimeEvent, filter: &str) -> bool {
    match filter {
        "all" => true,
        "ai" => matches!(
            event,
            RuntimeEvent::AiCallStart { .. }
                | RuntimeEvent::AiCallComplete { .. }
                | RuntimeEvent::AiCallError { .. }
        ),
        "emit" => matches!(event, RuntimeEvent::Emit { .. }),
        "log" => matches!(event, RuntimeEvent::Log { .. }),
        "shell" => matches!(
            event,
            RuntimeEvent::ShellExec { .. } | RuntimeEvent::ShellResult { .. }
        ),
        "messages" => matches!(
            event,
            RuntimeEvent::MessageSend { .. }
                | RuntimeEvent::MessageAsk { .. }
                | RuntimeEvent::MessageResponse { .. }
                | RuntimeEvent::UserPrompt { .. }
                | RuntimeEvent::UserResponse { .. }
        ),
        "agents" => matches!(
            event,
            RuntimeEvent::AgentSpawned { .. } | RuntimeEvent::AgentKilled { .. }
        ),
        "progress" => matches!(
            event,
            RuntimeEvent::Progress { .. }
                | RuntimeEvent::ProgramStarted { .. }
                | RuntimeEvent::ProgramFinished { .. }
                | RuntimeEvent::TraceSpanRecorded { .. }
        ),
        "errors" => matches!(event, RuntimeEvent::Error { .. }),
        _ => true,
    }
}

fn render_event(event: &RuntimeEvent) -> Element {
    let text = ansi::format_event(event);
    let clean = strip_ansi(&text);
    rsx! {
        div { class: "text-xs font-mono text-gray-300 py-0.5 px-2 hover:bg-gray-800 rounded",
            "{clean}"
        }
    }
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_escape = false;
    for c in s.chars() {
        if c == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        result.push(c);
    }
    result
}
