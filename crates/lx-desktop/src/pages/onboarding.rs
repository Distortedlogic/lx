use dioxus::prelude::*;

use crate::contexts::onboarding::OnboardingCtx;

#[component]
pub fn Onboarding() -> Element {
  let onboarding = use_context::<OnboardingCtx>();
  use_effect(move || {
    onboarding.open_wizard(crate::contexts::onboarding::OnboardingOptions::default());
  });
  rsx! {
    crate::pages::agents::Agents {}
  }
}
