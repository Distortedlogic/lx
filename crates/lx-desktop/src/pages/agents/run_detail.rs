use dioxus::prelude::*;

use super::list::StatusBadge;
use super::run_types::{HeartbeatRun, RunMetrics, format_tokens, run_metrics, source_label};
use super::transcript::TranscriptView;

#[component]
pub fn RunDetailPanel(run: HeartbeatRun) -> Element {
  let metrics = run_metrics(&run);
  let short_id = &run.id[..8.min(run.id.len())];
  let is_live = run.status == "running" || run.status == "queued";

  rsx! {
    div { class: "space-y-6",
      div { class: "flex items-center justify-between",
        div { class: "flex items-center gap-2",
          if is_live {
            span { class: "relative flex h-2 w-2",
              span { class: "animate-pulse absolute inline-flex h-full w-full rounded-full bg-cyan-400 opacity-75" }
              span { class: "relative inline-flex rounded-full h-2 w-2 bg-cyan-400" }
            }
          }
          span { class: "text-sm font-mono text-[var(--on-surface)]", "{short_id}" }
          StatusBadge { status: run.status.clone() }
          span { class: "text-xs text-[var(--outline)]",
            "{source_label(&run.invocation_source)}"
          }
        }
        span { class: "text-xs text-[var(--outline)]", "{run.created_at}" }
      }
      RunMetricsGrid { metrics }
      if let Some(err) = &run.error {
        div { class: "border border-red-500/20 bg-red-500/10 rounded-lg p-3",
          p { class: "text-xs text-red-600", "{err}" }
        }
      }
      TranscriptView { run_id: run.id.clone() }
    }
  }
}

#[component]
fn RunMetricsGrid(metrics: RunMetrics) -> Element {
  let cost_str = if metrics.cost_usd > 0.0 { format!("${:.4}", metrics.cost_usd) } else { "-".to_string() };
  rsx! {
    div { class: "border border-[var(--outline-variant)]/30 rounded-lg p-4",
      div { class: "grid grid-cols-2 md:grid-cols-4 gap-4",
        MetricCell {
          label: "Input tokens",
          value: format_tokens(metrics.input_tokens),
        }
        MetricCell {
          label: "Output tokens",
          value: format_tokens(metrics.output_tokens),
        }
        MetricCell {
          label: "Cached tokens",
          value: format_tokens(metrics.cached_tokens),
        }
        MetricCell { label: "Cost", value: cost_str }
      }
    }
  }
}

#[component]
fn MetricCell(label: &'static str, value: String) -> Element {
  rsx! {
    div {
      span { class: "text-xs text-[var(--outline)] block", "{label}" }
      span { class: "text-lg font-semibold text-[var(--on-surface)] tabular-nums",
        "{value}"
      }
    }
  }
}
