# WU-16: @mention Autocomplete

## Dependencies: WU-01 must run first.

## Fixes
- Fix 1: Create `mention_popup.rs` component -- a floating autocomplete popup that shows matching agent names
- Fix 2: Add @-trigger detection to `EditorTextarea` in `markdown_editor.rs` -- detect when user types `@` and extract the partial query
- Fix 3: Wire the popup into `MarkdownEditor` with position tracking and selection insertion
- Fix 4: Wire the popup into `CommentThread` via the existing `MarkdownEditor` usage (no changes needed in `comment_thread.rs` itself -- it already delegates to `MarkdownEditor`)

## Files Modified
- `crates/lx-desktop/src/components/markdown_editor.rs` (~220 lines post-WU-01)
- `crates/lx-desktop/src/components/markdown_toolbar.rs` (new file, split from markdown_editor.rs)
- `crates/lx-desktop/src/components/mention_popup.rs` (new file)
- `crates/lx-desktop/src/components/mod.rs` (27 lines)

## Preconditions (post-WU-01 state)
- `markdown_editor.rs` at ~220 lines after WU-01
- `MarkdownEditor` component generates `editor_id` internally via `use_hook(|| format!("lx-md-editor-{}", Uuid::new_v4().simple()))` -- it is NOT a prop
- `EditorTextarea` signature: `fn EditorTextarea(editor_id: String, value: String, placeholder: String, on_change: EventHandler<String>, #[props(optional)] on_submit: Option<EventHandler<String>>) -> Element`
- `insert_at_cursor` signature: `fn insert_at_cursor(editor_id: &str, value: &str, before: &str, after: &str, on_change: EventHandler<String>)`
- `ToolbarButtons` signature: `fn ToolbarButtons(editor_id: String, value: String, on_change: EventHandler<String>) -> Element`
- The textarea uses `id: "{editor_id}"` (dynamic, not hardcoded)
- `comment_thread.rs` uses `MarkdownEditor` at line 58 -- it will automatically gain @mention support when MarkdownEditor is updated
- `components/mod.rs` lists all component modules at lines 1-27

## Steps

### Step 1: Split markdown_editor.rs proactively

WU-16 additions will push `markdown_editor.rs` from ~220 lines (post-WU-01) to ~314 lines, exceeding the 300-line limit. Split BEFORE adding mention code.

Extract into new file `crates/lx-desktop/src/components/markdown_toolbar.rs`:
- `insert_at_cursor` function (~40 lines)
- `ToolbarButtons` component (~40 lines)
- `ToolbarBtn` component (~10 lines)

This moves ~90 lines out, leaving `markdown_editor.rs` at ~130 lines before WU-16 additions.

`markdown_toolbar.rs` content (post-WU-01 state):
```rust
use dioxus::prelude::*;

pub fn insert_at_cursor(editor_id: &str, value: &str, before: &str, after: &str, on_change: EventHandler<String>) {
  let val = value.to_string();
  let editor_id = editor_id.to_string();
  let before = before.to_string();
  let after = after.to_string();
  spawn(async move {
    let js = format!(
      "(function() {{ var el = document.getElementById('{}'); if (!el) return JSON.stringify({{start: -1, end: -1}}); return JSON.stringify({{start: el.selectionStart, end: el.selectionEnd}}); }})()",
      editor_id
    );
    let (start, end) = match document::eval(&js).await {
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
      "setTimeout(function() {{ var el = document.getElementById('{editor_id}'); if (el) {{ el.selectionStart = {new_cursor}; el.selectionEnd = {new_cursor}; el.focus(); }} }}, 0)"
    );
    let _ = document::eval(&set_cursor_js).await;
  });
}

#[component]
pub fn ToolbarButtons(editor_id: String, value: String, on_change: EventHandler<String>) -> Element {
  rsx! {
    div { class: "flex gap-0.5",
      ToolbarBtn {
        icon: "format_bold",
        on_click: {
          let eid = editor_id.clone();
          let v = value.clone();
          move |_| insert_at_cursor(&eid, &v, "**", "**", on_change)
        },
      }
      ToolbarBtn {
        icon: "format_italic",
        on_click: {
          let eid = editor_id.clone();
          let v = value.clone();
          move |_| insert_at_cursor(&eid, &v, "*", "*", on_change)
        },
      }
      ToolbarBtn {
        icon: "code",
        on_click: {
          let eid = editor_id.clone();
          let v = value.clone();
          move |_| insert_at_cursor(&eid, &v, "\n```\n", "\n```", on_change)
        },
      }
      ToolbarBtn {
        icon: "link",
        on_click: {
          let eid = editor_id.clone();
          let v = value.clone();
          move |_| insert_at_cursor(&eid, &v, "[", "](url)", on_change)
        },
      }
      ToolbarBtn {
        icon: "title",
        on_click: {
          let eid = editor_id.clone();
          let v = value.clone();
          move |_| insert_at_cursor(&eid, &v, "\n## ", "", on_change)
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
```

