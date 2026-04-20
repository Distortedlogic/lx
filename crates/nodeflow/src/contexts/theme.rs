use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Theme {
  Light,
  #[default]
  Dark,
}

impl Theme {
  pub fn css_class(&self) -> &'static str {
    match self {
      Theme::Dark => "dark",
      Theme::Light => "light",
    }
  }
}

#[derive(Clone, Copy)]
pub struct ThemeState {
  pub theme: Signal<Theme>,
}

impl ThemeState {
  pub fn provide() -> Self {
    let state = Self { theme: Signal::new(Theme::Dark) };
    use_context_provider(|| state);
    apply_theme_class(Theme::Dark);
    state
  }

  pub fn current(&self) -> Theme {
    *self.theme.read()
  }

  pub fn set(&self, theme: Theme) {
    let mut sig = self.theme;
    sig.set(theme);
    apply_theme_class(theme);
  }

  pub fn toggle(&self) {
    let mut sig = self.theme;
    let next = match *sig.read() {
      Theme::Dark => Theme::Light,
      Theme::Light => Theme::Dark,
    };
    sig.set(next);
    apply_theme_class(next);
  }

  pub fn is_dark(&self) -> bool {
    *self.theme.read() == Theme::Dark
  }
}

fn apply_theme_class(theme: Theme) {
  let class = theme.css_class();
  let remove = match theme {
    Theme::Dark => "light",
    Theme::Light => "dark",
  };
  let js = format!("document.documentElement.classList.remove('{remove}'); document.documentElement.classList.add('{class}');");
  spawn(async move {
    let _ = document::eval(&js).await;
  });
}
