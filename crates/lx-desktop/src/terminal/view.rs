use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use dioxus::prelude::*;
use lx_ui::tab_state::NotificationLevel;
use lx_ui::tab_state::PaneNotification;

use super::use_tabs_state;
use crate::ts_widget::use_ts_widget;

#[component]
pub fn TerminalView(terminal_id: String, working_dir: String, command: Option<String>) -> Element {
    let (element_id, widget) = use_ts_widget("terminal", serde_json::json!({}));
    let mut tabs_state = use_tabs_state();
    let tid_notif = terminal_id.clone();

    let eid_rsx = element_id.clone();
    use_future(move || {
        let element_id = element_id.clone();
        let tid_notif = tid_notif.clone();
        let wd = working_dir.clone();
        let cmd = command.clone();
        async move {
            let session = match lx_ui::pty_session::get_or_create(
                &element_id,
                80,
                24,
                Some(&wd),
                cmd.as_deref(),
            ) {
                Ok(s) => s,
                Err(e) => {
                    dioxus::logger::tracing::error!("terminal {element_id}: PTY failed: {e}");
                    return;
                }
            };

            let (initial, mut rx) = session.subscribe();
            if !initial.is_empty() {
                widget.send_update(B64.encode(&initial));
            }

            loop {
                tokio::select! {
                    result = rx.recv() => {
                        match result {
                            Ok(bytes) => {
                                widget.send_update(B64.encode(&bytes));
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                tabs_state.write().set_notification(
                                    &tid_notif,
                                    PaneNotification { level: NotificationLevel::Success, message: None },
                                );
                                break;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                        }
                    }
                    result = widget.recv::<serde_json::Value>() => {
                        match result {
                            Ok(msg) => match msg["type"].as_str() {
                                Some("input") => {
                                    if let Some(data) = msg["data"].as_str() {
                                        let _ = session.send_input(data.as_bytes().to_vec()).await;
                                    }
                                }
                                Some("resize") => {
                                    let cols = msg["cols"].as_u64().unwrap_or(80) as u16;
                                    let rows = msg["rows"].as_u64().unwrap_or(24) as u16;
                                    let _ = session.resize(cols, rows);
                                    widget.send_resize();
                                }
                                _ => {}
                            },
                            Err(_) => break,
                        }
                    }
                }
            }
        }
    });

    rsx! {
        div {
            id: "{eid_rsx}",
            class: "w-full h-full bg-gray-950 overflow-hidden",
        }
    }
}

#[component]
pub fn BrowserView(browser_id: String, url: String, devtools: bool) -> Element {
    let (element_id, _widget) = use_ts_widget("browser", serde_json::json!({ "url": url }));

    rsx! {
        div {
            id: "{element_id}",
            class: "w-full h-full",
        }
    }
}

#[component]
pub fn EditorView(editor_id: String, file_path: String, language: Option<String>) -> Element {
    let lang = language.unwrap_or_else(|| "plaintext".into());
    let (element_id, _widget) = use_ts_widget(
        "editor",
        serde_json::json!({
            "language": lang,
            "filePath": file_path,
        }),
    );

    rsx! {
        div {
            id: "{element_id}",
            class: "w-full h-full",
        }
    }
}

#[component]
pub fn AgentView(agent_id: String, session_id: String, model: String) -> Element {
    let (element_id, _widget) = use_ts_widget("agent", serde_json::json!({}));

    rsx! {
        div {
            id: "{element_id}",
            class: "w-full h-full",
        }
    }
}

#[component]
pub fn CanvasView(canvas_id: String, widget_type: String, config: serde_json::Value) -> Element {
    let (element_id, _widget) = use_ts_widget(&widget_type, &config);

    rsx! {
        div {
            id: "{element_id}",
            class: "w-full h-full",
        }
    }
}

#[component]
pub fn ChartView(chart_id: String, chart_json: String, title: Option<String>) -> Element {
    let div_id = use_hook(|| format!("chart-{}", uuid::Uuid::new_v4().simple()));
    let id = div_id.clone();
    use_effect(move || {
        let json = chart_json.clone();
        if json.is_empty() {
            return;
        }
        document::eval(&format!("LxCharts.initChart('{id}', {json})"));
    });
    let id_drop = div_id.clone();
    use_drop(move || {
        document::eval(&format!("LxCharts.disposeChart('{id_drop}')"));
    });
    rsx! {
        div { id: "{div_id}", class: "w-full h-full min-h-32" }
    }
}

#[component]
pub fn FlowGraphView(graph_id: String, source_path: String) -> Element {
    let (element_id, widget) = use_ts_widget("flow-graph", serde_json::json!({}));
    let source = source_path.clone();
    let bus: Signal<std::sync::Arc<lx_dx::event::EventBus>> = use_context();

    use_future(move || {
        let source = source.clone();
        async move {
            let src = match std::fs::read_to_string(&source) {
                Ok(s) => s,
                Err(e) => {
                    dioxus::logger::tracing::error!("flow graph: read error: {e}");
                    return;
                }
            };
            let tokens = match lx::lexer::lex(&src) {
                Ok(t) => t,
                Err(e) => {
                    dioxus::logger::tracing::error!("flow graph: lex error: {e}");
                    return;
                }
            };
            let program = match lx::parser::parse(tokens) {
                Ok(p) => p,
                Err(e) => {
                    dioxus::logger::tracing::error!("flow graph: parse error: {e}");
                    return;
                }
            };
            let graph_json = lx::stdlib::diag::extract_echart_json(&program);
            widget.send_update(serde_json::json!({
                "type": "full-update",
                "graphData": serde_json::from_str::<serde_json::Value>(&graph_json).unwrap_or_default()
            }));

            while let Ok(msg) = widget.recv::<serde_json::Value>().await {
                if msg["type"].as_str() == Some("node-click")
                    && let Some(offset) = msg["sourceOffset"].as_u64()
                {
                    let line = src[..offset as usize]
                        .chars()
                        .filter(|&c| c == '\n')
                        .count()
                        + 1;
                    dioxus::logger::tracing::info!("flow graph: clicked node at line {line}");
                }
            }
        }
    });

    let widget_bus = widget;
    use_future(move || async move {
        let mut rx = bus.read().subscribe();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let status_update = match &event {
                        lx_dx::event::RuntimeEvent::AgentSpawned { name, .. } => {
                            Some((name.clone(), "running"))
                        }
                        lx_dx::event::RuntimeEvent::AiCallStart { agent_id, .. } => {
                            Some((agent_id.clone(), "active"))
                        }
                        lx_dx::event::RuntimeEvent::AiCallComplete { agent_id, .. } => {
                            Some((agent_id.clone(), "completed"))
                        }
                        lx_dx::event::RuntimeEvent::AiCallError { agent_id, .. } => {
                            Some((agent_id.clone(), "error"))
                        }
                        lx_dx::event::RuntimeEvent::Error { agent_id, .. } => {
                            Some((agent_id.clone(), "error"))
                        }
                        lx_dx::event::RuntimeEvent::AgentKilled { agent_id, .. } => {
                            Some((agent_id.clone(), "completed"))
                        }
                        _ => None,
                    };
                    if let Some((node_id, status)) = status_update {
                        widget_bus.send_update(serde_json::json!({
                            "type": "node-status",
                            "nodeId": node_id,
                            "status": status
                        }));
                    }
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
            }
        }
    });

    rsx! {
        div {
            id: "{element_id}",
            class: "w-full h-full bg-gray-950",
        }
    }
}
