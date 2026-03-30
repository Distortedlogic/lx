# Unit 03: MarkdownEditor Cursor-Position Insertion

## Goal

Change all 5 toolbar buttons in `MarkdownEditor` to insert formatting at the textarea cursor position instead of appending to the end of the value.

## Preconditions

- No other units need to be complete first.
- The `document::eval()` JS interop pattern is already established in the codebase (see `crates/lx-desktop/src/components/scroll_to_bottom.rs`, `crates/lx-desktop/src/components/copy_text.rs`).

## Files to Modify

- `crates/lx-desktop/src/components/markdown_editor.rs` (currently 158 lines)

## Context: Current Implementation

The current `ToolbarButtons` component (lines 107-147) has 5 buttons that each call `on_change` with `format!("{v}<suffix>")` where `v` is the full current value. This appends to the end:

```rust
// Bold: format!("{v}****")
// Italic: format!("{v}**")
// Code: format!("{v}\n```\n\n```")
// Link: format!("{v}[text](url)")
// Heading: format!("{v}\n## ")
```

The `EditorTextarea` component (lines 76-92) renders a `<textarea>` with no `id` attribute.

## Context: JS Interop Pattern

From `scroll_to_bottom.rs`, the established pattern is:

```rust
use dioxus::prelude::*;
// ...
spawn(async move {
    let js = format!("...");
    let _ = document::eval(&js).await;
});
```

The `document::eval()` function returns a `Result` that can contain a string value from JS.

## Steps

### Step 1: Add a stable ID to the textarea

In the `EditorTextarea` component (line 78-91), add an `id` attribute to the `<textarea>` so JS can find it. Use a fixed ID since there will typically be one editor visible at a time.

In the `textarea` RSX block inside `EditorTextarea`, add:

```rust
textarea {
  id: "lx-md-editor",
  class: "w-full min-h-[8rem] max-h-80 p-3 bg-transparent outline-none text-sm font-mono text-[var(--on-surface)] placeholder:text-[var(--outline)]/40 resize-y",
  // ... rest unchanged
}
```

### Step 2: Create a helper function for cursor-position insertion

Add this function above `ToolbarButtons` (around line 106):

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
      "setTimeout(function() {{ var el = document.getElementById('lx-md-editor'); if (el) {{ el.selectionStart = {pos}; el.selectionEnd = {pos}; el.focus(); }} }}, 0)",
      pos = new_cursor
    );
    let _ = document::eval(&set_cursor_js).await;
  });
}
```

This function:
1. Reads `selectionStart` and `selectionEnd` from the textarea via JS.
2. Wraps the selected text (or inserts at cursor if no selection) with `before` and `after` strings.
3. Calls `on_change` with the new value.
4. Restores cursor position after the insertion via a `setTimeout` (needed because Dioxus re-renders the textarea).

### Step 3: Replace ToolbarButtons to use insert_at_cursor

Replace the entire `ToolbarButtons` component (lines 107-147) with:

```rust
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
```

### Step 4: Behavioral details for each button

| Button | Before | After | Behavior with selected text "hello" | Behavior without selection |
|--------|--------|-------|-------------------------------------|--------------------------|
| Bold | `**` | `**` | `**hello**` | `****` (cursor between) |
| Italic | `*` | `*` | `*hello*` | `**` (cursor between) |
| Code | `\n```\n` | `\n``` ` | `\n```\nhello\n``` ` | `\n```\n\n``` ` (cursor on empty line) |
| Link | `[` | `](url)` | `[hello](url)` | `[](url)` (cursor between brackets) |
| Heading | `\n## ` | `` | `\n## hello` | `\n## ` (cursor at end) |

### Step 5: Ensure serde_json is available

Check that `serde_json` is already a dependency in `crates/lx-desktop/Cargo.toml`. It is (line 48: `serde_json = { workspace = true }`). No change needed.

## Verification

1. Run `just diagnose` to confirm no compilation errors or warnings.
2. Open the MarkdownEditor in the app (e.g., in a comment thread or issue dialog).
3. Type some text: `Hello world this is a test`.
4. Place cursor between "world" and "this" (click there).
5. Click the Bold button. Verify `****` is inserted at the cursor position, not at the end.
6. Select the word "test" by double-clicking it.
7. Click the Italic button. Verify the selected text becomes `*test*`.
8. Place cursor at the beginning of the text. Click Heading. Verify `\n## ` is inserted at the beginning.
9. Select some text and click the Link button. Verify it wraps as `[selected](url)`.
10. Click the Code button with a selection. Verify it wraps in triple backtick fences.
