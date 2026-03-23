# Goal

Fix the Browser, Editor, and Canvas pane types so they render usable content when created from the pane dropdown. The Browser pane currently uses iframe mode which is fundamentally blocked by Dioxus's navigation handler — switch it to CDP mode UI with a placeholder until a full CDP backend is implemented separately. The Editor pane currently renders an invisible empty div because the Rust side never passes file content to the TypeScript widget — fix the data flow and add visible empty-state styling. The Canvas widgets (log-viewer, markdown, json-viewer) mount correctly but show empty containers with no indication of purpose — add visible placeholder messages.

# Why

- Clicking "Browser" or "Editor" in the pane type dropdown creates panes that appear broken or invisible, making those pane types unusable
- The Browser pane uses an iframe to load external URLs, but Dioxus desktop's `with_navigation_handler` in `webview.rs` intercepts ALL http/https navigations (including iframe src changes on WebKitGTK) and opens them in the system browser instead — iframe-based browsing is architecturally impossible in this context
- The Editor pane's TypeScript widget reads `cfg.content` but the Rust `EditorView` only passes `filePath` and `language` — content is never provided, so the widget renders an empty contentEditable div that is invisible against the dark background
- The browser.ts already has a CDP mode (`cfg.mode === "cdp"`) that renders a canvas + toolbar without using iframes — this mode works within the navigation handler constraints
- The Canvas widget types (log-viewer, markdown, json-viewer) mount visible containers but show nothing — no placeholder, no indication of what the pane is or that it's waiting for data
- The pane divider drag logic in `terminals.rs` has a 30-line inline JS string (`build_drag_js`) that duplicates the already-existing `ts/widget-bridge/src/divider.ts` TypeScript implementation — the Rust side should call `WidgetBridge.runDividerDrag(dioxus)` instead of embedding raw JS

# What changes

**Browser pane — switch to CDP mode UI (Rust side):** In `crates/lx-desktop/src/terminal/view.rs`, update `BrowserView` to pass `"mode": "cdp"` in the widget config JSON. This makes browser.ts use `mountCdp()` instead of `mountIframe()`, rendering a canvas and toolbar without iframes.

**Browser pane — canvas placeholder (TypeScript side):** In `ts/widget-bridge/widgets/browser.ts`, in the `mountCdp` function, after creating the canvas, draw a centered placeholder message on the canvas (e.g., "CDP backend not connected" in outline-colored text on a dark background). This gives the user visible feedback that the pane loaded but the backend is pending. The toolbar with address bar, back/forward/refresh buttons still renders and is interactive — events are sent to Rust via `dx.send()` as the existing code already does.

**Editor pane — pass file content from Rust:** In `crates/lx-desktop/src/terminal/view.rs`, update `EditorView` to read the file from disk when `file_path` is non-empty using `std::fs::read_to_string`, and include the content in the widget config JSON as the `content` field. When `file_path` is empty, pass `content: ""`.

**Editor pane — visible empty state (TypeScript side):** In `ts/widget-bridge/widgets/editor.ts`, in the `mount` function, when content is empty, set a placeholder attribute or display placeholder text in outline color so the editor is visually distinguishable from the background. Add a visible left border in a subtle color to the container for orientation.

**Canvas widgets — visible empty state (TypeScript side):** In `ts/widget-bridge/widgets/log-viewer.ts`, `markdown.ts`, and `json-viewer.ts`, add a placeholder message in each widget's `mount` function so newly created canvas panes show what type of widget they are and that they're awaiting data.

**Divider drag — hoist inline JS to TypeScript:** In `crates/lx-desktop/src/pages/terminals.rs`, replace the `build_drag_js` function and its inline JS string with a call to `WidgetBridge.runDividerDrag(dioxus)` followed by sending the parameters (containerId, direction, parentStart, parentSize) via the eval channel. The TypeScript implementation already exists at `ts/widget-bridge/src/divider.ts` and is already exported on `window.WidgetBridge`.

# How it works

**Browser:** When a Browser pane is created, `BrowserView` passes `{ "url": "about:blank", "mode": "cdp" }` to the widget. The browser.ts widget calls `mountCdp()` which creates a toolbar (back/forward/refresh/address bar) and a canvas. The canvas immediately draws a "CDP backend not connected" placeholder. The toolbar is fully interactive — typing a URL and pressing Enter sends `{ type: "navigate", url }` to the Rust side via the widget bridge. A future work item will implement the CDP backend to capture browser frames and send them as base64 images to the canvas.