Update `markdown_editor.rs` imports to use the extracted module:
- Replace `use super::markdown_body::MarkdownBody;` with:
```rust
use super::markdown_body::MarkdownBody;
use super::markdown_toolbar::ToolbarButtons;
```
- Remove the `insert_at_cursor`, `ToolbarButtons`, and `ToolbarBtn` definitions from `markdown_editor.rs`

Register the new module in `mod.rs` -- add `pub mod markdown_toolbar;` after `pub mod markdown_editor;`

### Step 2: Create `mention_popup.rs`
- Create new file: `crates/lx-desktop/src/components/mention_popup.rs`
- Content:

```rust
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub struct MentionCandidate {
  pub id: String,
  pub name: String,
}

#[component]
pub fn MentionPopup(
  candidates: Vec<MentionCandidate>,
  query: String,
  visible: bool,
  top: f64,
  left: f64,
  selected_index: usize,
  on_select: EventHandler<MentionCandidate>,
) -> Element {
  if !visible || candidates.is_empty() {
    return rsx! {};
  }

  let filtered: Vec<&MentionCandidate> = candidates
    .iter()
    .filter(|c| {
      query.is_empty() || c.name.to_lowercase().contains(&query.to_lowercase())
    })
    .collect();

  if filtered.is_empty() {
    return rsx! {};
  }

  rsx! {
    div {
      class: "fixed z-[100] bg-[var(--surface-container-high)] border border-[var(--outline-variant)] rounded-lg shadow-lg py-1 min-w-[180px] max-h-48 overflow-y-auto",
      style: "top: {top}px; left: {left}px;",
      for (i, candidate) in filtered.iter().enumerate() {
        {
          let c = (*candidate).clone();
          let is_selected = i == selected_index;
          let bg = if is_selected { "bg-[var(--surface-container-highest)]" } else { "" };
          rsx! {
            button {
              key: "{c.id}",
              class: "w-full text-left px-3 py-1.5 text-sm text-[var(--on-surface)] hover:bg-[var(--surface-container-highest)] flex items-center gap-2 {bg}",
              onmousedown: {
                let c = c.clone();
                move |evt: MouseEvent| {
                  evt.prevent_default();
                  on_select.call(c.clone());
                }
              },
              span { class: "w-5 h-5 rounded-full bg-[var(--primary)]/20 text-[var(--primary)] text-xs flex items-center justify-center font-semibold shrink-0",
                "{c.name.chars().next().unwrap_or('?')}"
              }
              span { "{c.name}" }
            }
          }
        }
      }
    }
  }
}
```

### Step 3: Register `mention_popup` module
- Open `crates/lx-desktop/src/components/mod.rs`
- At line 16 (after `pub mod markdown_editor;`), find:
```
pub mod markdown_editor;
pub mod metric_card;
```
- Replace with:
```
pub mod markdown_editor;
pub mod markdown_toolbar;
pub mod mention_popup;
pub mod metric_card;
```

### Step 4: Add @mention state and detection to `MarkdownEditor`
- Open `crates/lx-desktop/src/components/markdown_editor.rs`
- At line 1, add the mention_popup import:
```rust
use dioxus::prelude::*;
use uuid::Uuid;

use super::markdown_body::MarkdownBody;
use super::markdown_toolbar::ToolbarButtons;
use super::mention_popup::{MentionCandidate, MentionPopup};
```

- At the `MarkdownEditor` component signature, add the `mention_candidates` prop:
```rust
#[component]
pub fn MarkdownEditor(
  value: String,
  on_change: EventHandler<String>,
  #[props(optional)] on_submit: Option<EventHandler<String>>,
  #[props(optional)] placeholder: Option<String>,
  #[props(optional)] class: Option<String>,
  #[props(optional)] mention_candidates: Option<Vec<MentionCandidate>>,
) -> Element {
```

