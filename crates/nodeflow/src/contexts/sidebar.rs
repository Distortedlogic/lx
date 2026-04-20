use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct SidebarState {
  pub open: Signal<bool>,
}

impl SidebarState {
  pub fn provide() -> Self {
    let state = Self { open: Signal::new(true) };
    use_context_provider(|| state);
    state
  }

  pub fn is_open(&self) -> bool {
    *self.open.read()
  }

  pub fn set_open(&self, v: bool) {
    let mut o = self.open;
    o.set(v);
  }

  pub fn toggle(&self) {
    let mut o = self.open;
    let current = *o.read();
    o.set(!current);
  }
}
