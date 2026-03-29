use dioxus::prelude::*;

use crate::styles::{FLEX_BETWEEN, PAGE_HEADING};

use super::accounting_card::AccountingModelCard;
use super::budget_card::BudgetCard;
use super::provider_card::ProviderCard;
use super::types::{BudgetPolicy, ProviderSpend, format_cents};

fn default_budget_policies() -> Vec<BudgetPolicy> {
  vec![
    BudgetPolicy {
      id: "bp-1".into(),
      scope_type: "COMPANY".into(),
      scope_id: "co-1".into(),
      scope_name: "Company".into(),
      amount_cents: 10000,
      observed_cents: 4200,
      warn_percent: 80,
      hard_stop: true,
      status: "ok".into(),
      paused: false,
    },
    BudgetPolicy {
      id: "bp-2".into(),
      scope_type: "PROJECT".into(),
      scope_id: "proj-1".into(),
      scope_name: "Project Alpha".into(),
      amount_cents: 5000,
      observed_cents: 4500,
      warn_percent: 75,
      hard_stop: false,
      status: "warning".into(),
      paused: false,
    },
    BudgetPolicy {
      id: "bp-3".into(),
      scope_type: "AGENT".into(),
      scope_id: "agent-1".into(),
      scope_name: "Code Review Bot".into(),
      amount_cents: 2000,
      observed_cents: 2000,
      warn_percent: 90,
      hard_stop: true,
      status: "hard_stop".into(),
      paused: true,
    },
  ]
}

fn default_provider_spend() -> Vec<ProviderSpend> {
  vec![
    ProviderSpend { provider: "Anthropic".into(), model: "claude-sonnet-4-20250514".into(), input_tokens: 1_250_000, output_tokens: 320_000, cost_cents: 2400 },
    ProviderSpend { provider: "Anthropic".into(), model: "claude-haiku-4-20250514".into(), input_tokens: 800_000, output_tokens: 150_000, cost_cents: 600 },
    ProviderSpend { provider: "OpenAI".into(), model: "gpt-4o".into(), input_tokens: 500_000, output_tokens: 120_000, cost_cents: 1800 },
    ProviderSpend { provider: "OpenAI".into(), model: "gpt-4o-mini".into(), input_tokens: 2_000_000, output_tokens: 400_000, cost_cents: 400 },
  ]
}

#[component]
pub fn Costs() -> Element {
  let budgets = dioxus_storage::use_persistent("lx_budget_policies", default_budget_policies);
  let spend = dioxus_storage::use_persistent("lx_provider_spend", default_provider_spend);
  let mut active_tab = use_signal(|| "overview");

  let spend_rows = spend();
  let total_cost: u64 = spend_rows.iter().map(|r| r.cost_cents).sum();
  let unique_providers: Vec<String> = {
    let mut ps: Vec<String> = spend_rows.iter().map(|r| r.provider.clone()).collect();
    ps.sort();
    ps.dedup();
    ps
  };
  let provider_count = unique_providers.len();
  let model_count = spend_rows.len();

  rsx! {
    div { class: "flex flex-col h-full p-4 overflow-auto gap-4",
      div { class: FLEX_BETWEEN,
        h1 { class: PAGE_HEADING, "COSTS" }
      }
      div { class: "flex gap-2",
        TabButton {
          label: "OVERVIEW",
          active: active_tab() == "overview",
          onclick: move |_| active_tab.set("overview"),
        }
        TabButton {
          label: "BUDGETS",
          active: active_tab() == "budgets",
          onclick: move |_| active_tab.set("budgets"),
        }
        TabButton {
          label: "PROVIDERS",
          active: active_tab() == "providers",
          onclick: move |_| active_tab.set("providers"),
        }
      }
      if active_tab() == "overview" {
        div { class: "grid grid-cols-3 gap-4",
          MetricBox { label: "TOTAL SPEND", value: format_cents(total_cost) }
          MetricBox { label: "PROVIDERS", value: provider_count.to_string() }
          MetricBox { label: "MODELS", value: model_count.to_string() }
        }
        AccountingModelCard {}
      }
      if active_tab() == "budgets" {
        div { class: "flex flex-col gap-4",
          for (idx , policy) in budgets().iter().enumerate() {
            BudgetCard {
              key: "{policy.id}",
              policy: policy.clone(),
              on_save: move |new_cents: u64| {
                  let mut b = budgets;
                  let mut all = b();
                  if let Some(p) = all.get_mut(idx) {
                      p.amount_cents = new_cents;
                      if new_cents == 0 {
                          p.status = "ok".into();
                      } else if p.observed_cents >= new_cents {
                          p.status = "hard_stop".into();
                      } else if p.observed_cents * 100 >= new_cents * p.warn_percent as u64 {
                          p.status = "warning".into();
                      } else {
                          p.status = "ok".into();
                      }
                      if p.status != "hard_stop" {
                          p.paused = false;
                      }
                  }
                  b.set(all);
              },
            }
          }
        }
      }
      if active_tab() == "providers" {
        div { class: "flex flex-col gap-4",
          for provider_name in unique_providers.iter() {
            {
                let rows: Vec<ProviderSpend> = spend_rows
                    .iter()
                    .filter(|r| r.provider == *provider_name)
                    .cloned()
                    .collect();
                rsx! {
                  ProviderCard { key: "{provider_name}", provider: provider_name.clone(), rows }
                }
            }
          }
        }
      }
    }
  }
}

#[component]
fn TabButton(label: &'static str, active: bool, onclick: EventHandler<MouseEvent>) -> Element {
  let cls = if active {
    "px-4 py-2 text-xs font-semibold uppercase tracking-wider rounded bg-[var(--primary)] text-[var(--on-primary)]"
  } else {
    "px-4 py-2 text-xs font-semibold uppercase tracking-wider rounded text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container)]"
  };
  rsx! {
    button { class: cls, onclick: move |evt| onclick.call(evt), "{label}" }
  }
}

#[component]
fn MetricBox(label: &'static str, value: String) -> Element {
  rsx! {
    div { class: "bg-[var(--surface-container-lowest)] border border-[var(--outline-variant)] rounded-lg p-4",
      p { class: "text-[10px] uppercase tracking-[0.18em] text-[var(--outline)]",
        "{label}"
      }
      p { class: "text-2xl font-semibold text-[var(--on-surface)] mt-1", "{value}" }
    }
  }
}
