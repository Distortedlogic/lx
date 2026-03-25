use super::voice_context::{VoiceContext, VoiceStatus};
use crate::terminal::status_badge::{BadgeVariant, StatusBadge};
use dioxus::prelude::*;

#[component]
pub fn AgentCard() -> Element {
  let ctx = use_context::<VoiceContext>();
  let status = (ctx.status)();
  let is_active = status != VoiceStatus::Idle;
  let stage = (ctx.pipeline_stage)();
  let entries = ctx.transcript.read();
  let turn_count = entries.iter().filter(|e| e.is_user).count();
  let border_class = if is_active { "border border-[var(--primary)]/60" } else { "border border-[var(--primary)]/30" };

  let (badge_variant, badge_label) =
    if status == VoiceStatus::Idle { (BadgeVariant::Idle, "IDLE".to_string()) } else { (BadgeVariant::Active, status.to_string()) };

  rsx! {
    div { class: "bg-[var(--surface-container)] rounded-lg p-4 {border_class}",
      div { class: "flex items-center gap-3 mb-3",
        span { class: "text-[var(--primary)]", "\u{25CF}" }
        span { class: "font-semibold uppercase text-sm tracking-wider text-[var(--on-surface)]",
          "VOICE_AGENT"
        }
        StatusBadge { label: badge_label, variant: badge_variant }
      }
      if is_active {
        div { class: "flex gap-4 text-xs mb-3",
          div {
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
              "PIPELINE_STAGE"
            }
            p { class: "text-[var(--on-surface-variant)] uppercase", "{stage}" }
          }
          div {
            p { class: "text-[10px] uppercase tracking-wider text-[var(--outline)] mb-1",
              "TURNS"
            }
            p { class: "text-[var(--on-surface-variant)]", "{turn_count}" }
          }
        }
      }
      if is_active {
        div { class: "mt-3",
          button {
            class: "w-full border border-[var(--outline)] text-[var(--on-surface)] rounded py-2 text-xs uppercase tracking-wider hover:bg-[var(--surface-container-high)] transition-colors duration-150",
            onclick: move |_| {
                if let Some(w) = (ctx.widget)() {
                    w.send_update(serde_json::json!({ "type" : "stop_capture" }));
                }
            },
            "TERMINATE"
          }
        }
      }
    }
  }
}
