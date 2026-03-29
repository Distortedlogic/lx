use dioxus::prelude::*;

use super::types::{BudgetPolicy, format_cents, utilization_percent};

#[component]
pub fn BudgetCard(policy: BudgetPolicy, on_save: Option<EventHandler<u64>>) -> Element {
  let mut draft_budget = use_signal(|| format!("{:.2}", policy.amount_cents as f64 / 100.0));
  let util = utilization_percent(policy.observed_cents, policy.amount_cents);
  let remaining = policy.amount_cents.saturating_sub(policy.observed_cents);

  let fill_color = if policy.status == "hard_stop" {
    "bg-red-500"
  } else if policy.status == "warning" {
    "bg-amber-500"
  } else {
    "bg-green-500"
  };

  let status_badge = if policy.status == "hard_stop" {
    rsx! {
      span { class: "flex items-center gap-1 text-red-500 text-xs font-semibold",
        span { class: "material-symbols-outlined text-sm", "gpp_maybe" }
        "HARD STOP"
      }
    }
  } else if policy.status == "warning" {
    rsx! {
      span { class: "text-amber-500 text-xs font-semibold", "WARNING" }
    }
  } else {
    rsx! {
      span { class: "text-green-500 text-xs font-semibold", "HEALTHY" }
    }
  };

  let budget_display = if policy.amount_cents == 0 { "DISABLED".to_string() } else { format_cents(policy.amount_cents) };

  let parsed = draft_budget().trim().parse::<f64>().ok().filter(|v| *v >= 0.0);
  let parsed_cents = parsed.map(|v| (v * 100.0).round() as u64);
  let save_disabled = parsed_cents.is_none() || parsed_cents == Some(policy.amount_cents);
  let btn_label = if policy.amount_cents == 0 { "SET BUDGET" } else { "UPDATE BUDGET" };

  rsx! {
    div { class: "bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-5 rounded-lg space-y-4",
      div { class: "flex items-center justify-between",
        div {
          p { class: "text-[10px] uppercase tracking-[0.18em] text-[var(--outline)]",
            "{policy.scope_type}"
          }
          p { class: "text-lg font-bold text-[var(--on-surface)]", "{policy.scope_name}" }
        }
        {status_badge}
      }
      div { class: "grid grid-cols-2 gap-4",
        div {
          p { class: "text-[10px] uppercase tracking-[0.18em] text-[var(--outline)]",
            "OBSERVED"
          }
          p { class: "text-xl font-semibold text-[var(--on-surface)]",
            "{format_cents(policy.observed_cents)}"
          }
          p { class: "text-xs text-[var(--outline)]", "{util}% utilized" }
        }
        div {
          p { class: "text-[10px] uppercase tracking-[0.18em] text-[var(--outline)]",
            "BUDGET"
          }
          p { class: "text-xl font-semibold text-[var(--on-surface)]", "{budget_display}" }
          p { class: "text-xs text-[var(--outline)]", "Warn at {policy.warn_percent}%" }
        }
      }
      div {
        div { class: "flex items-center justify-between mb-1",
          p { class: "text-[10px] uppercase tracking-[0.18em] text-[var(--outline)]",
            "Remaining"
          }
          p { class: "text-xs text-[var(--outline)]", "{format_cents(remaining)}" }
        }
        div { class: "h-2 rounded-full bg-[var(--outline-variant)] overflow-hidden",
          div {
            class: "h-full rounded-full {fill_color}",
            style: "width: {util}%",
          }
        }
      }
      if policy.paused {
        div { class: "border border-red-500/50 rounded-lg p-3 flex items-center gap-2 text-red-400 text-xs",
          span { class: "material-symbols-outlined text-sm", "pause_circle" }
          "Execution is paused until the budget is raised"
        }
      }
      if on_save.is_some() {
        div { class: "flex items-center gap-3 pt-2",
          input {
            class: "flex-1 bg-[var(--surface-container-lowest)] border border-[var(--outline-variant)] text-xs px-3 py-2 rounded outline-none text-[var(--on-surface-variant)] placeholder-[var(--outline)]",
            inputmode: "decimal",
            placeholder: "0.00",
            value: "{draft_budget}",
            oninput: move |evt| draft_budget.set(evt.value()),
          }
          button {
            class: "bg-[var(--primary)] text-[var(--on-primary)] px-4 py-2 text-xs uppercase font-semibold rounded disabled:opacity-40",
            disabled: save_disabled,
            onclick: move |_| {
                if let Some(cents) = parsed_cents
                    && let Some(ref handler) = on_save {
                    handler.call(cents);
                }
            },
            "{btn_label}"
          }
        }
        if parsed.is_none() && !draft_budget().trim().is_empty() {
          p { class: "text-xs text-red-500 mt-1",
            "Enter a valid non-negative dollar amount"
          }
        }
      }
    }
  }
}
