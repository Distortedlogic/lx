# WU-17: File Drag-and-Drop Upload

## Dependencies: WU-01 and WU-16 must run first.

## Fixes
- Fix 1: Add `ondragover` handler to `MarkdownEditor` outer div for drag visual feedback
- Fix 2: Add `ondragleave` handler to remove feedback on leave
- Fix 3: Add `ondrop` handler to receive dropped files
- Fix 4: Add drag-active visual state (border highlight) to the editor wrapper
- Fix 5: Add `on_files` callback prop to `MarkdownEditor` for parent components to handle file data
- Fix 6: Insert markdown image/file link placeholder into editor text on drop
- Fix 7: Add drag overlay feedback div with "Drop files here" text
- Fix 8: Wire `on_files` callback on `NewIssueDialog` description area
- Fix 9: Wire `on_files` callback on `CommentThread` editor area
- Fix 10: Add `dragging` signal state to `MarkdownEditor`
- Fix 11: Add CSS class for drag-active border highlight in `tailwind.css`
- Fix 12: Prevent default browser behavior on dragover to enable drop
- Fix 13: Use `evt.prevent_default()` on drop to prevent browser navigation
- Fix 14: Read file names from the drop event via JS eval with synchronous data capture
- Fix 15: Generate markdown link syntax for dropped files (`![filename](...)` for images, `[filename](...)` for others)
- Fix 16: Append placeholder links to the current editor value
- Fix 17: Support multiple files in a single drop
- Fix 18: Visual feedback: border turns primary color during drag
- Fix 19: Visual feedback: semi-transparent overlay with icon appears during drag

## Files Modified
- `crates/lx-desktop/src/components/markdown_editor.rs` (~225 lines post-WU-16)
- `crates/lx-desktop/src/pages/issues/new_issue.rs` (167 lines)
- `crates/lx-desktop/src/components/comment_thread.rs` (82 lines)
- `crates/lx-desktop/src/tailwind.css` (196 lines)

## Preconditions (post-WU-01 + WU-16 state)
- `markdown_editor.rs` is at ~225 lines after WU-16 (WU-16 split toolbar code into `markdown_toolbar.rs`)
- `MarkdownEditor` component signature (post-WU-16): `value`, `on_change`, `on_submit`, `placeholder`, `class`, `mention_candidates`
- `MarkdownEditor` generates `editor_id` internally via `use_hook`
- `EditorTextarea` component has props: `editor_id`, `value`, `placeholder`, `on_change`, `on_submit`, `on_mention_trigger`, `on_mention_dismiss`, `on_mention_nav`
- The outer `div` at line 26 has class `"flex flex-col border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden {extra_class}"`
- `new_issue.rs` has `MarkdownEditor` usage at line 101
- `comment_thread.rs` has `MarkdownEditor` usage at line 58
- Dioxus supports `ondragover`, `ondragleave`, `ondrop` event handlers on HTML elements
- Dioxus `DragEvent` does not directly expose `dataTransfer.files`; JS bridge is required

## Steps

### Step 1: Add drag-active CSS class to tailwind.css
- Open `crates/lx-desktop/src/tailwind.css`
- After the `.animate-transcript-enter` block (line 196), add:

```css

.drag-active {
  border-color: var(--primary) !important;
  background-color: color-mix(in srgb, var(--primary) 4%, transparent);
}
```

### Step 2: Add `on_files` prop and `dragging` state to `MarkdownEditor`
- Open `crates/lx-desktop/src/components/markdown_editor.rs`
- At the `MarkdownEditor` component signature (post-WU-16), find:
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
- Replace with:
```rust
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
```

- Add a `DroppedFile` struct at the top of the file (after imports):
```rust
#[derive(Clone, Debug, PartialEq)]
pub struct DroppedFile {
  pub name: String,
  pub mime_type: String,
  pub size: u64,
  pub data_base64: String,
}
```

- After the mention state signals (post-WU-16), add:
```rust
  let mut dragging = use_signal(|| false);
  let drag_class = if dragging() { "drag-active" } else { "" };
```

### Step 3: Install JS drop event listener with synchronous data capture
- After the state signals in `MarkdownEditor`, add a `use_effect` to install a JS listener that synchronously captures file data before `dataTransfer` is cleared:

```rust
  use_effect(|| {
    spawn(async {
      let js = r#"
        if (!window._dropListenerInstalled) {
          window._dropListenerInstalled = true;
          document.addEventListener('drop', function(e) {
            var files = e.dataTransfer ? e.dataTransfer.files : [];
            var captured = [];
            var pending = files.length;
            if (pending === 0) {
              window._lastDropFiles = '[]';
              return;
            }
            for (var i = 0; i < files.length; i++) {
              (function(file) {
                var reader = new FileReader();
                reader.onload = function() {
                  captured.push({
                    name: file.name,
                    mime: file.type || 'application/octet-stream',
                    size: file.size,
                    data: reader.result.split(',')[1] || ''
                  });
                  pending--;
                  if (pending === 0) {
                    window._lastDropFiles = JSON.stringify(captured);
                  }
                };
                reader.readAsDataURL(file);
              })(files[i]);
            }
          }, true);
          document.addEventListener('dragover', function(e) {
            e.preventDefault();
          }, true);
        }
      "#;
      let _ = document::eval(js).await;
    });
  });
```

