# Unit 5: terminal/view.rs — use_resource + use_effect → use_loader in EditorView

## Violation

Two rules violated in `EditorView` component:

1. Rule: "No use_resource in fullstack apps" — `use_resource` on line 97 loads file content.
2. Rule: "No use_effect reacting to resource" — `use_effect` on line 104 reads the resource result to send content to a widget.

File: `crates/lx-desktop/src/terminal/view.rs`, lines 94-145 (`EditorView` component only).

## Current Code (lines 96-108)

```rust
pub fn EditorView(editor_id: String, file_path: String, language: Option<String>) -> Element {
  let fp = file_path.clone();
  let content = use_resource(move || {
    let fp = fp.clone();
    async move { if fp.is_empty() { String::new() } else { tokio::fs::read_to_string(&fp).await.unwrap_or_default() } }
  });

  let (element_id, widget) = use_ts_widget("editor", serde_json::json!({}));

  use_effect(move || {
    if let Some(text) = content.value().read().as_ref() {
      widget.send_update(serde_json::json!({ "content": text }));
    }
  });
```

## Approach

`use_loader` accepts arbitrary async closures, not just server functions. Its signature requires `Future<Output = Result<T, E>>` where `T: PartialEq + Serialize + DeserializeOwned` and `E: Into<CapturedError>`. `String` satisfies `T`'s bounds. `tokio::fs::read_to_string` returns `Result<String, io::Error>`, which satisfies the `Result` requirement directly.

After converting to `use_loader`, the `?` operator suspends the component until content loads. The content is then available synchronously, but we still need `use_effect` to send it to the widget (this is a side effect that must run after render, and re-run when content changes — e.g. if `file_path` prop changes).

## Required Changes

Replace lines 96-108 with:

```rust
  let fp = file_path.clone();
  let content = use_loader(move || {
    let fp = fp.clone();
    async move {
      if fp.is_empty() {
        Ok(String::new())
      } else {
        tokio::fs::read_to_string(&fp).await
      }
    }
  })?;

  let (element_id, widget) = use_ts_widget("editor", serde_json::json!({}));

  use_effect(move || {
    let text = content.read();
    widget.send_update(serde_json::json!({ "content": *text }));
  });
```

Changes:
- `use_resource` → `use_loader` with `?` for suspension
- Closure now returns `Result<String, io::Error>` instead of bare `String` — the `Ok(String::new())` path wraps the empty-file case, the `else` path returns the `Result` from `tokio::fs::read_to_string` directly
- `use_effect` body simplified: `content.read()` returns `Ref<String>` (Loader implements `Readable`), no more `if let Some(...)` check since the loader guarantees the value is present after suspension

## Files Modified

- `crates/lx-desktop/src/terminal/view.rs` — only the `EditorView` component (lines 94-108). Do not modify `TerminalView`, `AgentView`, `CanvasView`, or `ChartView`.

## Verification

Run `just diagnose` and confirm no errors in `view.rs`.
