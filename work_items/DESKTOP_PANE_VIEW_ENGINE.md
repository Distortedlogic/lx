# Goal

Fix all pane views that discard widget messages (EditorView, AgentView, CanvasView), make ChartView useful on creation, add a file-path input bar for Editor panes in the toolbar, and wire the terminal toolbar badge to actual PTY notification state.

# Why

- EditorView creates a `_widget` (unused) and reads files synchronously in the render path — the TypeScript editor widget sends save/cursor events that are silently dropped, and `std::fs::read_to_string` blocks the UI thread
- AgentView creates a `_widget` (unused) and ignores its `session_id`/`model` props — the TypeScript agent widget has a full chat UI that sends `user_message` events but the Rust side never processes them
- CanvasView creates a `_widget` (unused) — canvas widget events are silently dropped
- ChartView opens blank because `make_default` sets `chart_json` to an empty string and the `use_effect` early-returns on empty strings — there is no UI to provide chart data
- Editor panes created from the dropdown open with empty `file_path` and no way to specify one — unlike Browser panes which have a URL bar in the toolbar
- Terminal toolbar badge always shows "ACTIVE" regardless of whether the PTY process has exited

# Architecture

All pane views live in `src/terminal/view.rs`. The established pattern is `TerminalView`: create a widget via `use_ts_widget`, then run a `use_future` with a `tokio::select!` loop that handles both widget-to-Rust messages (`widget.recv`) and Rust-to-widget data. The `_widget` views need to follow this same pattern.

The agent.ts widget (at `dioxus-common/ts/widget-bridge/widgets/agent.ts`) already has a complete chat UI. It sends `{ type: "user_message", content }` when the user types. It expects these update messages from Rust:
- `{ type: "assistant_chunk", text }` — streaming text
- `{ type: "assistant_done" }` — end of response
- `{ type: "tool_call", call_id, name, arguments }` — tool use approval
- `{ type: "error", message }` — error display

The editor.ts widget delegates to a shared editor module. It accepts `{ content, language, filePath }` in config and sends cursor/content events.

The `StatusBarState` context (from WU-1) must already be provided before this unit runs, as EditorView will write cursor position to it.

# Files Affected

| File | Change |
|------|--------|
| `src/panes.rs` | Change Chart default to include sample ECharts JSON |
| `src/terminal/view.rs` | Rewrite EditorView, AgentView, CanvasView with message loops |
| `src/terminal/toolbar.rs` | Add file-path input for Editor panes; fix terminal badge |

# Task List

### Task 1: Provide sample chart JSON in make_default

**Subject:** Make ChartView render a visible chart on creation instead of blank

**Description:** Edit `crates/lx-desktop/src/panes.rs`. In the `make_default` method, find the `PaneKind::Chart` arm (line 67):

```rust
PaneKind::Chart => Self::Chart { id, chart_json: String::new(), title: None, name: None },
```

Replace `String::new()` with a sample ECharts JSON string. Use this minimal bar chart:

```rust
PaneKind::Chart => Self::Chart {
    id,
    chart_json: r#"{"xAxis":{"type":"category","data":["A","B","C","D","E"]},"yAxis":{"type":"value"},"series":[{"data":[120,200,150,80,70],"type":"bar"}]}"#.into(),
    title: None,
    name: None,
},
```

This ensures the `use_effect` in ChartView (which early-returns on empty `chart_json`) actually executes and renders a chart via `DioxusCharts.initChart`.

**ActiveForm:** Adding sample chart JSON to make_default

---

### Task 2: Rewrite EditorView with async file loading and message loop

**Subject:** Replace sync file read with use_resource and add use_future for widget events

**Description:** Edit `crates/lx-desktop/src/terminal/view.rs`. Replace the entire `EditorView` component (lines 91-110).

The new implementation:

1. Use `use_resource` to load file content asynchronously. The resource depends on `file_path`:

```rust
let fp = file_path.clone();
let content = use_resource(move || {
    let fp = fp.clone();
    async move {
        if fp.is_empty() {
            String::new()
        } else {
            tokio::fs::read_to_string(&fp).await.unwrap_or_default()
        }
    }
});
```

2. Create the widget: `let (element_id, widget) = use_ts_widget("editor", serde_json::json!({}));`

