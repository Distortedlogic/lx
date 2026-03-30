use dioxus::prelude::*;
use uuid::Uuid;

use super::drag_drop::{build_markdown_links, install_drop_listener, read_dropped_files, DragOverlay, DroppedFile};
use super::editor_textarea::EditorTextarea;
use super::markdown_body::MarkdownBody;
use super::markdown_toolbar::ToolbarButtons;
use super::mention_popup::{MentionCandidate, MentionPopup};

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
  #[props(optional)] mention_candidates: Option<Vec<MentionCandidate>>,
  #[props(optional)] on_files: Option<EventHandler<Vec<DroppedFile>>>,
) -> Element {
  let editor_id = use_hook(|| format!("lx-md-editor-{}", Uuid::new_v4().simple()));
  let mut mode = use_signal(|| EditorMode::Edit);
  let mut mention_visible = use_signal(|| false);
  let mut mention_query = use_signal(String::new);
  let mut mention_top = use_signal(|| 0.0_f64);
  let mut mention_left = use_signal(|| 0.0_f64);
  let mut mention_selected = use_signal(|| 0_usize);
  let mut mention_start_pos = use_signal(|| 0_usize);
  let candidates = mention_candidates.unwrap_or_default();
  let mut dragging = use_signal(|| false);

  use_effect(install_drop_listener);

  let current_mode = *mode.read();
  let extra_class = class.as_deref().unwrap_or("");
  let drag_class = if dragging() { "drag-active" } else { "" };
  let placeholder_text = placeholder.as_deref().unwrap_or("Write markdown...");

  let on_mention_trigger = {
    let editor_id = editor_id.clone();
    move |(query, start): (String, usize)| {
      mention_query.set(query);
      mention_start_pos.set(start);
      mention_visible.set(true);
      mention_selected.set(0);
      let editor_id = editor_id.clone();
      spawn(async move {
        let js = format!(
          "(function() {{ var el = document.getElementById('{editor_id}'); if (!el) return JSON.stringify({{top: 0, left: 0}}); var rect = el.getBoundingClientRect(); return JSON.stringify({{top: rect.bottom, left: rect.left + 12}}); }})()"
        );
        if let Ok(result) = document::eval(&js).await {
          let s = result.to_string();
          let s = s.trim_matches('"');
          if let Ok(v) = serde_json::from_str::<serde_json::Value>(s) {
            mention_top.set(v["top"].as_f64().unwrap_or(0.0));
            mention_left.set(v["left"].as_f64().unwrap_or(0.0));
          }
        }
      });
    }
  };

  let on_mention_dismiss = move |_: ()| {
    mention_visible.set(false);
    mention_query.set(String::new());
  };

  let on_mention_nav = {
    let candidates = candidates.clone();
    let value = value.clone();
    move |dir: &'static str| {
      if !mention_visible() { return; }
      let filtered: Vec<_> = candidates.iter().filter(|c| {
        mention_query().is_empty() || c.name.to_lowercase().contains(&mention_query().to_lowercase())
      }).collect();
      if filtered.is_empty() { return; }
      match dir {
        "down" => mention_selected.set((mention_selected() + 1) % filtered.len()),
        "up" => mention_selected.set(mention_selected().checked_sub(1).unwrap_or(filtered.len() - 1)),
        "select" => {
          if let Some(c) = filtered.get(mention_selected()) {
            let start = mention_start_pos();
            let at_end = value[start..].find(' ').map(|i| start + i).unwrap_or(value.len());
            let new_value = format!("{}@{} {}", &value[..start], c.name, &value[at_end..]);
            on_change.call(new_value);
            mention_visible.set(false);
            mention_query.set(String::new());
          }
        }
        "dismiss" => {
          mention_visible.set(false);
          mention_query.set(String::new());
        }
        _ => {}
      }
    }
  };

  rsx! {
    div {
      class: "flex flex-col border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden relative {extra_class} {drag_class}",
      ondragover: move |evt: DragEvent| {
        evt.prevent_default();
        dragging.set(true);
      },
      ondragleave: move |_: DragEvent| {
        dragging.set(false);
      },
      ondrop: {
        let value = value.clone();
        move |evt: DragEvent| {
          evt.prevent_default();
          dragging.set(false);
          let value = value.clone();
          spawn(async move {
            let dropped = read_dropped_files().await;
            if !dropped.is_empty() {
              if let Some(ref handler) = on_files {
                handler.call(dropped.clone());
              }
              let links = build_markdown_links(&dropped);
              on_change.call(format!("{}{}", value, links));
            }
          });
        }
      },
      if dragging() { DragOverlay {} }
      div { class: "flex items-center justify-between border-b border-[var(--outline-variant)]/30 px-2 py-1 bg-[var(--surface-container)]",
        ToolbarButtons { editor_id: editor_id.clone(), value: value.clone(), on_change: on_change }
        div { class: "flex gap-0.5",
          ModeButton { label: "Edit", active: current_mode == EditorMode::Edit, on_click: move |_| mode.set(EditorMode::Edit) }
          ModeButton { label: "Preview", active: current_mode == EditorMode::Preview, on_click: move |_| mode.set(EditorMode::Preview) }
          ModeButton { label: "Split", active: current_mode == EditorMode::Split, on_click: move |_| mode.set(EditorMode::Split) }
        }
      }
      match current_mode {
          EditorMode::Edit => rsx! {
            EditorTextarea {
              editor_id: editor_id.clone(),
              value: value.clone(),
              placeholder: placeholder_text.to_string(),
              on_change: on_change,
              on_submit: on_submit,
              on_mention_trigger: on_mention_trigger,
              on_mention_dismiss: on_mention_dismiss,
              on_mention_nav: on_mention_nav,
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
                editor_id: editor_id.clone(),
                value: value.clone(),
                placeholder: placeholder_text.to_string(),
                on_change: on_change,
                on_submit: on_submit,
                on_mention_trigger: on_mention_trigger,
                on_mention_dismiss: on_mention_dismiss,
                on_mention_nav: on_mention_nav,
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
      MentionPopup {
        candidates: candidates.clone(),
        query: mention_query(),
        visible: mention_visible(),
        top: mention_top(),
        left: mention_left(),
        selected_index: mention_selected(),
        on_select: {
          let value = value.clone();
          move |candidate: MentionCandidate| {
            let start = mention_start_pos();
            let at_end = value[start..].find(' ').map(|i| start + i).unwrap_or(value.len());
            let new_value = format!("{}@{}{}", &value[..start], candidate.name, &value[at_end..]);
            on_change.call(new_value);
            mention_visible.set(false);
            mention_query.set(String::new());
          }
        },
      }
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
