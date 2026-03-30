use dioxus::prelude::*;

use super::markdown_body::MarkdownBody;

#[derive(Clone, Copy, Debug, PartialEq)]
enum EditorMode {
  Edit,
  Preview,
  Split,
}

#[component]
pub fn MarkdownEditor(
  value: String,
  on_change: EventHandler<String>,
  #[props(optional)] on_submit: Option<EventHandler<String>>,
  #[props(optional)] placeholder: Option<String>,
  #[props(optional)] class: Option<String>,
) -> Element {
  let mut mode = use_signal(|| EditorMode::Edit);
  let current_mode = *mode.read();
  let extra_class = class.as_deref().unwrap_or("");
  let placeholder_text = placeholder.as_deref().unwrap_or("Write markdown...");

  rsx! {
    div { class: "flex flex-col border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden {extra_class}",
      div { class: "flex items-center justify-between border-b border-[var(--outline-variant)]/30 px-2 py-1 bg-[var(--surface-container)]",
        ToolbarButtons { value: value.clone(), on_change: on_change }
        div { class: "flex gap-0.5",
          ModeButton { label: "Edit", active: current_mode == EditorMode::Edit, on_click: move |_| mode.set(EditorMode::Edit) }
          ModeButton { label: "Preview", active: current_mode == EditorMode::Preview, on_click: move |_| mode.set(EditorMode::Preview) }
          ModeButton { label: "Split", active: current_mode == EditorMode::Split, on_click: move |_| mode.set(EditorMode::Split) }
        }
      }
      match current_mode {
          EditorMode::Edit => rsx! {
            EditorTextarea {
              value: value.clone(),
              placeholder: placeholder_text.to_string(),
              on_change: on_change,
              on_submit: on_submit,
            }
          },
          EditorMode::Preview => rsx! {
            div { class: "p-3 min-h-[8rem] max-h-80 overflow-y-auto",
              if value.is_empty() {
                p { class: "text-sm text-[var(--outline)]", "Nothing to preview." }
              } else {
                MarkdownBody { content: value.clone() }
              }
            }
          },
          EditorMode::Split => rsx! {
            div { class: "grid grid-cols-2 divide-x divide-[var(--outline-variant)]/30",
              EditorTextarea {
                value: value.clone(),
                placeholder: placeholder_text.to_string(),
                on_change: on_change,
                on_submit: on_submit,
              }
              div { class: "p-3 min-h-[8rem] max-h-80 overflow-y-auto",
                if value.is_empty() {
                  p { class: "text-sm text-[var(--outline)]", "Nothing to preview." }
                } else {
                  MarkdownBody { content: value.clone() }
                }
              }
            }
          },
      }
    }
  }
}

#[component]
fn EditorTextarea(value: String, placeholder: String, on_change: EventHandler<String>, #[props(optional)] on_submit: Option<EventHandler<String>>) -> Element {
  rsx! {
    textarea {
      class: "w-full min-h-[8rem] max-h-80 p-3 bg-transparent outline-none text-sm font-mono text-[var(--on-surface)] placeholder:text-[var(--outline)]/40 resize-y",
      value: "{value}",
      placeholder: "{placeholder}",
      oninput: move |evt| on_change.call(evt.value().to_string()),
      onkeydown: move |evt| {
          if evt.modifiers().meta() && evt.key() == Key::Enter
            && let Some(ref handler) = on_submit
          {
            handler.call(value.clone());
          }
      },
    }
  }
}

#[component]
fn ModeButton(label: &'static str, active: bool, on_click: EventHandler<()>) -> Element {
  let bg = if active { "bg-[var(--surface-container-high)] text-[var(--on-surface)]" } else { "text-[var(--outline)] hover:text-[var(--on-surface)]" };
  rsx! {
    button {
      class: "px-2 py-0.5 text-[11px] font-medium rounded transition-colors {bg}",
      onclick: move |_| on_click.call(()),
      "{label}"
    }
  }
}

#[component]
fn ToolbarButtons(value: String, on_change: EventHandler<String>) -> Element {
  rsx! {
    div { class: "flex gap-0.5",
      ToolbarBtn {
        icon: "format_bold",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}****"))
        },
      }
      ToolbarBtn {
        icon: "format_italic",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}**"))
        },
      }
      ToolbarBtn {
        icon: "code",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}\n```\n\n```"))
        },
      }
      ToolbarBtn {
        icon: "link",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}[text](url)"))
        },
      }
      ToolbarBtn {
        icon: "title",
        on_click: {
            let v = value.clone();
            move |_| on_change.call(format!("{v}\n## "))
        },
      }
    }
  }
}

#[component]
fn ToolbarBtn(icon: &'static str, on_click: EventHandler<()>) -> Element {
  rsx! {
    button {
      class: "w-6 h-6 flex items-center justify-center text-[var(--outline)] hover:text-[var(--on-surface)] hover:bg-[var(--surface-container-high)] rounded transition-colors",
      onclick: move |_| on_click.call(()),
      span { class: "material-symbols-outlined text-sm", "{icon}" }
    }
  }
}
