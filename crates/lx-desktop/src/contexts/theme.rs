use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
  Light,
  #[default]
  Dark,
}

#[derive(Clone, Copy)]
pub struct ThemeState {
  pub theme: Signal<Theme>,
}

impl ThemeState {
  pub fn provide() -> Self {
    let state = Self { theme: Signal::new(Theme::Dark) };
    use_context_provider(|| state);
    state
  }

  pub fn current(&self) -> Theme {
    *self.theme.read()
  }

  pub fn set(&self, theme: Theme) {
    let mut sig = self.theme;
    sig.set(theme);
  }

  pub fn toggle(&self) {
    let mut sig = self.theme;
    let next = match *sig.read() {
      Theme::Dark => Theme::Light,
      Theme::Light => Theme::Dark,
    };
    sig.set(next);
  }

  pub fn is_dark(&self) -> bool {
    *self.theme.read() == Theme::Dark
  }
}
