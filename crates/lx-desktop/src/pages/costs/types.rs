use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetPolicy {
  pub id: String,
  pub scope_type: String,
  pub scope_id: String,
  pub scope_name: String,
  pub amount_cents: u64,
  pub observed_cents: u64,
  pub warn_percent: u32,
  pub hard_stop: bool,
  pub status: String,
  pub paused: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProviderSpend {
  pub provider: String,
  pub model: String,
  pub input_tokens: u64,
  pub output_tokens: u64,
  pub cost_cents: u64,
}

pub fn format_cents(cents: u64) -> String {
  let dollars = cents / 100;
  let remainder = cents % 100;
  format!("${dollars}.{remainder:02}")
}

pub fn format_tokens(tokens: u64) -> String {
  if tokens >= 1_000_000 {
    format!("{:.1}M", tokens as f64 / 1_000_000.0)
  } else if tokens >= 1_000 {
    format!("{:.1}k", tokens as f64 / 1_000.0)
  } else {
    format!("{tokens}")
  }
}

pub fn utilization_percent(observed: u64, budget: u64) -> u32 {
  if budget == 0 {
    return 0;
  }
  ((observed as f64 / budget as f64) * 100.0).min(100.0) as u32
}
