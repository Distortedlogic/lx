use dioxus::prelude::*;
use lx_ui::components::PageHeader;
use lx_ui::pane_tree::{PaneNode, Rect, SplitDirection};

#[component]
pub fn Terminals() -> Element {
    let mut root = use_signal(|| PaneNode::Terminal {
        id: uuid::Uuid::new_v4().to_string(),
        working_dir: ".".to_string(),
        command: None,
    });

    let pane_rects = root.read().compute_pane_rects(Rect::default());

    rsx! {
        PageHeader { title: "Terminals".to_string() }
        div { class: "p-4",
            div { class: "flex gap-2 mb-4",
                button {
                    class: "px-3 py-1 bg-gray-700 text-sm rounded hover:bg-gray-600",
                    onclick: move |_| {
                        let current = root.read().clone();
                        if let Some(first_id) = current.first_terminal_id() {
                            let new_pane = PaneNode::Terminal {
                                id: uuid::Uuid::new_v4().to_string(),
                                working_dir: ".".to_string(),
                                command: None,
                            };
                            root.set(current.split(&first_id, SplitDirection::Horizontal, new_pane));
                        }
                    },
                    "Split H"
                }
                button {
                    class: "px-3 py-1 bg-gray-700 text-sm rounded hover:bg-gray-600",
                    onclick: move |_| {
                        let current = root.read().clone();
                        if let Some(first_id) = current.first_terminal_id() {
                            let new_pane = PaneNode::Terminal {
                                id: uuid::Uuid::new_v4().to_string(),
                                working_dir: ".".to_string(),
                                command: None,
                            };
                            root.set(current.split(&first_id, SplitDirection::Vertical, new_pane));
                        }
                    },
                    "Split V"
                }
            }
            div { class: "relative w-full h-96 bg-gray-800 rounded border border-gray-700",
                for (pane, rect) in pane_rects {
                    {render_pane(pane, rect)}
                }
            }
        }
    }
}

fn render_pane(pane: PaneNode, rect: Rect) -> Element {
    let id = pane.pane_id().unwrap_or("").to_string();
    let style = format!(
        "position:absolute;left:{}%;top:{}%;width:{}%;height:{}%;",
        rect.left, rect.top, rect.width, rect.height
    );
    rsx! {
        div {
            key: "{id}",
            style: "{style}",
            class: "border border-gray-600 bg-gray-900 p-2 text-xs text-gray-400 overflow-hidden",
            "Terminal: {id}"
        }
    }
}
