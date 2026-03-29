use dioxus::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardingInitialStep {
  Company,
  Agent,
}

impl OnboardingInitialStep {
  pub fn index(&self) -> u8 {
    match self {
      Self::Company => 1,
      Self::Agent => 2,
    }
  }
}

#[derive(Clone, Debug, Default)]
pub struct OnboardingOptions {
  pub initial_step: Option<OnboardingInitialStep>,
  pub company_id: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
pub struct OnboardingCtx {
  pub open: Signal<bool>,
  pub options: Signal<OnboardingOptions>,
}

impl OnboardingCtx {
  pub fn provide() -> Self {
    let ctx = Self { open: Signal::new(false), options: Signal::new(OnboardingOptions::default()) };
    use_context_provider(|| ctx);
    ctx
  }

  pub fn open_wizard(&self, opts: OnboardingOptions) {
    let mut options = self.options;
    let mut open = self.open;
    options.set(opts);
    open.set(true);
  }

  pub fn close_wizard(&self) {
    let mut open = self.open;
    let mut options = self.options;
    open.set(false);
    options.set(OnboardingOptions::default());
  }
}
