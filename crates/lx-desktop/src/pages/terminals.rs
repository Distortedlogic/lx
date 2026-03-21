use dioxus::prelude::*;
use lx_ui::pane_tree::{PaneNode, Rect, SplitDirection};
use lx_ui::tab_state::TabsState;
use uuid::Uuid;

use crate::terminal::tab_bar::TabBar;
use crate::terminal::toolbar::PaneToolbar;
use crate::terminal::use_tabs_state;
use crate::terminal::view::{
    AgentView, BrowserView, CanvasView, ChartView, EditorView, FlowGraphView, TerminalView,
};

fn create_new_tab(tabs_state: Signal<TabsState>) {
    let working_dir = tabs_state
        .read()
        .focused_working_dir()
        .or_else(|| {
            std::env::current_dir()
                .ok()
                .map(|p| p.display().to_string())
        })
        .unwrap_or_else(|| ".".into());
    let id = Uuid::new_v4().to_string();
    let title = format!("Terminal {}", tabs_state.read().tabs.len() + 1);
    crate::terminal::add_terminal_tab(tabs_state, id, title, working_dir, None);
}

fn split_pane(mut tabs_state: Signal<TabsState>, pane_id: &str, direction: SplitDirection) {
    let new_id = Uuid::new_v4().to_string();
    let new_pane = {
        let state = tabs_state.read();
        match state.active_tab() {
            Some(tab) => {
                let panes = tab.root.compute_pane_rects(Rect::default());
                let source = panes.iter().find(|(p, _)| p.pane_id() == Some(pane_id));
                match source.map(|(p, _)| p) {
                    Some(PaneNode::Terminal { working_dir, .. }) => PaneNode::Terminal {
                        id: new_id,
                        working_dir: working_dir.clone(),
                        command: None,
                    },
                    _ => PaneNode::Terminal {
                        id: new_id,
                        working_dir: ".".into(),
                        command: None,
                    },
                }
            }
            None => PaneNode::Terminal {
                id: new_id,
                working_dir: ".".into(),
                command: None,
            },
        }
    };
    tabs_state.write().split_pane(pane_id, direction, new_pane);
}

fn close_pane(mut tabs_state: Signal<TabsState>, pane_id: &str) {
    tabs_state.write().close_pane_in_active_tab(pane_id);
}

#[component]
pub fn Terminals() -> Element {
    let tabs_state = use_tabs_state();
    let state = tabs_state.read();
    let tabs = state.tabs.clone();
    let active_tab_id = state.active_tab_id.clone();
    let focused_pane_id = state.focused_pane_id.clone();
    let has_tabs = !tabs.is_empty();
    drop(state);

    rsx! {
        div { class: "flex flex-col h-full",
            div { class: "flex items-center",
                div { class: "flex-1",
                    TabBar {
                        tabs_state,
                        on_new_tab: move |_| create_new_tab(tabs_state),
                    }
                }
            }
            if has_tabs {
                div { class: "relative flex-1 min-h-0",
                    for tab in tabs.iter() {
                        {render_tab(tabs_state, tab, &active_tab_id, &focused_pane_id)}
                    }
                }
            } else {
                div { class: "flex flex-1 items-center justify-center text-gray-400",
                    div { class: "text-center",
                        p { class: "text-lg mb-2", "No terminals open" }
                        button {
                            class: "px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-500",
                            onclick: move |_| create_new_tab(tabs_state),
                            "New Terminal"
                        }
                    }
                }
            }
        }
    }
}

fn render_tab(
    tabs_state: Signal<TabsState>,
    tab: &lx_ui::tab_state::TerminalTab,
    active_tab_id: &Option<String>,
    focused_pane_id: &Option<String>,
) -> Element {
    let is_active = active_tab_id.as_deref() == Some(tab.id.as_str());
    let visibility = if is_active { "visible" } else { "hidden" };
    let tab_container_id = format!("tab-container-{}", tab.id);
    let full_rect = Rect::default();
    let pane_rects = tab.root.compute_pane_rects(full_rect);
    let dividers = tab.root.compute_dividers(full_rect);

    rsx! {
        div {
            id: "{tab_container_id}",
            key: "{tab.id}",
            class: "absolute inset-0 bg-gray-900",
            style: "visibility: {visibility};",
            for (pane, rect) in pane_rects.iter() {
                {render_pane_item(tabs_state, pane, rect, focused_pane_id)}
            }
            for divider in dividers.iter() {
                {render_divider_item(tabs_state, divider, &tab_container_id)}
            }
        }
    }
}

fn render_pane_item(
    mut tabs_state: Signal<TabsState>,
    pane: &PaneNode,
    rect: &Rect,
    focused_pane_id: &Option<String>,
) -> Element {
    let pid = pane.pane_id().unwrap_or_default().to_owned();
    let is_focused = focused_pane_id.as_deref() == Some(pid.as_str());
    let border = if is_focused {
        "border-blue-400"
    } else {
        "border-gray-700"
    };
    let pid_focus = pid.clone();
    let pid_sh = pid.clone();
    let pid_sv = pid.clone();
    let pid_close = pid.clone();
    let pid_convert = pid.clone();
    let pane_node_toolbar = pane.clone();
    let pane_view = pane.clone();

    rsx! {
        div {
            key: "{pid}",
            class: "group absolute flex flex-col border {border}",
            style: "left: {rect.left}%; top: {rect.top}%; width: {rect.width}%; height: {rect.height}%;",
            onclick: move |_| {
                tabs_state.write().focused_pane_id = Some(pid_focus.clone());
            },
            PaneToolbar {
                pane_node: pane_node_toolbar,
                on_split_h: move |_| split_pane(tabs_state, &pid_sh, SplitDirection::Horizontal),
                on_split_v: move |_| split_pane(tabs_state, &pid_sv, SplitDirection::Vertical),
                on_close: move |_| close_pane(tabs_state, &pid_close),
                on_navigate: None::<EventHandler<String>>,
                on_convert: move |new_node: PaneNode| {
                    tabs_state.write().convert_pane_in_active_tab(&pid_convert, new_node);
                },
            }
            div { class: "flex-1 min-h-0",
                {render_pane_view(&pane_view)}
            }
        }
    }
}

