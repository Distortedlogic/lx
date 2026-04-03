use super::transcript::{ToolStatus, TranscriptBlock, TranscriptDensity, TranscriptMode, summarize_tool_input};
use super::transcript_groups::{render_command_group, render_stderr_group, render_tool_group};
use crate::components::markdown_body::MarkdownBody;
use dioxus::prelude::*;

#[component]
pub fn TranscriptBlockView(block: TranscriptBlock, mode: TranscriptMode, density: TranscriptDensity) -> Element {
  let mut tool_open = use_signal(|| false);
  let cmd_group_open = use_signal(|| false);
  let tool_group_open = use_signal(|| false);
  let stderr_open = use_signal(|| false);
  let mut stdout_open = use_signal(|| true);

  if let TranscriptBlock::Tool { is_error, .. } = &block
    && *is_error
    && !tool_open()
  {
    tool_open.set(true);
  }

  match block {
    TranscriptBlock::Message { role, text, .. } => {
      let icon = if role == "assistant" { "smart_toy" } else { "person" };
      let bg = if role == "assistant" { "bg-[var(--surface-container)]" } else { "bg-[var(--surface-container-high)]" };
      rsx! {
        div { class: if density == TranscriptDensity::Compact { "flex gap-3 p-2 rounded {bg} animate-transcript-enter" } else { "flex gap-3 p-3 rounded-lg {bg} animate-transcript-enter" },
          span { class: "material-symbols-outlined text-sm text-[var(--outline)] shrink-0 mt-0.5",
            "{icon}"
          }
          div { class: "flex-1 min-w-0",
            MarkdownBody { content: text }
          }
        }
      }
    },
    TranscriptBlock::Thinking { text, .. } => {
      rsx! {
        div { class: "flex gap-3 p-3 rounded-lg bg-[var(--warning)]/5 border border-[var(--warning)]/10 animate-transcript-enter",
          span { class: "material-symbols-outlined text-sm text-[var(--warning)] shrink-0 mt-0.5",
            "psychology"
          }
          div { class: "flex-1 min-w-0 text-xs text-[var(--outline)] italic whitespace-pre-wrap",
            "{text}"
          }
        }
      }
    },
    TranscriptBlock::Tool { name, input, result, is_error, status, token_count, .. } => {
      let status_label = match status {
        ToolStatus::Running => "Running",
        ToolStatus::Completed => "Completed",
        ToolStatus::Error => "Errored",
      };
      let status_color = match status {
        ToolStatus::Running => "text-[var(--tertiary)]",
        ToolStatus::Completed => "text-[var(--success)]",
        ToolStatus::Error => "text-[var(--error)]",
      };
      let icon = match status {
        ToolStatus::Running => "build",
        ToolStatus::Completed => "check_circle",
        ToolStatus::Error => "error",
      };
      let border = if is_error { "border-[var(--error)]/20 bg-[var(--error)]/[0.04]" } else { "border-[var(--outline-variant)]/20" };
      rsx! {
        div { class: if density == TranscriptDensity::Compact { "border {border} rounded p-2 space-y-1 animate-transcript-enter" } else { "border {border} rounded-lg p-3 space-y-2 animate-transcript-enter" },
          div { class: "flex items-center gap-2",
            span { class: "material-symbols-outlined text-sm {status_color}",
              "{icon}"
            }
            span { class: "text-[11px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]",
              "{name}"
            }
            span { class: "text-[10px] font-semibold uppercase tracking-wider {status_color}",
              "{status_label}"
            }
            if let Some(tokens) = token_count {
              span { class: "text-[10px] text-[var(--outline)] tabular-nums ml-1",
                "{tokens} tok"
              }
            }
            button {
              class: "ml-auto text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
              onclick: move |_| tool_open.set(!tool_open()),
              span { class: "material-symbols-outlined text-sm",
                if tool_open() {
                  "expand_more"
                } else {
                  "chevron_right"
                }
              }
            }
          }
          if !input.is_empty() && !tool_open() {
            p { class: "text-xs text-[var(--outline)] font-mono truncate",
              {
                  if mode == TranscriptMode::Nice {
                      summarize_tool_input(&input, 120)
                  } else {
                      input.clone()
                  }
              }
            }
          }
          if tool_open() {
            div { class: "grid gap-3 lg:grid-cols-2",
              div {
                div { class: "mb-1 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]",
                  "Input"
                }
                pre { class: "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80",
                  if input.is_empty() {
                    "<empty>"
                  } else {
                    "{input}"
                  }
                }
              }
              div {
                div { class: "mb-1 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]",
                  "Result"
                }
                pre { class: if is_error { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--error)]" } else { "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80" },
                  match result {
                      Some(r) => rsx! { "{r}" },
                      None => rsx! { "Waiting for result..." },
                  }
                }
              }
            }
          }
        }
      }
    },
    TranscriptBlock::Activity { name, status, .. } => {
      rsx! {
        div { class: "flex items-start gap-2 animate-transcript-enter",
          match status {
              ToolStatus::Completed => rsx! {
                span { class: "material-symbols-outlined text-sm text-[var(--success)] shrink-0 mt-0.5",
                  "check_circle"
                }
              },
              _ => rsx! {
                span { class: "relative mt-1 flex h-2.5 w-2.5 shrink-0",
                  span { class: "absolute inline-flex h-full w-full animate-ping rounded-full bg-[var(--tertiary)] opacity-70" }
                  span { class: "relative inline-flex h-2.5 w-2.5 rounded-full bg-[var(--tertiary)]" }
                }
              },
          }
          div { class: "break-words text-sm text-[var(--on-surface)]/80 leading-6",
            "{name}"
          }
        }
      }
    },
    TranscriptBlock::CommandGroup { ref items, .. } => render_command_group(items, cmd_group_open),
    TranscriptBlock::ToolGroup { ref items, .. } => render_tool_group(items, tool_group_open),
    TranscriptBlock::StderrGroup { ref lines, .. } => render_stderr_group(lines, stderr_open),
    TranscriptBlock::Stdout { text, .. } => {
      rsx! {
        div { class: "animate-transcript-enter",
          div { class: "flex items-center gap-2",
            span { class: "text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]",
              "stdout"
            }
            button {
              class: "text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
              onclick: move |_| stdout_open.set(!stdout_open()),
              span { class: "material-symbols-outlined text-sm",
                if stdout_open() {
                  "expand_more"
                } else {
                  "chevron_right"
                }
              }
            }
          }
          if stdout_open() {
            pre { class: "mt-2 overflow-x-auto whitespace-pre-wrap break-words font-mono text-xs text-[var(--on-surface)]/80",
              "{text}"
            }
          }
        }
      }
    },
    TranscriptBlock::Event { label, text, detail, tone, .. } => {
      let wrapper_class = match tone.as_str() {
        "error" => "rounded-lg border border-[var(--error)]/20 bg-[var(--error)]/[0.06] p-3 text-[var(--error)]",
        "warn" => "text-[var(--warning)]",
        "info" => "text-[var(--tertiary)]",
        _ => "text-[var(--on-surface)]/75",
      };
      let icon = match tone.as_str() {
        "error" => "error",
        "warn" => "terminal",
        _ => "circle",
      };
      rsx! {
        div { class: "{wrapper_class} animate-transcript-enter",
          div { class: "flex items-start gap-2",
            span { class: "material-symbols-outlined text-sm shrink-0 mt-0.5",
              "{icon}"
            }
            div { class: "min-w-0 flex-1",
              div { class: "whitespace-pre-wrap break-words text-xs",
                span { class: "text-[10px] font-semibold uppercase tracking-wider text-[var(--on-surface-variant)]/70",
                  "{label}"
                }
                if !text.is_empty() {
                  span { class: "ml-2", "{text}" }
                }
              }
              if let Some(ref d) = detail {
                pre { class: "mt-2 overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/75",
                  "{d}"
                }
              }
            }
          }
        }
      }
    },
  }
}
