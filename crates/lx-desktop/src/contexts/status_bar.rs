use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct StatusBarState {
  pub branch: Signal<String>,
  pub line: Signal<u32>,
  pub col: Signal<u32>,
  pub encoding: Signal<String>,
  pub notification_count: Signal<usize>,
  pub pane_label: Signal<String>,
}

impl StatusBarState {
  pub fn provide() -> Self {
    let ctx = Self {
      branch: Signal::new("main".into()),
      line: Signal::new(1),
      col: Signal::new(1),
      encoding: Signal::new("UTF-8".into()),
      notification_count: Signal::new(0),
      pane_label: Signal::new("READY".into()),
    };
    use_context_provider(|| ctx);
    ctx
  }

  pub fn update_cursor(&self, line: u32, col: u32) {
    let mut line_sig = self.line;
    line_sig.set(line);
    let mut col_sig = self.col;
    col_sig.set(col);
  }
}
