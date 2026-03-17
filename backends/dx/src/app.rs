use std::sync::Arc;

use dioxus::prelude::*;

use crate::components::pane_manager::{Layout, PaneManager};
use crate::components::toolbar::Toolbar;
use crate::event::EventBus;
use crate::langfuse::LangfuseClient;

#[component]
pub fn App() -> Element {
    let bus = use_signal(|| Arc::new(EventBus::new()));
    let langfuse = use_signal(|| Arc::new(LangfuseClient::from_env()));
    let layout = use_signal(|| Layout::TwoColumn);
    let running = use_signal(|| false);

    rsx! {
        style { {include_str!("../assets/style.css")} }
        div {
            class: "lx-dx",
            Toolbar {
                bus: bus.read().clone(),
                langfuse: langfuse.read().clone(),
                layout,
                running,
            }
            PaneManager {
                bus: bus.read().clone(),
                layout,
            }
        }
    }
}
