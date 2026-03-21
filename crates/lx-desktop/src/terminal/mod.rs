pub mod tab_bar;
pub mod toolbar;
pub mod view;

use dioxus::prelude::*;
use lx_ui::pane_tree::PaneNode;
use lx_ui::tab_state::{TabsState, TerminalTab};

pub fn use_provide_tabs() -> Signal<TabsState> {
    use_context_provider(|| Signal::new(TabsState::default()))
}

pub fn use_tabs_state() -> Signal<TabsState> {
    use_context()
}

pub fn add_tab(mut state: Signal<TabsState>, id: String, title: String, root: PaneNode) {
    let tab = TerminalTab { id, title, root };
    state.write().add_tab(tab);
}

pub fn add_terminal_tab(
    state: Signal<TabsState>,
    id: String,
    title: String,
    working_dir: String,
    command: Option<String>,
) {
    add_tab(
        state,
        id.clone(),
        title,
        PaneNode::Terminal {
            id,
            working_dir,
            command,
        },
    );
}
