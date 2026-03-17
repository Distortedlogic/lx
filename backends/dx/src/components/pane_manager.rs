use std::sync::Arc;

use dioxus::prelude::*;

use crate::components::pane::Pane;
use crate::event::{EventBus, RuntimeEvent};

#[derive(Clone, PartialEq)]
pub enum Layout {
    Single,
    TwoColumn,
    ThreeColumn,
    Grid,
}

#[derive(Props, Clone, PartialEq)]
pub struct PaneManagerProps {
    pub bus: Arc<EventBus>,
    pub layout: Signal<Layout>,
}

#[component]
pub fn PaneManager(props: PaneManagerProps) -> Element {
    let mut agent_ids: Signal<Vec<String>> = use_signal(|| vec!["main".to_string()]);
    let bus = props.bus.clone();

    use_future(move || {
        let bus = bus.clone();
        async move {
            let mut rx = bus.subscribe();
            loop {
                match rx.recv().await {
                    Ok(RuntimeEvent::AgentSpawned { agent_id, .. }) => {
                        let mut ids = agent_ids.write();
                        if !ids.contains(&agent_id) {
                            ids.push(agent_id);
                        }
                    }
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    });

    let layout = props.layout.read().clone();
    let ids = agent_ids.read().clone();
    let grid_class = match layout {
        Layout::Single => "pane-grid single",
        Layout::TwoColumn => "pane-grid two-col",
        Layout::ThreeColumn => "pane-grid three-col",
        Layout::Grid => "pane-grid grid",
    };

    rsx! {
        div {
            class: "{grid_class}",
            for id in ids.iter() {
                Pane {
                    key: "{id}",
                    agent_id: id.clone(),
                    bus: props.bus.clone(),
                }
            }
        }
    }
}
