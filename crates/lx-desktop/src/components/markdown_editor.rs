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
      id: "lx-md-editor",
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

fn insert_at_cursor(value: &str, before: &str, after: &str, on_change: EventHandler<String>) {
  let val = value.to_string();
  let before = before.to_string();
  let after = after.to_string();
  spawn(async move {
    let js = r#"
      (function() {
        var el = document.getElementById('lx-md-editor');
        if (!el) return JSON.stringify({start: -1, end: -1});
        return JSON.stringify({start: el.selectionStart, end: el.selectionEnd});
      })()
    "#;
    let (start, end) = match document::eval(js).await {
      Ok(result) => {
        let s = result.to_string();
        let s = s.trim_matches('"');
        match serde_json::from_str::<serde_json::Value>(s) {
          Ok(v) => {
            let st = v["start"].as_i64().unwrap_or(-1);
            let en = v["end"].as_i64().unwrap_or(-1);
            if st >= 0 && en >= 0 { (st as usize, en as usize) } else { (val.len(), val.len()) }
          },
          Err(_) => (val.len(), val.len()),
        }
      },
      Err(_) => (val.len(), val.len()),
    };

    let start = start.min(val.len());
    let end = end.min(val.len());
    let selected = &val[start..end];
    let new_value = format!("{}{}{}{}{}", &val[..start], before, selected, after, &val[end..]);
    on_change.call(new_value);

    let new_cursor = start + before.len() + selected.len();
    let set_cursor_js = format!(
      "setTimeout(function() {{ var el = document.getElementById('lx-md-editor'); if (el) {{ el.selectionStart = {new_cursor}; el.selectionEnd = {new_cursor}; el.focus(); }} }}, 0)"
    );
    let _ = document::eval(&set_cursor_js).await;
  });
}

#[component]
fn ToolbarButtons(value: String, on_change: EventHandler<String>) -> Element {
  rsx! {
    div { class: "flex gap-0.5",
      ToolbarBtn {
        icon: "format_bold",
        on_click: {
          let v = value.clone();
          move |_| insert_at_cursor(&v, "**", "**", on_change)
        },
      }
      ToolbarBtn {
        icon: "format_italic",
        on_click: {
          let v = value.clone();
          move |_| insert_at_cursor(&v, "*", "*", on_change)
        },
      }
      ToolbarBtn {
        icon: "code",
        on_click: {
          let v = value.clone();
          move |_| insert_at_cursor(&v, "\n```\n", "\n```", on_change)
        },
      }
      ToolbarBtn {
        icon: "link",
        on_click: {
          let v = value.clone();
          move |_| insert_at_cursor(&v, "[", "](url)", on_change)
        },
      }
      ToolbarBtn {
        icon: "title",
        on_click: {
          let v = value.clone();
          move |_| insert_at_cursor(&v, "\n## ", "", on_change)
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
