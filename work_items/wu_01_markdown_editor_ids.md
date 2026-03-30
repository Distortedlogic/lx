# WU-01: MarkdownEditor unique IDs and auto-grow textarea

## Fixes
- Fix 2: Hardcoded `id: "lx-md-editor"` means multiple MarkdownEditor instances on the same page share a single DOM ID, breaking `insert_at_cursor` for all but the first instance.
- Fix 9: Textarea has a fixed `min-h-[8rem] max-h-80` with manual `resize-y` but does not auto-grow to fit content. Add JS-based auto-grow on input.

## Files Modified
- `crates/lx-desktop/src/components/markdown_editor.rs` (201 lines)

## Preconditions
- `EditorTextarea` component defined at line 75 with hardcoded `id: "lx-md-editor"` at line 79.
- `insert_at_cursor` function defined at line 107, references `'lx-md-editor'` as a string literal at lines 114 and 143.
- The crate already depends on `uuid` (Cargo.toml line 49), and the pattern `use_hook(|| format!("...-{}", Uuid::new_v4().simple()))` is used elsewhere (e.g., `terminal/view.rs` line 220).
- `MarkdownEditor` is the top-level public component at line 13. It renders `EditorTextarea` at lines 37 and 55. It also renders `ToolbarButtons` at line 28 which calls `insert_at_cursor`.

## Steps

### Step 1: Add uuid import
- Open `crates/lx-desktop/src/components/markdown_editor.rs`
- At line 1, find:
```rust
use dioxus::prelude::*;
```
- Replace with:
```rust
use dioxus::prelude::*;
use uuid::Uuid;
```
- Why: Need Uuid for generating unique IDs per editor instance.

### Step 2: Generate a unique ID in MarkdownEditor and pass it down
- At line 20, find:
```rust
  let mut mode = use_signal(|| EditorMode::Edit);
```
- Add before this line:
```rust
  let editor_id = use_hook(|| format!("lx-md-editor-{}", Uuid::new_v4().simple()));
```
- Why: `use_hook` runs once per component mount, giving each MarkdownEditor instance a stable unique DOM ID.

### Step 3: Pass editor_id to ToolbarButtons
- At line 28, find:
```rust
        ToolbarButtons { value: value.clone(), on_change: on_change }
```
- Replace with:
```rust
        ToolbarButtons { editor_id: editor_id.clone(), value: value.clone(), on_change: on_change }
```
- Why: ToolbarButtons calls `insert_at_cursor` which needs the correct element ID.

### Step 4: Pass editor_id to EditorTextarea (both occurrences)
- At lines 37-42 (Edit mode), find:
```rust
            EditorTextarea {
              value: value.clone(),
              placeholder: placeholder_text.to_string(),
              on_change: on_change,
              on_submit: on_submit,
            }
```
- Replace with:
```rust
            EditorTextarea {
              editor_id: editor_id.clone(),
              value: value.clone(),
              placeholder: placeholder_text.to_string(),
              on_change: on_change,
              on_submit: on_submit,
            }
```
- Do the same for lines 55-60 (Split mode) — add `editor_id: editor_id.clone(),` as the first prop.
- Why: EditorTextarea uses the ID as its DOM `id` attribute.

### Step 5: Update EditorTextarea to accept and use editor_id
- At line 76, find:
```rust
fn EditorTextarea(value: String, placeholder: String, on_change: EventHandler<String>, #[props(optional)] on_submit: Option<EventHandler<String>>) -> Element {
```
- Replace with:
```rust
fn EditorTextarea(editor_id: String, value: String, placeholder: String, on_change: EventHandler<String>, #[props(optional)] on_submit: Option<EventHandler<String>>) -> Element {
```
- At line 79, find:
```rust
      id: "lx-md-editor",
```
- Replace with:
```rust
      id: "{editor_id}",
```
- Why: Each textarea now gets its own unique DOM ID.