3. Use a `use_effect` to send content to the widget once the resource resolves:

```rust
use_effect(move || {
    if let Some(text) = content.value().read().as_ref() {
        widget.send_update(serde_json::json!({ "content": text }));
    }
});
```

4. Add a `use_future` message loop that handles widget events:

```rust
use_future(move || async move {
    loop {
        let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };
        match msg["type"].as_str() {
            Some("cursor") => {
                let line = msg["line"].as_u64().unwrap_or(1) as u32;
                let col = msg["col"].as_u64().unwrap_or(1) as u32;
                let ctx = use_context::<crate::contexts::status_bar::StatusBarState>();
                ctx.update_cursor(line, col);
            }
            Some("save") => {
                if let Some(text) = msg["content"].as_str() {
                    let fp = file_path.clone();
                    if !fp.is_empty() {
                        let text = text.to_owned();
                        let _ = tokio::fs::write(&fp, &text).await;
                    }
                }
            }
            _ => {}
        }
    }
});
```

5. Render the same div: `rsx! { div { id: "{element_id}", class: "w-full h-full bg-[var(--surface-container-lowest)]" } }`

Keep the same component signature: `pub fn EditorView(editor_id: String, file_path: String, language: Option<String>) -> Element`.

**ActiveForm:** Rewriting EditorView with async file loading and widget message loop

---

### Task 3: Wire AgentView to handle chat messages via widget bridge

**Subject:** Connect AgentView to the agent widget's chat protocol using voice_backend::ClaudeCliBackend

**Description:** Edit `crates/lx-desktop/src/terminal/view.rs`. Replace the entire `AgentView` component (lines 112-122).

The new implementation:

1. Pass `session_id` and `model` to the widget config:

```rust
let (element_id, widget) = use_ts_widget(
    "agent",
    serde_json::json!({ "sessionId": session_id, "model": model }),
);
```

2. Add a `use_future` message loop. The agent.ts widget sends `{ type: "user_message", content }` when the user submits text. The Rust side processes it via `ClaudeCliBackend` and streams the response back:

```rust
use_future(move || async move {
    loop {
        let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };
        match msg["type"].as_str() {
            Some("user_message") => {
                let content = msg["content"].as_str().unwrap_or("").to_owned();
                if content.is_empty() { continue; }
                match crate::voice_backend::ClaudeCliBackend.query(&content).await {
                    Ok(response) => {
                        widget.send_update(serde_json::json!({
                            "type": "assistant_chunk",
                            "text": response,
                        }));
                        widget.send_update(serde_json::json!({ "type": "assistant_done" }));
                    }
                    Err(e) => {
                        widget.send_update(serde_json::json!({
                            "type": "error",
                            "message": format!("{e:#}"),
                        }));
                    }
                }
            }
            Some("tool_decision") => {}
            _ => {}
        }
    }
});
```

This uses `ClaudeCliBackend` which is the existing agent backend (it shells out to the `claude` CLI). The response comes back as a single string (not streaming), so it is sent as one `assistant_chunk` followed by `assistant_done`. The `tool_decision` arm is a no-op because `ClaudeCliBackend` returns plain text without tool use. The arm exists so the match is explicit about the agent.ts protocol.

3. Add the import at the top of view.rs: `use common_voice::AgentBackend as _;`

4. Render: `rsx! { div { id: "{element_id}", class: "w-full h-full bg-[var(--surface-container)]" } }`

Keep the same component signature: `pub fn AgentView(agent_id: String, session_id: String, model: String) -> Element`.

**ActiveForm:** Wiring AgentView to agent chat widget protocol

---

### Task 4: Add CanvasView message loop

**Subject:** Handle widget events in CanvasView instead of discarding them

**Description:** Edit `crates/lx-desktop/src/terminal/view.rs`. Replace the entire `CanvasView` component (lines 124-134).

The new implementation:

1. Keep the existing widget creation: `let (element_id, widget) = use_ts_widget(&widget_type, &config);`

2. Add a `use_future` message loop that forwards events to the activity log:

```rust
use_future(move || async move {
    loop {
        let Ok(msg) = widget.recv::<serde_json::Value>().await else { break };
        match msg["type"].as_str() {
            Some("content_update") => {}
            Some("interaction") => {}
            _ => {}
        }
    }
});
```

