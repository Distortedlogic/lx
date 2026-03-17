use std::sync::Arc;

use dioxus::prelude::*;

use crate::components::pane_manager::Layout;
use crate::event::EventBus;
use crate::langfuse::LangfuseClient;
use crate::runner::ProgramRunner;

#[derive(Props, Clone, PartialEq)]
pub struct ToolbarProps {
    pub bus: Arc<EventBus>,
    pub langfuse: Arc<LangfuseClient>,
    pub layout: Signal<Layout>,
    pub running: Signal<bool>,
}

#[component]
pub fn Toolbar(props: ToolbarProps) -> Element {
    let mut source_path: Signal<String> = use_signal(String::new);
    let langfuse_enabled = props.langfuse.is_enabled();

    let run_handler = {
        let bus = props.bus.clone();
        let langfuse = props.langfuse.clone();
        let mut running = props.running;
        move |_| {
            let path = source_path.read().clone();
            if path.is_empty() || *running.read() {
                return;
            }
            running.set(true);
            let bus = bus.clone();
            let langfuse = langfuse.clone();
            tokio::spawn(async move {
                let runner = ProgramRunner::new(bus, langfuse);
                let _ = runner.run(&path).await;
                running.set(false);
            });
        }
    };

    let mut layout = props.layout;

    rsx! {
        div {
            class: "toolbar",
            div {
                class: "toolbar-section",
                input {
                    class: "file-input",
                    r#type: "text",
                    placeholder: "path/to/program.lx",
                    value: "{source_path}",
                    oninput: move |evt| source_path.set(evt.value().clone()),
                }
                button {
                    class: "btn btn-run",
                    disabled: *props.running.read(),
                    onclick: run_handler,
                    if *props.running.read() { "Running..." } else { "Run" }
                }
            }
            div {
                class: "toolbar-section",
                button {
                    class: "btn btn-layout",
                    onclick: move |_| layout.set(Layout::Single),
                    "1-col"
                }
                button {
                    class: "btn btn-layout",
                    onclick: move |_| layout.set(Layout::TwoColumn),
                    "2-col"
                }
                button {
                    class: "btn btn-layout",
                    onclick: move |_| layout.set(Layout::ThreeColumn),
                    "3-col"
                }
                button {
                    class: "btn btn-layout",
                    onclick: move |_| layout.set(Layout::Grid),
                    "Grid"
                }
            }
            div {
                class: "toolbar-section toolbar-status",
                span {
                    class: if langfuse_enabled { "langfuse-on" } else { "langfuse-off" },
                    if langfuse_enabled { "Langfuse: connected" } else { "Langfuse: off" }
                }
            }
        }
    }
}