- After `let mut mode = use_signal(|| EditorMode::Edit);`, add the following state signals:
```rust
  let mut mention_visible = use_signal(|| false);
  let mut mention_query = use_signal(String::new);
  let mut mention_top = use_signal(|| 0.0_f64);
  let mut mention_left = use_signal(|| 0.0_f64);
  let mut mention_selected = use_signal(|| 0_usize);
  let mut mention_start_pos = use_signal(|| 0_usize);
  let candidates = mention_candidates.unwrap_or_default();
```

- In the `rsx!` block, inside the outermost `div { ... }` but after the `match current_mode { ... }` block, add the `MentionPopup` component:

Find the end of the match block:
```
      }
    }
  }
}
```
Replace with:
```
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
            let before = &value[..start];
            let at_end = value[start..].find(' ').map(|i| start + i).unwrap_or(value.len());
            let after = &value[at_end..];
            let new_value = format!("{}@{}{}", before, candidate.name, after);
            on_change.call(new_value);
            mention_visible.set(false);
            mention_query.set(String::new());
          }
        },
      }
    }
  }
}
```

### Step 5: Update `EditorTextarea` to detect @mention trigger
- In `markdown_editor.rs`, find the `EditorTextarea` component. Post-WU-01 signature is:
```rust
fn EditorTextarea(editor_id: String, value: String, placeholder: String, on_change: EventHandler<String>, #[props(optional)] on_submit: Option<EventHandler<String>>) -> Element {
```
- Replace with:
```rust
fn EditorTextarea(
  editor_id: String,
  value: String,
  placeholder: String,
  on_change: EventHandler<String>,
  #[props(optional)] on_submit: Option<EventHandler<String>>,
  #[props(optional)] on_mention_trigger: Option<EventHandler<(String, usize)>>,
  #[props(optional)] on_mention_dismiss: Option<EventHandler<()>>,
  #[props(optional)] on_mention_nav: Option<EventHandler<&'static str>>,
) -> Element {
```

- Replace the `oninput` handler. Post-WU-01 the `oninput` is:
```rust
      oninput: {
          let eid = editor_id.clone();
          move |evt: FormEvent| {
              on_change.call(evt.value().to_string());
              let eid = eid.clone();
              spawn(async move {
                  let js = format!(
                      "var el = document.getElementById('{}'); if (el) {{ el.style.height = 'auto'; el.style.height = el.scrollHeight + 'px'; }}",
                      eid
                  );
                  let _ = document::eval(&js).await;
              });
          }
      },
```
- Replace with:
```rust
      oninput: {
          let eid = editor_id.clone();
          let on_mention_trigger = on_mention_trigger.clone();
          let on_mention_dismiss = on_mention_dismiss.clone();
          move |evt: FormEvent| {
              let new_val = evt.value().to_string();
              on_change.call(new_val.clone());
              let eid = eid.clone();
              let on_mention_trigger = on_mention_trigger.clone();
              let on_mention_dismiss = on_mention_dismiss.clone();
              spawn(async move {
                  let grow_js = format!(
                      "var el = document.getElementById('{}'); if (el) {{ el.style.height = 'auto'; el.style.height = el.scrollHeight + 'px'; }}",
                      eid
                  );
                  let _ = document::eval(&grow_js).await;

                  let pos_js = format!(
                      "(function() {{ var el = document.getElementById('{}'); if (!el) return JSON.stringify({{pos: -1}}); return JSON.stringify({{pos: el.selectionStart}}); }})()",
                      eid
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
                      },
                      Err(_) => None,
                  };
                  if let Some(cursor) = pos {
                      let before_cursor = &new_val[..cursor.min(new_val.len())];
                      if let Some(at_pos) = before_cursor.rfind('@') {
                          let between = &before_cursor[at_pos + 1..];
                          let valid = between.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-');
                          let preceded_by_space_or_start = at_pos == 0 || before_cursor.as_bytes().get(at_pos - 1).map_or(false, |b| *b == b' ' || *b == b'\n');
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
```

- Replace the `onkeydown` handler. Post-WU-01 it is:
```rust
      onkeydown: move |evt| {
          if evt.modifiers().meta() && evt.key() == Key::Enter
            && let Some(ref handler) = on_submit
          {
            handler.call(value.clone());
          }
      },
```
- Replace with:
```rust
      onkeydown: {
        let on_mention_nav = on_mention_nav.clone();
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
```

### Step 6: Wire mention callbacks from MarkdownEditor to EditorTextarea