The match arms are no-ops. Canvas widget event handling is type-specific and outside this unit's scope. The critical fix is that the widget message channel is now drained instead of silently accumulating in the recv buffer. Without this loop, the channel buffer grows unboundedly for long-lived canvas panes.

3. Render: `rsx! { div { id: "{element_id}", class: "w-full h-full bg-[var(--surface-container)]" } }`

Keep the same component signature.

**ActiveForm:** Adding CanvasView message loop

---

### Task 5: Add file-path input bar for Editor panes in PaneToolbar

**Subject:** Mirror the Browser URL bar pattern for Editor panes

**Description:** Edit `crates/lx-desktop/src/terminal/toolbar.rs`. In the `PaneToolbar` component, find the `left_section` match (starting at line 34). Currently there are two arms: `DesktopPane::Browser { .. }` renders the URL bar + nav buttons, and `_` renders a text label.

Add a new arm between them for `DesktopPane::Editor { .. }`:

```rust
DesktopPane::Editor { file_path, .. } => {
    let mut path_input = use_signal(|| file_path.clone());
    rsx! {
        span { class: "text-[10px] text-[var(--outline)] uppercase tracking-wider mr-1", "FILE" }
        input {
            class: "flex-1 bg-[var(--surface-container-lowest)] rounded text-xs px-1.5 py-0.5 outline-none focus:bg-[var(--surface-container-low)] focus:border-b focus:border-[var(--primary)] transition-colors duration-150 font-mono",
            value: "{path_input}",
            placeholder: "Enter file path...",
            oninput: move |evt| path_input.set(evt.value()),
            onkeydown: move |evt: KeyboardEvent| {
                if evt.key() == Key::Enter {
                    let new_id = uuid::Uuid::new_v4().to_string();
                    let new_pane = PaneNode::Leaf(DesktopPane::Editor {
                        id: new_id,
                        file_path: path_input(),
                        language: None,
                        name: None,
                    });
                    on_convert.call(new_pane);
                }
            },
        }
    }
},
```

This renders a file path input. On Enter, it converts the pane to a new Editor with the entered path, which triggers EditorView to load the file via `use_resource`.

The `_` wildcard arm stays as the final catchall for Terminal, Agent, Canvas, Chart panes.

**ActiveForm:** Adding file-path input bar for Editor panes

---

### Task 6: Wire terminal badge to notification state

**Subject:** Replace hardcoded ACTIVE badge with notification-aware status

**Description:** Edit `crates/lx-desktop/src/terminal/toolbar.rs`. Find the terminal badge rendering (lines 149-154):

```rust
if pane.kind() == PaneKind::Terminal {
    StatusBadge {
        label: "ACTIVE".to_string(),
        variant: BadgeVariant::Active,
    }
}
```

Replace with logic that reads the pane's notification from TabsState:

```rust
if pane.kind() == PaneKind::Terminal {
    {
        let tabs = crate::terminal::use_tabs_state();
        let notification = tabs.read().get_notification(pane.pane_id()).cloned();
        let (label, variant) = match notification.as_ref().map(|n| n.level) {
            Some(common_pane_tree::NotificationLevel::Success) => ("EXITED".to_string(), BadgeVariant::Idle),
            Some(common_pane_tree::NotificationLevel::Error) => ("ERROR".to_string(), BadgeVariant::Idle),
            _ => ("ACTIVE".to_string(), BadgeVariant::Active),
        };
        rsx! { StatusBadge { label, variant } }
    }
}
```

The `TerminalView` already sets a `Success` notification when the PTY closes (view.rs line 51-54). This change makes the badge reflect that state. The `get_notification` call returns `Option<&PaneNotification>` — we clone it to avoid holding the read lock in the rsx block.

Add `use common_pane_tree::NotificationLevel;` to the imports if not already present — but since we're using the fully qualified path `common_pane_tree::NotificationLevel` in the match, no additional import is needed.

**ActiveForm:** Wiring terminal badge to notification state

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

1. **Call `complete_task` after each task.**
2. **Call `next_task` to get the next task.**
3. **Do not add, skip, reorder, or combine tasks.**
4. **Tasks are implementation-only.**

---

## Task Loading Instructions

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_PANE_VIEW_ENGINE.md" })
```