### Step 4: Update the outer div class and add drag event handlers
- At the outer `div` in `MarkdownEditor`, find:
```rust
    div { class: "flex flex-col border border-[var(--outline-variant)]/30 rounded-lg overflow-hidden {extra_class}",
```
- Replace with:
```rust
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
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            let js = r#"
              (function() {
                var data = window._lastDropFiles || '[]';
                window._lastDropFiles = '[]';
                return data;
              })()
            "#;
            let files: Vec<serde_json::Value> = match document::eval(js).await {
              Ok(result) => {
                let s = result.to_string();
                let s = s.trim_matches('"');
                let unescaped = s.replace("\\\"", "\"").replace("\\\\", "\\");
                serde_json::from_str(&unescaped).unwrap_or_default()
              }
              Err(_) => vec![],
            };
            if !files.is_empty() {
              let dropped: Vec<DroppedFile> = files.iter().filter_map(|f| {
                Some(DroppedFile {
                  name: f["name"].as_str()?.to_string(),
                  mime_type: f["mime"].as_str().unwrap_or("application/octet-stream").to_string(),
                  size: f["size"].as_u64().unwrap_or(0),
                  data_base64: f["data"].as_str().unwrap_or("").to_string(),
                })
              }).collect();
              if let Some(ref handler) = on_files {
                handler.call(dropped.clone());
              }
              let mut links = String::new();
              for file in &dropped {
                let is_image = file.mime_type.starts_with("image/");
                if is_image {
                  links.push_str(&format!("\n![{}](upload://{})", file.name, file.name));
                } else {
                  links.push_str(&format!("\n[{}](upload://{})", file.name, file.name));
                }
              }
              let new_value = format!("{}{}", value, links);
              on_change.call(new_value);
            }
          });
        }
      },
```

### Step 5: Add drag overlay inside the outer div
- After the opening of the outer `div` (after the drag event handlers), before the toolbar div, add:

```rust
      if dragging() {
        div {
          class: "absolute inset-0 z-10 flex items-center justify-center bg-[var(--surface)]/80 pointer-events-none",
          div { class: "flex flex-col items-center gap-2 text-[var(--primary)]",
            span { class: "material-symbols-outlined text-3xl", "upload_file" }
            span { class: "text-sm font-medium", "Drop files here" }
          }
        }
      }
```

### Step 6: No structural changes to `EditorTextarea`
- The drag events are handled on the outer `MarkdownEditor` div, not on the `textarea` itself
- This means drag-and-drop works regardless of which mode (Edit/Preview/Split) is active

### Step 7: Wire `on_files` in `new_issue.rs`
- Open `crates/lx-desktop/src/pages/issues/new_issue.rs`
- At line 101, find:
```rust
          MarkdownEditor {
              value: description.read().clone(),
              on_change: move |val: String| description.set(val),
              placeholder: "Description (optional)".to_string(),
              class: "min-h-[120px]".to_string(),
          }
```
- Replace with:
```rust
          MarkdownEditor {
              value: description.read().clone(),
              on_change: move |val: String| description.set(val),
              placeholder: "Description (optional, drag files here)".to_string(),
              class: "min-h-[120px]".to_string(),
              on_files: move |files: Vec<crate::components::markdown_editor::DroppedFile>| {
                tracing::info!("Files dropped in new issue: {:?}", files.iter().map(|f| &f.name).collect::<Vec<_>>());
              },
          }
```

### Step 8: Wire `on_files` in `comment_thread.rs`
- Open `crates/lx-desktop/src/components/comment_thread.rs`
- At line 58, find:
```rust
        MarkdownEditor {
          value: body(),
          on_change: move |v: String| body.set(v),
          on_submit: move |v: String| submit(v),
          placeholder: "Leave a comment...".to_string(),
        }
```
- Replace with:
```rust
        MarkdownEditor {
          value: body(),
          on_change: move |v: String| body.set(v),
          on_submit: move |v: String| submit(v),
          placeholder: "Leave a comment (drag files here)...".to_string(),
          on_files: move |files: Vec<super::markdown_editor::DroppedFile>| {
            tracing::info!("Files dropped in comment: {:?}", files.iter().map(|f| &f.name).collect::<Vec<_>>());
          },
        }
```

## File Size Check
- `markdown_editor.rs`: was ~225 lines (post-WU-16), now ~290 lines (under 300) -- added ~65 lines for drag state, event handlers, overlay, JS bridge, and DroppedFile struct
- `new_issue.rs`: was 167 lines, now ~170 lines (under 300)
- `comment_thread.rs`: was 82 lines, now ~85 lines (under 300)
- `tailwind.css`: was 196 lines, now ~201 lines (under 300)

## Verification
- Run `just diagnose` to confirm compilation
- Launch the desktop app and test:
  1. Open new issue dialog -- the description editor should have "Description (optional, drag files here)" placeholder
  2. Drag a file from the desktop/file manager over the editor:
     - Border should turn green (primary color)
     - "Drop files here" overlay with upload icon should appear
  3. Move the dragged file away without dropping:
     - Border should revert to normal
     - Overlay should disappear
  4. Drop an image file (e.g., `screenshot.png`):
     - Editor text should gain `\n![screenshot.png](upload://screenshot.png)` appended
     - `on_files` callback should fire with `DroppedFile` containing the file's name, mime type, size, and base64 data
  5. Drop a non-image file (e.g., `report.pdf`):
     - Editor text should gain `\n[report.pdf](upload://report.pdf)` appended
  6. Drop multiple files at once:
     - Each file should get its own line with the appropriate link syntax
  7. Test in comment thread (open an issue detail page):
     - Same drag-and-drop behavior should work in the comment editor
  8. Test in Preview and Split modes:
     - Dragging over the editor area should still show the overlay (since it's on the outer div)