**Editor:** When an Editor pane is created with a file path, `EditorView` reads the file content from disk and passes it as `{ "content": "file contents...", "language": "plaintext", "filePath": "/path" }`. The widget renders the content in a monospace contentEditable div. When created with no file path (default), `EditorView` passes `{ "content": "", "language": "plaintext", "filePath": "" }` and the widget shows a visible empty editor with placeholder text.

# Files affected

- `crates/lx-desktop/src/terminal/view.rs` — update `BrowserView` to pass `mode: "cdp"` in config; update `EditorView` to read file content and pass it as `content` field
- `ts/widget-bridge/widgets/browser.ts` — add canvas placeholder drawing in `mountCdp` after canvas creation
- `ts/widget-bridge/widgets/editor.ts` — add visible empty-state styling and placeholder text when content is empty
- `ts/widget-bridge/widgets/log-viewer.ts` — add placeholder message in mount
- `ts/widget-bridge/widgets/markdown.ts` — add placeholder message in mount
- `ts/widget-bridge/widgets/json-viewer.ts` — add placeholder message in mount
- `crates/lx-desktop/src/pages/terminals.rs` — replace `build_drag_js` inline JS with call to `WidgetBridge.runDividerDrag`
- `ts/widget-bridge/src/divider.ts` — add `await dx.recv()` at end of `runDividerDrag` to keep the eval channel alive

# Task List

### Task 1: Switch BrowserView to CDP mode

**Subject:** Pass mode "cdp" in BrowserView widget config

**Description:** In `crates/lx-desktop/src/terminal/view.rs`, in the `BrowserView` component, change the `use_ts_widget` call from `use_ts_widget("browser", serde_json::json!({ "url": url }))` to `use_ts_widget("browser", serde_json::json!({ "url": url, "mode": "cdp" }))`.

**ActiveForm:** Switching BrowserView to CDP mode config

### Task 2: Add canvas placeholder in CDP mode

**Subject:** Draw placeholder message on CDP canvas when no backend is connected

**Description:** In `ts/widget-bridge/widgets/browser.ts`, in the `mountCdp` function, find line 205 where the container is appended to the DOM: `if (el) el.appendChild(container);`. Immediately AFTER that line and BEFORE the canvas click event listener on line 207, insert the placeholder drawing code. Set `canvas.width = 800` and `canvas.height = 600`. Get a 2d context with `canvas.getContext("2d")`. Fill the entire canvas with `ctx.fillStyle = "#0e0e0e"; ctx.fillRect(0, 0, 800, 600);`. Then set `ctx.fillStyle = "#757575"; ctx.font = "14px monospace"; ctx.textAlign = "center";` and call `ctx.fillText("CDP backend not connected", 400, 300);`. The canvas must be in the DOM before drawing so that getContext works correctly.

**ActiveForm:** Adding CDP canvas placeholder message

### Task 3: Pass file content from EditorView

**Subject:** Read file content in EditorView and pass to widget config

**Description:** In `crates/lx-desktop/src/terminal/view.rs`, in the `EditorView` component (around line 100), before the `use_ts_widget` call, add a synchronous file read: `let content = if file_path.is_empty() { String::new() } else { std::fs::read_to_string(&file_path).unwrap_or_default() };`. This is intentionally synchronous — it runs once during component init, and file reads for editor-sized files are fast enough to not block rendering. Then change the `use_ts_widget` config from `serde_json::json!({ "language": lang, "filePath": file_path })` to `serde_json::json!({ "content": content, "language": lang, "filePath": file_path })`. Do NOT attempt to make this async or use `use_future` — the widget needs the content in the initial config message.

**ActiveForm:** Reading file content in EditorView for widget config

### Task 4: Add visible empty-state styling to editor widget

**Subject:** Add placeholder text and border to editor when content is empty