fn render_pane_view(pane: &PaneNode) -> Element {
    match pane {
        PaneNode::Terminal {
            id,
            working_dir,
            command,
        } => rsx! {
            TerminalView {
                terminal_id: id.clone(),
                working_dir: working_dir.clone(),
                command: command.clone(),
            }
        },
        PaneNode::Browser { id, url, devtools } => rsx! {
            BrowserView { browser_id: id.clone(), url: url.clone(), devtools: *devtools }
        },
        PaneNode::Editor {
            id,
            file_path,
            language,
        } => rsx! {
            EditorView {
                editor_id: id.clone(),
                file_path: file_path.clone(),
                language: language.clone(),
            }
        },
        PaneNode::Agent {
            id,
            session_id,
            model,
        } => rsx! {
            AgentView {
                agent_id: id.clone(),
                session_id: session_id.clone(),
                model: model.clone(),
            }
        },
        PaneNode::Canvas {
            id,
            widget_type,
            config,
        } => rsx! {
            CanvasView {
                canvas_id: id.clone(),
                widget_type: widget_type.clone(),
                config: config.clone(),
            }
        },
        PaneNode::Chart {
            id,
            chart_json,
            title,
        } => rsx! {
            ChartView {
                chart_id: id.clone(),
                chart_json: chart_json.clone(),
                title: title.clone(),
            }
        },
        PaneNode::FlowGraph { id, source_path } => rsx! {
            FlowGraphView {
                graph_id: id.clone(),
                source_path: source_path.clone(),
            }
        },
        PaneNode::Split { .. } => unreachable!(),
    }
}

fn render_divider_item(
    mut tabs_state: Signal<TabsState>,
    divider: &lx_ui::pane_tree::DividerInfo,
    container_id: &str,
) -> Element {
    let (div_class, div_style) = match divider.direction {
        SplitDirection::Horizontal => (
            "absolute z-20 cursor-col-resize hover:bg-blue-500/50 bg-gray-700",
            format!(
                "left: {}%; top: {}%; width: 4px; height: {}%;",
                divider.rect.left, divider.rect.top, divider.rect.height,
            ),
        ),
        SplitDirection::Vertical => (
            "absolute z-20 cursor-row-resize hover:bg-blue-500/50 bg-gray-700",
            format!(
                "left: {}%; top: {}%; width: {}%; height: 4px;",
                divider.rect.left, divider.rect.top, divider.rect.width,
            ),
        ),
    };
    let split_id = divider.split_id.clone();
    let parent_rect = divider.parent_rect;
    let direction = divider.direction;
    let cid = container_id.to_owned();

    rsx! {
        div {
            key: "div-{split_id}",
            class: "{div_class}",
            style: "{div_style}",
            onmousedown: move |evt| {
                evt.stop_propagation();
                let sid = split_id.clone();
                let cid = cid.clone();
                let is_h = matches!(direction, SplitDirection::Horizontal);
                let p_start = if is_h { parent_rect.left } else { parent_rect.top };
                let p_size = if is_h { parent_rect.width } else { parent_rect.height };
                spawn(async move {
                    let js = build_drag_js(&cid, is_h, p_start, p_size);
                    let mut eval = document::eval(&js);
                    while let Ok(msg) = eval.recv::<serde_json::Value>().await {
                        match msg["type"].as_str() {
                            Some("ratio") => {
                                if let Some(v) = msg["value"].as_f64() {
                                    tabs_state.write().set_active_tab_ratio(&sid, v);
                                }
                            }
                            _ => break,
                        }
                    }
                });
            },
        }
    }
}

fn build_drag_js(cid: &str, is_h: bool, p_start: f64, p_size: f64) -> String {
    format!(
        r#"(async function(dioxus) {{
            const c = document.getElementById("{cid}");
            if (!c) {{ dioxus.send({{ type: "done" }}); return; }}
            const r = c.getBoundingClientRect();
            const isH = {is_h};
            const aPos = isH ? r.x : r.y;
            const aDim = isH ? r.width : r.height;
            const pS = aPos + (aDim * {p_start}) / 100;
            const pSz = (aDim * {p_size}) / 100;
            function onMove(e) {{
                const pos = (isH ? e.clientX : e.clientY) - pS;
                const ratio = Math.max(0.1, Math.min(0.9, pos / pSz));
                dioxus.send({{ type: "ratio", value: ratio }});
            }}
            function onUp() {{
                document.removeEventListener("mousemove", onMove);
                document.removeEventListener("mouseup", onUp);
                document.body.style.cursor = "";
                document.body.style.userSelect = "";
                dioxus.send({{ type: "done" }});
            }}
            document.addEventListener("mousemove", onMove);
            document.addEventListener("mouseup", onUp);
            document.body.style.cursor = isH ? "col-resize" : "row-resize";
            document.body.style.userSelect = "none";
            await dioxus.recv();
        }})(dioxus)"#,
        cid = cid,
        is_h = is_h,
        p_start = p_start,
        p_size = p_size,
    )
}
