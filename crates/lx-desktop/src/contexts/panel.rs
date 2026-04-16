use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PanelContent {
  FlowNode { node_id: String },
  FlowEdge { edge_id: String },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct PanelState {
  pub visible: Signal<bool>,
  pub content: Signal<Option<PanelContent>>,
}

impl PanelState {
  pub fn provide() -> Self {
    let state = Self { visible: Signal::new(true), content: Signal::new(None) };
    use_context_provider(|| state);
    state
  }

  pub fn open(&self, content: PanelContent) {
    let mut current = self.content;
    current.set(Some(content));
    self.set_visible(true);
  }

  pub fn close(&self) {
    let mut current = self.content;
    current.set(None);
  }

  pub fn set_visible(&self, v: bool) {
    let mut vis = self.visible;
    vis.set(v);
  }

  pub fn toggle_visible(&self) {
    let mut vis = self.visible;
    let current = *vis.read();
    vis.set(!current);
  }

  pub fn is_visible(&self) -> bool {
    *self.visible.read()
  }

  pub fn has_content(&self) -> bool {
    self.content.read().is_some()
  }
}
