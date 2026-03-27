# Unit 5: terminal/view.rs — use_resource + use_effect → use_loader in EditorView

## Violation

Two rules violated in `EditorView` component:

1. Rule: "No use_resource in fullstack apps" — `use_resource` on line 97 loads file content. Must use `use_loader` for data loading.
2. Rule: "No use_effect reacting to use_action" (same anti-pattern with resources) — `use_effect` on line 104 reads the resource result to send it to a widget. This is the react-to-completion anti-pattern.

File: `crates/lx-desktop/src/terminal/view.rs`, lines 94-108 (`EditorView` component).

## Current Code (lines 94-108)

```rust
#[component]
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

## Context

`EditorView` loads a file's text content and sends it to a TypeScript widget for rendering. The file content is loaded via `use_resource` (async tokio fs read), then a `use_effect` watches for the resource to complete and sends the content to the widget.

This is a desktop app — `use_loader` works for desktop too (not just fullstack server). The file read is data loading that should suspend the component until ready.

However, there is a subtlety: `tokio::fs::read_to_string` is a local filesystem read, not a server function. `use_loader` is designed for server functions that support SSR serialization. For local-only async operations in desktop apps, `use_resource` may actually be appropriate.

## Required Changes

### Option A: If use_loader works with local async (preferred per audit rule)

Replace lines 96-108 with:

```rust
  let fp = file_path.clone();
  let content = use_loader(move || {
    let fp = fp.clone();
    async move { if fp.is_empty() { String::new() } else { tokio::fs::read_to_string(&fp).await.unwrap_or_default() } }
  })?;

  let (element_id, widget) = use_ts_widget("editor", serde_json::json!({}));

  let text = content.read();
  widget.send_update(serde_json::json!({ "content": *text }));
```

This removes the `use_effect` entirely — the content is available synchronously after `use_loader` suspends, so we can send it to the widget directly.

### Option B: If use_loader doesn't work for local async (fallback)

Keep `use_resource` but convert the `use_effect` pattern. Replace the `use_effect` (lines 104-108) with a `use_future` that awaits the resource:

```rust
  use_future(move || async move {
    loop {
      if let Some(text) = content.value().read().as_ref() {
        widget.send_update(serde_json::json!({ "content": text }));
        break;
      }
      tokio::task::yield_now().await;
    }
  });
```

### Verify which option is valid

Before implementing, check whether `use_loader` accepts a non-server-function closure. Read the Dioxus `use_loader` source/docs. If it requires a server function (annotated with `#[get]`/`#[post]`), then use Option B.

## Files Modified

- `crates/lx-desktop/src/terminal/view.rs` — only the `EditorView` component (lines 94-108). Do not modify `TerminalView`, `AgentView`, `CanvasView`, or `ChartView`.

## Verification

Run `just diagnose` and confirm no errors in `view.rs`.
