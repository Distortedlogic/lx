use dioxus::prelude::*;

#[component]
pub fn EditorTextarea(
  editor_id: String,
  value: String,
  placeholder: String,
  on_change: EventHandler<String>,
  #[props(optional)] on_submit: Option<EventHandler<String>>,
  #[props(optional)] on_mention_trigger: Option<EventHandler<(String, usize)>>,
  #[props(optional)] on_mention_dismiss: Option<EventHandler<()>>,
  #[props(optional)] on_mention_nav: Option<EventHandler<&'static str>>,
) -> Element {
  rsx! {
    textarea {
      id: "{editor_id}",
      class: "w-full min-h-[8rem] p-3 bg-transparent outline-none text-sm font-mono text-[var(--on-surface)] placeholder:text-[var(--outline)]/40 resize-none overflow-hidden",
      value: "{value}",
      placeholder: "{placeholder}",
      oninput: {
          let eid = editor_id.clone();
          let on_mention_trigger = on_mention_trigger;
          let on_mention_dismiss = on_mention_dismiss;
          move |evt: FormEvent| {
              let new_val = evt.value().to_string();
              on_change.call(new_val.clone());
              let eid = eid.clone();
              let on_mention_trigger = on_mention_trigger;
              let on_mention_dismiss = on_mention_dismiss;
              spawn(async move {
                  let grow_js = format!(
                      "var el = document.getElementById('{eid}'); if (el) {{ el.style.height = 'auto'; el.style.height = el.scrollHeight + 'px'; }}",
                  );
                  let _ = document::eval(&grow_js).await;

                  let pos_js = format!(
                      "(function() {{ var el = document.getElementById('{eid}'); if (!el) return JSON.stringify({{pos: -1}}); return JSON.stringify({{pos: el.selectionStart}}); }})()",
                  );
                  let pos = match document::eval(&pos_js).await {
                      Ok(result) => {
                          let s = result.to_string();
                          let s = s.trim_matches('"');
                          serde_json::from_str::<serde_json::Value>(s)
                              .ok()
                              .and_then(|v| v["pos"].as_i64())
                              .filter(|p| *p >= 0)
                              .map(|p| p as usize)
                      }
                      Err(_) => None,
                  };
                  if let Some(cursor) = pos {
                      let before_cursor = &new_val[..cursor.min(new_val.len())];
                      if let Some(at_pos) = before_cursor.rfind('@') {
                          let between = &before_cursor[at_pos + 1..];
                          let valid = between
                              .chars()
                              .all(|c| c.is_alphanumeric() || c == '_' || c == '-');
                          let preceded_by_space_or_start = at_pos == 0
                              || before_cursor
                                  .as_bytes()
                                  .get(at_pos - 1)
                                  .is_some_and(|b| *b == b' ' || *b == b'\n');
                          if valid && preceded_by_space_or_start {
                              if let Some(ref handler) = on_mention_trigger {
                                  handler.call((between.to_string(), at_pos));
                              }
                              return;
                          }
                      }
                      if let Some(ref handler) = on_mention_dismiss {
                          handler.call(());
                      }
                  }
              });
          }
      },
      onkeydown: {
          let on_mention_nav = on_mention_nav;
          move |evt: KeyboardEvent| {
              if evt.modifiers().meta() && evt.key() == Key::Enter {
                  if let Some(ref handler) = on_submit {
                      handler.call(value.clone());
                  }
                  return;
              }
              match evt.key() {
                  Key::ArrowDown | Key::ArrowUp | Key::Enter | Key::Escape => {
                      if let Some(ref handler) = on_mention_nav {
                          let dir = match evt.key() {
                              Key::ArrowDown => "down",
                              Key::ArrowUp => "up",
                              Key::Enter => "select",
                              Key::Escape => "dismiss",
                              _ => return,
                          };
                          evt.prevent_default();
                          handler.call(dir);
                      }
                  }
                  _ => {}
              }
          }
      },
    }
  }
}
