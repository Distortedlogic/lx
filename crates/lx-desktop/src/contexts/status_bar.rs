use dioxus::prelude::*;

#[derive(Store, Clone, PartialEq)]
pub struct StatusBarState {
  pub branch: String,
  pub line: u32,
  pub col: u32,
  pub encoding: String,
  pub notification_count: usize,
  pub pane_label: String,
}

#[store(pub(crate))]
impl<Lens> Store<StatusBarState, Lens> {
  fn update_cursor(&mut self, line: u32, col: u32) {
    self.line().set(line);
    self.col().set(col);
  }
}
