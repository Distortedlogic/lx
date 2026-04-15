use dioxus::prelude::*;

use super::run_types::BudgetSummary;
#[component]
pub fn BudgetTab(summary: BudgetSummary, on_save: EventHandler<i64>) -> Element {
  let mut draft_dollars = use_signal(|| format!("{:.2}", summary.amount as f64 / 100.0));
  let parsed = parse_dollar_input(&draft_dollars.read());
  let can_save = parsed.is_some() && parsed != Some(summary.amount);

  let progress = if summary.amount > 0 { (summary.utilization_percent).min(100.0) } else { 0.0 };
  let status_tone = match summary.status.as_str() {
    "hard_stop" => "text-red-400 border-red-500/30 bg-red-500/10",
    "warning" => "text-amber-300 border-amber-500/30 bg-amber-500/10",
    _ => "text-emerald-300 border-emerald-500/30 bg-emerald-500/10",
  };

  rsx! {
    div { class: "max-w-3xl space-y-6",
      div {
        class: "inline-flex items-center gap-2 border rounded-full px-3 py-1 text-xs font-medium",
        class: "{status_tone}",
        "{summary.status}"
      }
      div { class: "grid gap-6 sm:grid-cols-2",
        div {
          div { class: "text-[11px] uppercase tracking-widest text-[var(--outline)]",
            "Observed"
          }
          div { class: "mt-2 text-xl font-semibold text-[var(--on-surface)] tabular-nums",
            "{format_cents(summary.observed_amount)}"
          }
          div { class: "mt-1 text-xs text-[var(--outline)]",
            if summary.amount > 0 {
              "{summary.utilization_percent:.0}% of limit"
            } else {
              "No cap configured"
            }
          }
        }
        div {
          div { class: "text-[11px] uppercase tracking-widest text-[var(--outline)]",
            "Budget"
          }
          div { class: "mt-2 text-xl font-semibold text-[var(--on-surface)] tabular-nums",
            if summary.amount > 0 {
              "{format_cents(summary.amount)}"
            } else {
              "Disabled"
            }
          }
          div { class: "mt-1 text-xs text-[var(--outline)]",
            "Soft alert at {summary.warn_percent}%"
          }
        }
      }
      if summary.amount > 0 {
        div { class: "w-full bg-[var(--surface-container-high)] rounded-full h-2",
          div {
            class: "h-2 rounded-full transition-all bg-[var(--primary)]",
            style: "width: {progress}%",
          }
        }
      }
      div { class: "space-y-3",
        h3 { class: "text-sm font-medium text-[var(--on-surface)]", "Set Monthly Budget" }
        div { class: "flex items-center gap-2",
          span { class: "text-sm text-[var(--outline)]", "$" }
          input {
            class: "input-field",
            r#type: "number",
            step: "0.01",
            min: "0",
            placeholder: "0.00",
            value: "{draft_dollars}",
            oninput: move |evt| draft_dollars.set(evt.value().to_string()),
          }
          if can_save {
            button {
              class: "btn-primary-sm",
              onclick: move |_| {
                  if let Some(cents) = parse_dollar_input(&draft_dollars.read()) {
                      on_save.call(cents);
                  }
              },
              "Save"
            }
          }
        }
      }
    }
  }
}

fn format_cents(cents: i64) -> String {
  if cents == 0 { "$0.00".to_string() } else { format!("${:.2}", cents as f64 / 100.0) }
}

fn parse_dollar_input(value: &str) -> Option<i64> {
  let trimmed = value.trim();
  if trimmed.is_empty() {
    return Some(0);
  }
  let parsed: f64 = trimmed.parse().ok()?;
  if parsed < 0.0 || !parsed.is_finite() {
    return None;
  }
  Some((parsed * 100.0).round() as i64)
}