In the `MarkdownEditor` component's `rsx!`, both `EditorTextarea` renders (Edit and Split modes) need mention handler props.

**Edit mode render** -- find:
```rust
            EditorTextarea {
              editor_id: editor_id.clone(),
              value: value.clone(),
              placeholder: placeholder_text.to_string(),
              on_change: on_change,
              on_submit: on_submit,
            }
```
Replace with:
```rust
            EditorTextarea {
              editor_id: editor_id.clone(),
              value: value.clone(),
              placeholder: placeholder_text.to_string(),
              on_change: on_change,
              on_submit: on_submit,
              on_mention_trigger: {
                let editor_id = editor_id.clone();
                move |(query, start): (String, usize)| {
                  mention_query.set(query);
                  mention_start_pos.set(start);
                  mention_visible.set(true);
                  mention_selected.set(0);
                  let editor_id = editor_id.clone();
                  spawn(async move {
                    let js = format!(
                      "(function() {{ var el = document.getElementById('{}'); if (!el) return JSON.stringify({{top: 0, left: 0}}); var rect = el.getBoundingClientRect(); return JSON.stringify({{top: rect.bottom, left: rect.left + 12}}); }})()",
                      editor_id
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
              },
              on_mention_dismiss: move |_| {
                mention_visible.set(false);
                mention_query.set(String::new());
              },
              on_mention_nav: {
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
                        let before = &value[..start];
                        let after = &value[at_end..];
                        let new_value = format!("{}@{} {}", before, c.name, after);
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
              },
            }
```

**Split mode render** -- find:
```rust
              EditorTextarea {
                editor_id: editor_id.clone(),
                value: value.clone(),
                placeholder: placeholder_text.to_string(),
                on_change: on_change,
                on_submit: on_submit,
              }
```
Replace with:
```rust
              EditorTextarea {
                editor_id: editor_id.clone(),
                value: value.clone(),
                placeholder: placeholder_text.to_string(),
                on_change: on_change,
                on_submit: on_submit,
                on_mention_trigger: {
                  let editor_id = editor_id.clone();
                  move |(query, start): (String, usize)| {
                    mention_query.set(query);
                    mention_start_pos.set(start);
                    mention_visible.set(true);
                    mention_selected.set(0);
                    let editor_id = editor_id.clone();
                    spawn(async move {
                      let js = format!(
                        "(function() {{ var el = document.getElementById('{}'); if (!el) return JSON.stringify({{top: 0, left: 0}}); var rect = el.getBoundingClientRect(); return JSON.stringify({{top: rect.bottom, left: rect.left + 12}}); }})()",
                        editor_id
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
                },
                on_mention_dismiss: move |_| {
                  mention_visible.set(false);
                  mention_query.set(String::new());
                },
                on_mention_nav: {
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
                          let before = &value[..start];
                          let after = &value[at_end..];
                          let new_value = format!("{}@{} {}", before, c.name, after);
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
                },
              }
```

### Step 7: No changes to comment_thread.rs
- `comment_thread.rs` line 58 uses `MarkdownEditor` which now has the optional `mention_candidates` prop
- Since `mention_candidates` is `#[props(optional)]`, existing callers continue to work without changes
- When a caller wants @mention support, they pass `mention_candidates: some_vec` -- this is a caller-side decision

## File Size Check
- `mention_popup.rs`: new file, ~65 lines (under 300)
- `markdown_toolbar.rs`: new file, ~90 lines (under 300)
- `markdown_editor.rs`: was ~220 lines (post-WU-01), minus ~90 lines extracted to toolbar = ~130 lines, plus ~95 lines of mention code = ~225 lines (under 300)
- `components/mod.rs`: was 27 lines, now 29 lines (under 300)
- `comment_thread.rs`: unchanged at 82 lines (under 300)

## Verification
- Run `just diagnose` to confirm compilation
- Launch the desktop app and test:
  1. Open any page that uses `MarkdownEditor` (e.g., new issue dialog, comment thread)
  2. Type `@` in the editor -- if `mention_candidates` is provided, the popup should appear
  3. Type additional characters after `@` -- the popup should filter the list
  4. Use arrow keys to navigate, Enter to select, Escape to dismiss
  5. Selecting a candidate inserts `@AgentName ` at the cursor position
  6. The popup should not appear when `@` is in the middle of a word (e.g., `email@`)
  7. Without `mention_candidates` prop, behavior is unchanged (no popup)
