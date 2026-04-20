use dioxus::prelude::*;

use crate::runtime::{tool_status_label, use_desktop_runtime};

#[component]
pub fn PiToolActivity(agent_id: String) -> Element {
  let runtime = use_desktop_runtime();
  let tools = runtime.registry.tools_for_agent(&agent_id);

  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)]/30 bg-[var(--surface-container-low)] p-3",
      div { class: "mb-3 text-xs font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
        "Tool Activity"
      }
      if tools.is_empty() {
        p { class: "text-sm text-[var(--outline)]", "No tool activity yet." }
      } else {
        div { class: "space-y-2 max-h-[18rem] overflow-y-auto",
          for tool in tools {
            div {
              key: "{tool.call_id}",
              class: "rounded-lg border border-[var(--outline-variant)]/20 bg-[var(--surface-container)] p-3",
              div { class: "flex items-center justify-between gap-3",
                div {
                  div { class: "text-sm font-medium text-[var(--on-surface)]",
                    "{tool.tool_name}"
                  }
                  div { class: "text-xs text-[var(--outline)] font-mono",
                    "{tool.call_id}"
                  }
                }
                span { class: tool_badge_class(tool_status_label(&tool.status)),
                  "{tool_status_label(&tool.status)}"
                }
              }
              if let Some(args) = serde_json::to_string_pretty(&tool.args).ok() {
                if tool.args != serde_json::Value::Null {
                  pre { class: "mt-2 text-xs text-[var(--outline)] whitespace-pre-wrap",
                    "{args}"
                  }
                }
              }
              if let Some(preview) = tool.result_preview {
                p { class: "mt-2 text-sm text-[var(--on-surface-variant)] whitespace-pre-wrap",
                  "{preview}"
                }
              }
            }
          }
        }
      }
    }
  }
}

fn tool_badge_class(status: &str) -> &'static str {
  match status {
    "running" => "inline-flex rounded-full bg-cyan-500/10 px-2 py-1 text-[11px] font-semibold uppercase text-cyan-300",
    "completed" => "inline-flex rounded-full bg-green-500/10 px-2 py-1 text-[11px] font-semibold uppercase text-green-300",
    _ => "inline-flex rounded-full bg-red-500/10 px-2 py-1 text-[11px] font-semibold uppercase text-red-300",
  }
}