**Description:** In `ts/widget-bridge/widgets/editor.ts`, in the `mount` function, after `container.spellcheck = false` (line 28), add `container.classList.add("border-l-2", "border-[var(--outline-variant)]");` for a visible left accent border via Tailwind (these classes are already in the compiled CSS from Rust file usage). Then add placeholder handling using JS focus/blur (Tailwind's `placeholder:` variant only works on input/textarea, not contentEditable divs, and CSS `:empty` breaks on contentEditable because browsers insert `<br>` elements). Create a `const placeholderText = "Empty — start typing";` constant. Create a function `updatePlaceholder` that checks if `container.textContent` is empty — if so, sets `container.style.color = "#757575"` and `container.textContent = placeholderText`; if not empty and the text equals `placeholderText`, do nothing. Add a `focus` event listener on the container that checks if `container.textContent === placeholderText` — if so, clears `container.textContent` to `""` and resets `container.style.color = "#e0e0e0"`. Add a `blur` event listener that calls `updatePlaceholder()` to re-show the placeholder if content is empty. Call `updatePlaceholder()` once immediately after setting up the listeners.

**ActiveForm:** Adding editor empty-state placeholder and border styling

### Task 5: Add placeholder messages to canvas widgets

**Subject:** Add visible empty-state placeholders to log-viewer, markdown, and json-viewer widgets

**Description:** In each of the three canvas widget files, add a placeholder message inside the `mount` function immediately after the container is appended to the DOM element:

**`ts/widget-bridge/widgets/log-viewer.ts`:** After `el.appendChild(container)` (line 60) and before the scroll listener (line 64), add a placeholder div: `const placeholder = document.createElement("div"); placeholder.textContent = "Log viewer — awaiting log entries"; placeholder.style.color = "#757575"; placeholder.style.padding = "8px"; placeholder.dataset.placeholder = "true"; container.appendChild(placeholder);`. In the `update` method, add this as the FIRST two lines of the function body (before the `if (Array.isArray(data))` branch on line 77): `const ph = state?.container.querySelector("[data-placeholder]"); if (ph) ph.remove();`.

**`ts/widget-bridge/widgets/markdown.ts`:** After `el.appendChild(container)` (line 33), add: `container.innerHTML = '<p style="color: #757575;">Markdown viewer — no content loaded</p>';`. The existing `update` method already replaces innerHTML, so the placeholder is automatically cleared on first update.

**`ts/widget-bridge/widgets/json-viewer.ts`:** After `el.appendChild(container)` (line 95), add: `const placeholder = document.createElement("div"); placeholder.textContent = "JSON viewer — no data loaded"; placeholder.style.color = "#757575"; container.appendChild(placeholder);`. The existing `update` method already clears innerHTML before rendering, so the placeholder is automatically cleared on first update.

**ActiveForm:** Adding empty-state placeholders to canvas widgets

### Task 6: Fix runDividerDrag eval channel lifetime in TypeScript

**Subject:** Add await dx.recv() to keep eval channel alive in runDividerDrag

**Description:** In `ts/widget-bridge/src/divider.ts`, in the `runDividerDrag` function (line 41), add `await dx.recv();` as the LAST line of the function body, after the `startDividerDrag(...)` call (line 48-54). The current `runDividerDrag` function resolves its promise immediately after calling `startDividerDrag`, which closes the Dioxus eval channel. Without this line, the mousemove/mouseup event listeners set up by `startDividerDrag` will silently fail when calling `dx.send()` because the channel is already closed. The `await dx.recv()` keeps the async function suspended (and the channel alive) until the Rust side drops the eval handle. The original inline JS in `build_drag_js` had this exact line (`await dioxus.recv()`) for this reason.

**ActiveForm:** Fixing runDividerDrag eval channel lifetime

### Task 7: Replace inline drag JS with WidgetBridge.runDividerDrag call

**Subject:** Hoist divider drag JS from Rust inline string to existing TypeScript function

**Description:** In `crates/lx-desktop/src/pages/terminals.rs`, delete the entire `build_drag_js` function (lines 244-274 — the function that returns a `format!` string containing inline JavaScript). In `render_divider_item`, replace the `onmousedown` handler's async block (lines 225-238). The current code calls `build_drag_js` to generate a JS string, then `document::eval(&js)`. Replace the two lines `let js = build_drag_js(&cid, is_h, p_start, p_size);` and `let mut eval = document::eval(&js);` with: `let mut eval = document::eval("WidgetBridge.runDividerDrag(dioxus)");` followed by `let _ = eval.send(serde_json::json!({ "containerId": cid, "direction": if is_h { "horizontal" } else { "vertical" }, "parentStart": p_start, "parentSize": p_size }));`. Keep the existing `while let Ok(msg) = eval.recv::<serde_json::Value>().await` loop and its match arms (`Some("ratio")` and the `_ => break` fallthrough) unchanged — the TypeScript `runDividerDrag` in `ts/widget-bridge/src/divider.ts` already sends the same `{ type: "ratio", value }` and `{ type: "done" }` messages that the Rust loop expects. Task 6 must be completed first — without the `await dx.recv()` fix, this task would break divider dragging.

**ActiveForm:** Replacing inline drag JS with WidgetBridge.runDividerDrag call

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

