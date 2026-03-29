use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatRun {
  pub id: String,
  pub agent_id: String,
  pub company_id: String,
  pub status: String,
  pub invocation_source: String,
  pub trigger_detail: Option<String>,
  pub started_at: Option<String>,
  pub finished_at: Option<String>,
  pub created_at: String,
  pub error: Option<String>,
  pub error_code: Option<String>,
  pub usage_json: Option<serde_json::Value>,
  pub result_json: Option<serde_json::Value>,
  pub context_snapshot: Option<serde_json::Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillEntry {
  pub key: String,
  pub name: String,
  pub description: Option<String>,
  pub detail: Option<String>,
  pub required: bool,
  pub location_label: Option<String>,
  pub origin_label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillSnapshot {
  pub entries: Vec<SkillEntry>,
  pub desired_skills: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BudgetSummary {
  pub amount: i64,
  pub observed_amount: i64,
  pub remaining_amount: i64,
  pub utilization_percent: f64,
  pub warn_percent: u32,
  pub hard_stop_enabled: bool,
  pub status: String,
  pub is_active: bool,
}

#[derive(Clone, PartialEq)]
pub struct RunMetrics {
  pub input_tokens: u64,
  pub output_tokens: u64,
  pub cached_tokens: u64,
  pub cost_usd: f64,
  pub total_tokens: u64,
}

pub fn run_metrics(run: &HeartbeatRun) -> RunMetrics {
  let usage = run.usage_json.as_ref();
  let result = run.result_json.as_ref();

  fn get_u64(val: Option<&serde_json::Value>, keys: &[&str]) -> u64 {
    let Some(obj) = val.and_then(|v| v.as_object()) else {
      return 0;
    };
    for key in keys {
      if let Some(n) = obj.get(*key).and_then(|v| v.as_u64()) {
        return n;
      }
    }
    0
  }

  fn get_f64(val: Option<&serde_json::Value>, keys: &[&str]) -> f64 {
    let Some(obj) = val.and_then(|v| v.as_object()) else {
      return 0.0;
    };
    for key in keys {
      if let Some(n) = obj.get(*key).and_then(|v| v.as_f64()) {
        return n;
      }
    }
    0.0
  }

  let input = get_u64(usage, &["inputTokens", "input_tokens"]);
  let output = get_u64(usage, &["outputTokens", "output_tokens"]);
  let cached = get_u64(usage, &["cachedInputTokens", "cached_input_tokens", "cache_read_input_tokens"]);
  let cost = get_f64(usage, &["totalCostUsd", "total_cost_usd"]).max(get_f64(result, &["totalCostUsd", "total_cost_usd"]));

  RunMetrics { input_tokens: input, output_tokens: output, cached_tokens: cached, cost_usd: cost, total_tokens: input + output }
}

pub fn source_label(source: &str) -> &str {
  match source {
    "timer" => "Timer",
    "assignment" => "Assignment",
    "on_demand" => "On-demand",
    "automation" => "Automation",
    other => other,
  }
}

pub fn run_status_class(status: &str) -> &'static str {
  match status {
    "succeeded" => "text-green-600",
    "failed" => "text-red-600",
    "running" => "text-cyan-600",
    "queued" => "text-yellow-600",
    "timed_out" => "text-orange-600",
    "cancelled" => "text-neutral-500",
    _ => "text-neutral-400",
  }
}

pub fn format_tokens(tokens: u64) -> String {
  if tokens >= 1_000_000 {
    format!("{:.1}M", tokens as f64 / 1_000_000.0)
  } else if tokens >= 1_000 {
    format!("{:.1}K", tokens as f64 / 1_000.0)
  } else {
    tokens.to_string()
  }
}
