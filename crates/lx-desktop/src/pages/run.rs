use std::sync::Arc;

use dioxus::prelude::*;
use lx_dx::event::EventBus;
use lx_ui::components::PageHeader;

use crate::server::lx::{LxRunState, RunStatus, start_run};

#[component]
pub fn Run() -> Element {
    let bus: Signal<Arc<EventBus>> = use_context();
    let mut file_path = use_signal(String::new);
    let mut run_state = use_signal(|| LxRunState::new(bus.read().clone()));

    let status = run_state.read().status.clone();
    let status_text = match &status {
        RunStatus::Idle => "idle".to_string(),
        RunStatus::Running => "running".to_string(),
        RunStatus::Completed { duration_ms } => format!("completed in {duration_ms}ms"),
        RunStatus::Failed { error, duration_ms } => format!("failed ({duration_ms}ms): {error}"),
    };

    let is_running = matches!(status, RunStatus::Running);

    rsx! {
        PageHeader { title: "Run".to_string(), subtitle: Some("Execute an lx program".to_string()) }
        div { class: "p-4 space-y-4",
            div { class: "flex gap-2",
                input {
                    r#type: "text",
                    class: "flex-1 bg-gray-800 border border-gray-600 rounded px-3 py-2 text-sm text-gray-100",
                    placeholder: "path/to/program.lx",
                    value: "{file_path}",
                    oninput: move |e| file_path.set(e.value()),
                }
                button {
                    class: "px-4 py-2 bg-blue-600 text-white rounded text-sm hover:bg-blue-500 disabled:opacity-50",
                    disabled: is_running || file_path.read().is_empty(),
                    onclick: move |_| {
                        let path = file_path.read().clone();
                        let mut state = run_state.write();
                        start_run(&mut state, path);
                    },
                    "Run"
                }
            }
            div { class: "text-sm text-gray-400",
                "Status: {status_text}"
            }
        }
    }
}
