use std::collections::VecDeque;

use dioxus::fullstack::{WebSocketOptions, use_websocket};
use dioxus::prelude::*;
use lx_api::types::ActivityEvent;
use lx_api::ws_events::ws_events;

#[component]
pub fn Events() -> Element {
  let mut events: Signal<VecDeque<ActivityEvent>> = use_signal(VecDeque::new);
  let mut filter = use_signal(|| "all".to_string());
  let expanded: Signal<std::collections::HashSet<usize>> = use_signal(Default::default);

  let mut socket = use_websocket(|| ws_events(WebSocketOptions::new()));

  use_future(move || async move {
    while let Ok(event) = socket.recv().await {
      let mut evts = events.write();
      evts.push_back(event);
      if evts.len() > 10_000 {
        evts.pop_front();
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
      event_type_matches(&e.kind, &current_filter)
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
            class: if current_filter == *f { "bg-[var(--primary)] text-[var(--on-primary)]" } else { "bg-[var(--surface-container-high)] text-[var(--on-surface-variant)]" },
            onclick: {
                let f = f.to_string();
                move |_| filter.set(f.clone())
            },
            "{f}"
          }
        }
      }
      div { class: "space-y-1",
        for (idx , event) in visible.iter().enumerate().rev().take(100) {
          {render_mobile_event(idx, event, &expanded)}
        }
        if visible.is_empty() {
          p { class: "text-[var(--outline)] text-sm", "No events yet" }
        }
      }
    }
  }
}

const FILTER_OPTIONS: &[&str] = &["all", "ai", "emit", "log", "shell", "messages", "agents", "progress", "errors"];

fn event_type_matches(event_type: &str, filter: &str) -> bool {
  match filter {
    "ai" => event_type.starts_with("ai_"),
    "emit" => event_type == "emit",
    "log" => event_type == "log",
    "shell" => event_type.starts_with("shell_"),
    "messages" => event_type.starts_with("message_") || event_type == "user_prompt" || event_type == "user_response",
    "agents" => event_type.starts_with("agent_"),
    "progress" => event_type == "progress" || event_type == "program_started" || event_type == "program_finished" || event_type == "trace_span_recorded",
    "errors" => event_type == "error",
    _ => true,
  }
}

fn render_mobile_event(idx: usize, event: &ActivityEvent, expanded: &Signal<std::collections::HashSet<usize>>) -> Element {
  let is_expanded = expanded.read().contains(&idx);
  let detail = format!("kind: {}\ntimestamp: {}\nmessage: {}", event.kind, event.timestamp, event.message);
  let mut expanded = *expanded;
  let kind = event.kind.clone();
  rsx! {
    div {
      class: "p-2 bg-[var(--surface-container)] rounded text-xs cursor-pointer",
      onclick: move |_| {
          let mut set = expanded.write();
          if set.contains(&idx) {
              set.remove(&idx);
          } else {
              set.insert(idx);
          }
      },
      div {
        span { class: "text-[var(--on-surface-variant)]", "{kind}" }
      }
      if is_expanded {
        pre { class: "mt-1 text-[var(--outline)] whitespace-pre-wrap break-all",
          "{detail}"
        }
      }
    }
  }
}
