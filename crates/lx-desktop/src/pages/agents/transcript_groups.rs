use super::transcript::{StderrLine, ToolItem, ToolStatus};
use dioxus::prelude::*;

pub fn status_class(status: &ToolStatus) -> &'static str {
  match status {
    ToolStatus::Running => "text-[10px] font-semibold uppercase tracking-wider text-[var(--tertiary)]",
    ToolStatus::Completed => "text-[10px] font-semibold uppercase tracking-wider text-[var(--success)]",
    ToolStatus::Error => "text-[10px] font-semibold uppercase tracking-wider text-[var(--error)]",
  }
}

pub fn status_label(status: &ToolStatus) -> &'static str {
  match status {
    ToolStatus::Running => "Running",
    ToolStatus::Completed => "Completed",
    ToolStatus::Error => "Errored",
  }
}

pub fn render_command_group(items: &[ToolItem], mut cmd_group_open: Signal<bool>) -> Element {
  let has_error = items.iter().any(|i| i.is_error);
  let is_running = items.iter().any(|i| matches!(i.status, ToolStatus::Running));
  let title = if is_running {
    "Executing command".to_string()
  } else if items.len() == 1 {
    "Executed command".to_string()
  } else {
    format!("Executed {} commands", items.len())
  };
  let wrapper = if has_error && cmd_group_open() { "rounded-lg border border-[var(--error)]/20 bg-[var(--error)]/[0.04] p-3" } else { "" };
  let err_cls = "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--error)]";
  let ok_cls = "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80";
  rsx! {
    div { class: "{wrapper}",
      div {
        class: "flex items-center gap-2 cursor-pointer",
        onclick: move |_| cmd_group_open.set(!cmd_group_open()),
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]", "terminal" }
        span { class: "text-[11px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]/70", "{title}" }
        button {
          class: "ml-auto text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
          onclick: move |evt| { evt.stop_propagation(); cmd_group_open.set(!cmd_group_open()); },
          span { class: "material-symbols-outlined text-sm",
            if cmd_group_open() { "expand_more" } else { "chevron_right" }
          }
        }
      }
      if cmd_group_open() {
        div { class: "mt-3 space-y-3",
          for (idx , item) in items.iter().enumerate() {
            div { key: "{idx}", class: "space-y-2",
              div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-xs text-[var(--outline)]", "terminal" }
                span { class: "font-mono text-xs break-all", "{item.input}" }
              }
              if let Some(ref res) = item.result {
                pre { class: if item.is_error { err_cls } else { ok_cls }, "{res}" }
              }
            }
          }
        }
      }
    }
  }
}

pub fn render_tool_group(items: &[ToolItem], mut tool_group_open: Signal<bool>) -> Element {
  let is_running = items.iter().any(|i| matches!(i.status, ToolStatus::Running));
  let title = if is_running { format!("Using {} tools", items.len()) } else { format!("Used {} tools ({} calls)", items.len(), items.len()) };
  let err_cls = "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--error)]";
  let ok_cls = "overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--on-surface)]/80";
  rsx! {
    div { class: "rounded-lg border border-[var(--outline-variant)]/40 bg-[var(--surface-container)]/25",
      div {
        class: "flex items-center gap-2 px-3 py-2.5 cursor-pointer",
        onclick: move |_| tool_group_open.set(!tool_group_open()),
        span { class: "material-symbols-outlined text-sm text-[var(--outline)]", "build" }
        span { class: "text-[11px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]/70", "{title}" }
        button {
          class: "ml-auto text-[var(--outline)] hover:text-[var(--on-surface)] transition-colors",
          onclick: move |evt| { evt.stop_propagation(); tool_group_open.set(!tool_group_open()); },
          span { class: "material-symbols-outlined text-sm",
            if tool_group_open() { "expand_more" } else { "chevron_right" }
          }
        }
      }
      if tool_group_open() {
        div { class: "space-y-2 border-t border-[var(--outline-variant)]/30 px-3 py-3",
          for (idx , item) in items.iter().enumerate() {
            div { key: "{idx}", class: "space-y-1.5",
              div { class: "flex items-center gap-2",
                span { class: "material-symbols-outlined text-xs text-[var(--outline)]", "build" }
                span { class: "text-[10px] font-semibold uppercase tracking-wider text-[var(--on-surface-variant)]", "{item.name}" }
                span { class: status_class(&item.status), {status_label(&item.status)} }
              }
              div { class: "grid gap-2 pl-7 grid-cols-1 lg:grid-cols-2",
                div {
                  div { class: "mb-0.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "Input" }
                  pre { class: ok_cls, if item.input.is_empty() { "<empty>" } else { "{item.input}" } }
                }
                if let Some(ref res) = item.result {
                  div {
                    div { class: "mb-0.5 text-[10px] font-semibold uppercase tracking-widest text-[var(--on-surface-variant)]", "Result" }
                    pre { class: if item.is_error { err_cls } else { ok_cls }, "{res}" }
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

pub fn render_stderr_group(lines: &[StderrLine], mut stderr_open: Signal<bool>) -> Element {
  let count = lines.len();
  let noun = if count == 1 { "line" } else { "lines" };
  rsx! {
    div { class: "rounded-lg border border-[var(--warning)]/20 bg-[var(--warning)]/[0.06] p-2 text-[var(--warning)]",
      div {
        class: "flex items-center gap-2 cursor-pointer",
        onclick: move |_| stderr_open.set(!stderr_open()),
        span { class: "text-[10px] font-semibold uppercase tracking-wider", "{count} log {noun}" }
        span { class: "material-symbols-outlined text-sm",
          if stderr_open() { "expand_more" } else { "chevron_right" }
        }
      }
      if stderr_open() {
        pre { class: "mt-2 overflow-x-auto whitespace-pre-wrap break-words font-mono text-[11px] text-[var(--warning)]/80 pl-5",
          for line in lines.iter() {
            "{line.text}\n"
          }
        }
      }
    }
  }
}
