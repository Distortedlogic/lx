use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct BreadcrumbEntry {
  pub label: String,
  pub href: Option<String>,
}

#[derive(Clone, Copy)]
pub struct BreadcrumbState {
  pub crumbs: Signal<Vec<BreadcrumbEntry>>,
}

impl BreadcrumbState {
  pub fn provide() -> Self {
    let state = Self { crumbs: Signal::new(Vec::new()) };
    use_context_provider(|| state);
    state
  }

  pub fn set(&self, entries: Vec<BreadcrumbEntry>) {
    let mut c = self.crumbs;
    c.set(entries);
  }

  pub fn clear(&self) {
    let mut c = self.crumbs;
    c.set(Vec::new());
  }

  pub fn entries(&self) -> Vec<BreadcrumbEntry> {
    self.crumbs.read().clone()
  }
}
