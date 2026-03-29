use dioxus::prelude::*;

use super::types::{ProviderSpend, format_cents, format_tokens};

#[component]
pub fn ProviderCard(provider: String, rows: Vec<ProviderSpend>) -> Element {
  let total_input: u64 = rows.iter().map(|r| r.input_tokens).sum();
  let total_output: u64 = rows.iter().map(|r| r.output_tokens).sum();
  let total_cost: u64 = rows.iter().map(|r| r.cost_cents).sum();
  let total_tokens = total_input + total_output;

  rsx! {
    div { class: "bg-[var(--surface-container-lowest)] border-2 border-[var(--outline-variant)] p-4 rounded-lg space-y-3",
      div { class: "flex items-center justify-between",
        p { class: "text-lg font-bold text-[var(--on-surface)]", "{provider}" }
        p { class: "text-xl font-semibold text-[var(--on-surface)]",
          "{format_cents(total_cost)}"
        }
      }
      p { class: "text-xs text-[var(--outline)]",
        "{format_tokens(total_input)} in / {format_tokens(total_output)} out"
      }
      div { class: "border-t border-[var(--outline-variant)] pt-3 space-y-3",
        for row in rows.iter() {
          {
              let row_tokens = row.input_tokens + row.output_tokens;
              let share = if total_tokens > 0 {
                  (row_tokens as f64 / total_tokens as f64 * 100.0) as u32
              } else {
                  0
              };
              rsx! {
                div { class: "space-y-1",
                  div { class: "flex items-center justify-between",
                    p { class: "text-xs font-mono text-[var(--outline)]", "{row.model}" }
                    p { class: "text-xs text-[var(--on-surface)]",
                      "{format_tokens(row_tokens)} tokens \u{00b7} {format_cents(row.cost_cents)}"
                    }
                  }
                  div { class: "h-2 rounded-full bg-[var(--outline-variant)] overflow-hidden",
                    div {
                      class: "h-full rounded-full bg-[var(--primary)]",
                      style: "width: {share}%",
                    }
                  }
                }
              }
          }
        }
      }
    }
  }
}