### Step 6: Add auto-grow behavior to EditorTextarea
- After the `onkeydown` handler closing brace (line 90) and before the textarea's closing brace, add an `oninput` handler that auto-grows. However, `oninput` is already present at line 83. Instead, modify the existing `oninput` at line 83.
- At line 78-91, replace the entire textarea element:
```rust
    textarea {
      id: "{editor_id}",
      class: "w-full min-h-[8rem] p-3 bg-transparent outline-none text-sm font-mono text-[var(--on-surface)] placeholder:text-[var(--outline)]/40 resize-none overflow-hidden",
      value: "{value}",
      placeholder: "{placeholder}",
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
      onkeydown: move |evt| {
          if evt.modifiers().meta() && evt.key() == Key::Enter
            && let Some(ref handler) = on_submit
          {
            handler.call(value.clone());
          }
      },
    }
```
- Why: `resize-none overflow-hidden` disables manual resize and hides scrollbar. On every input, JS sets `height = 'auto'` then `height = scrollHeight` to fit content. The `max-h-80` is removed to allow unlimited growth.

### Step 7: Update insert_at_cursor to accept editor_id parameter
- At line 107, find:
```rust
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
```
- Replace with:
```rust
fn insert_at_cursor(editor_id: &str, value: &str, before: &str, after: &str, on_change: EventHandler<String>) {
  let val = value.to_string();
  let editor_id = editor_id.to_string();
  let before = before.to_string();
  let after = after.to_string();
  spawn(async move {
    let js = format!(
      "(function() {{ var el = document.getElementById('{}'); if (!el) return JSON.stringify({{start: -1, end: -1}}); return JSON.stringify({{start: el.selectionStart, end: el.selectionEnd}}); }})()",
      editor_id
    );
```
- At line 142-144, find:
```rust
    let set_cursor_js = format!(
      "setTimeout(function() {{ var el = document.getElementById('lx-md-editor'); if (el) {{ el.selectionStart = {new_cursor}; el.selectionEnd = {new_cursor}; el.focus(); }} }}, 0)"
    );
```
- Replace with:
```rust
    let set_cursor_js = format!(
      "setTimeout(function() {{ var el = document.getElementById('{editor_id}'); if (el) {{ el.selectionStart = {new_cursor}; el.selectionEnd = {new_cursor}; el.focus(); }} }}, 0)"
    );
```

### Step 8: Update ToolbarButtons to accept and forward editor_id
- At line 150, find:
```rust
fn ToolbarButtons(value: String, on_change: EventHandler<String>) -> Element {
```
- Replace with:
```rust
fn ToolbarButtons(editor_id: String, value: String, on_change: EventHandler<String>) -> Element {
```
- Update all 5 `insert_at_cursor` calls (lines 157, 163, 170, 177, 184) to pass `&editor_id` as the first argument. For example, line 157:
```rust
          move |_| insert_at_cursor(&editor_id, &v, "**", "**", on_change)
```
- Each closure captures `editor_id` by reference (it's a `String` owned by the function). Add `let eid = editor_id.clone();` before each button's `on_click` closure. Change each `let v = value.clone();` block to also clone the id:
```rust
        let eid = editor_id.clone();
        let v = value.clone();
        move |_| insert_at_cursor(&eid, &v, "**", "**", on_change)
```
- Repeat for all 5 toolbar buttons.
- Why: Each toolbar button must reference the correct editor instance.

## File Size Check
- `markdown_editor.rs`: was 201 lines, now ~220 lines (under 300)

## Post-Execution State

After WU-01 completes, the following signatures and state apply. Downstream WUs (WU-16, WU-17) should reference these.

- **MarkdownEditor**: signature unchanged. `editor_id` is generated internally via `use_hook(|| format!("lx-md-editor-{}", Uuid::new_v4().simple()))` and passed as a prop to child components. It is NOT a prop of MarkdownEditor itself.
- **EditorTextarea**: `fn EditorTextarea(editor_id: String, value: String, placeholder: String, on_change: EventHandler<String>, #[props(optional)] on_submit: Option<EventHandler<String>>) -> Element`
- **insert_at_cursor**: `fn insert_at_cursor(editor_id: &str, value: &str, before: &str, after: &str, on_change: EventHandler<String>)`
- **ToolbarButtons**: `fn ToolbarButtons(editor_id: String, value: String, on_change: EventHandler<String>) -> Element`
- **File line count**: `markdown_editor.rs` ~220 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compilation errors.
- Open the issues page which has both `NewIssueDialog` (with MarkdownEditor) and `CommentThread` (with MarkdownEditor). Confirm each editor's toolbar bold/italic/etc buttons affect only their own textarea.
- Type multi-line content in an editor and confirm the textarea grows vertically to fit.
