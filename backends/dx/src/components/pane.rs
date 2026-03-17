use std::sync::Arc;

use dioxus::prelude::*;

use crate::event::{EventBus, RuntimeEvent};

#[derive(Props, Clone, PartialEq)]
pub struct PaneProps {
    pub agent_id: String,
    pub bus: Arc<EventBus>,
}

#[component]
pub fn Pane(props: PaneProps) -> Element {
    let mut events: Signal<Vec<RuntimeEvent>> = use_signal(Vec::new);
    let agent_id = props.agent_id.clone();
    let bus = props.bus.clone();

    use_future(move || {
        let agent_id = agent_id.clone();
        let bus = bus.clone();
        async move {
            let mut rx = bus.subscribe();
            loop {
                match rx.recv().await {
                    Ok(ev) => {
                        let matches = ev
                            .agent_id()
                            .is_some_and(|id| id == agent_id)
                            || matches!(
                                ev,
                                RuntimeEvent::ProgramStarted { .. }
                                    | RuntimeEvent::ProgramFinished { .. }
                            );
                        if matches {
                            events.write().push(ev);
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        events.write().push(RuntimeEvent::Log {
                            agent_id: agent_id.clone(),
                            level: "warn".to_string(),
                            msg: format!("skipped {n} events (lagged)"),
                            ts: std::time::Instant::now(),
                        });
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    });

    rsx! {
        div {
            class: "pane",
            div {
                class: "pane-header",
                span { class: "pane-title", "{props.agent_id}" }
            }
            div {
                class: "pane-body",
                for (i, ev) in events.read().iter().enumerate() {
                    {render_event(ev, i)}
                }
            }
        }
    }
}

fn render_event(ev: &RuntimeEvent, key: usize) -> Element {
    match ev {
        RuntimeEvent::AiCallStart {
            prompt, model, ..
        } => {
            let model_label = model
                .as_deref()
                .unwrap_or("ai");
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-ai-start",
                    span { class: "event-tag", "[AI] {model_label}" }
                    pre { class: "event-prompt", "{prompt}" }
                }
            }
        }
        RuntimeEvent::AiCallComplete {
            response,
            cost_usd,
            duration_ms,
            model,
            ..
        } => {
            let cost_str = cost_usd
                .map(|c| format!("${c:.4}"))
                .unwrap_or_default();
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-ai-complete",
                    pre { class: "event-response", "{response}" }
                    div {
                        class: "event-meta",
                        span { "{model}" }
                        span { "{cost_str}" }
                        span { "{duration_ms}ms" }
                    }
                }
            }
        }
        RuntimeEvent::AiCallError { error, .. } => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-ai-error",
                    span { class: "event-tag", "[AI ERROR]" }
                    span { class: "error-text", "{error}" }
                }
            }
        }
        RuntimeEvent::Emit { value, .. } => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-emit",
                    "{value}"
                }
            }
        }
        RuntimeEvent::Log { level, msg, .. } => {
            let class = match level.as_str() {
                "info" => "event event-log log-info",
                "warn" => "event event-log log-warn",
                "err" => "event event-log log-err",
                "debug" => "event event-log log-debug",
                _ => "event event-log",
            };
            rsx! {
                div {
                    key: "{key}",
                    class: "{class}",
                    span { class: "log-level", "[{level}]" }
                    span { " {msg}" }
                }
            }
        }
        RuntimeEvent::ShellExec { cmd, .. } => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-shell",
                    code { "$ {cmd}" }
                }
            }
        }
        RuntimeEvent::ShellResult {
            exit_code,
            stdout,
            stderr,
            ..
        } => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-shell-result",
                    if !stdout.is_empty() {
                        pre { class: "shell-stdout", "{stdout}" }
                    }
                    if !stderr.is_empty() {
                        pre { class: "shell-stderr", "{stderr}" }
                    }
                    span {
                        class: if *exit_code == 0 { "exit-ok" } else { "exit-err" },
                        "exit {exit_code}"
                    }
                }
            }
        }
        RuntimeEvent::Error {
            error, span_info, ..
        } => {
            let location = span_info
                .as_ref()
                .map(|s| {
                    format!(
                        " ({}:{}-{}:{})",
                        s.start_line, s.start_col, s.end_line, s.end_col
                    )
                })
                .unwrap_or_default();
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-error",
                    span { class: "error-text", "{error}{location}" }
                }
            }
        }
        RuntimeEvent::Progress {
            current,
            total,
            message,
            ..
        } => {
            let pct = if *total > 0 {
                (*current as f64 / *total as f64 * 100.0) as u32
            } else {
                0
            };
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-progress",
                    div {
                        class: "progress-bar",
                        div {
                            class: "progress-fill",
                            width: "{pct}%",
                        }
                    }
                    span { "[{current}/{total}] {message}" }
                }
            }
        }
        RuntimeEvent::ProgramStarted { source_path, .. } => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-program-start",
                    "Started: {source_path}"
                }
            }
        }
        RuntimeEvent::ProgramFinished {
            result,
            duration_ms,
            ..
        } => {
            let (class, text) = match result {
                Ok(v) => ("event-program-ok", format!("Finished: {v}")),
                Err(e) => ("event-program-err", format!("Failed: {e}")),
            };
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-program-finish {class}",
                    span { "{text}" }
                    span { class: "duration", " ({duration_ms}ms)" }
                }
            }
        }
        RuntimeEvent::UserPrompt { kind, .. } => {
            crate::components::user_prompt::render_prompt_inline(kind, key)
        }
        _ => {
            rsx! {
                div {
                    key: "{key}",
                    class: "event event-generic",
                    "{ev:?}"
                }
            }
        }
    }
}
