use dioxus::prelude::*;

#[component]
pub fn AccountingModelCard() -> Element {
  rsx! {
    div { class: "grid grid-cols-1 md:grid-cols-3 gap-4",
      LedgerSection {
        icon: "database",
        title: "Inference ledger",
        description: "Request-scoped usage and billed runs",
        bullets: vec![
            "tokens + billed dollars",
            "provider, biller, model",
            "subscription and overage aware",
        ],
      }
      LedgerSection {
        icon: "receipt_long",
        title: "Finance ledger",
        description: "Account-level charges not tied to a single request",
        bullets: vec![
                                                  "top-ups, refunds, fees",
                                                  "provisioned charges",
                                                  "credit expiries",
                                              ],
      }
      LedgerSection {
        icon: "speed",
        title: "Live quotas",
        description: "Provider windows that can stop traffic in real time",
        bullets: vec![
                                                  "provider quota windows",
                                                  "biller credit systems",
                                                  "errors surfaced directly",
                                              ],
      }
    }
  }
}

#[component]
fn LedgerSection(icon: &'static str, title: &'static str, description: &'static str, bullets: Vec<&'static str>) -> Element {
  rsx! {
    div { class: "border border-[var(--outline-variant)] rounded-lg p-4 space-y-3",
      div { class: "flex items-center gap-3",
        div { class: "w-8 h-8 rounded-full bg-[var(--surface-container)] flex items-center justify-center",
          span { class: "material-symbols-outlined text-base text-[var(--on-surface)]",
            "{icon}"
          }
        }
        p { class: "text-sm font-semibold text-[var(--on-surface)]", "{title}" }
      }
      p { class: "text-xs text-[var(--outline)]", "{description}" }
      ul { class: "space-y-1",
        for bullet in bullets.iter() {
          li { class: "text-xs text-[var(--on-surface-variant)] flex items-start gap-1.5",
            span { class: "text-[var(--outline)] mt-0.5", "\u{2022}" }
            "{bullet}"
          }
        }
      }
    }
  }
}
