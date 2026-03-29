use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct PanelState {
  pub visible: Signal<bool>,
  pub content_id: Signal<Option<String>>,
}

impl PanelState {
  pub fn provide() -> Self {
    let state = Self { visible: Signal::new(true), content_id: Signal::new(None) };
    use_context_provider(|| state);
    state
  }

  pub fn open(&self, id: String) {
    let mut c = self.content_id;
    c.set(Some(id));
  }

  pub fn close(&self) {
    let mut c = self.content_id;
    c.set(None);
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
    self.content_id.read().is_some()
  }
}
