pub mod browser_view;
pub mod status_badge;
pub mod tab_bar;
pub mod toolbar;
pub mod view;

use dioxus::prelude::*;
use pane_tree::{PaneNode, Tab, TabsState};

use crate::panes::DesktopPane;

pub fn use_provide_tabs() -> Signal<TabsState<DesktopPane>> {
  use_context_provider(|| Signal::new(TabsState { tabs: Vec::new(), active_tab_id: None, focused_pane_id: None, notifications: Default::default() }))
}

pub fn use_tabs_state() -> Signal<TabsState<DesktopPane>> {
  use_context()
}

pub fn add_tab(mut state: Signal<TabsState<DesktopPane>>, id: String, title: String, root: PaneNode<DesktopPane>) {
  let mut tab = Tab::with_root("", root);
  tab.id = id;
  tab.title = title;
  state.write().add_tab(tab);
}

pub fn add_terminal_tab(state: Signal<TabsState<DesktopPane>>, id: String, title: String, working_dir: String, command: Option<String>) {
  let pane = DesktopPane::Terminal { id: id.clone(), working_dir, command, name: None };
  add_tab(state, id, title, PaneNode::Leaf(pane));
}
