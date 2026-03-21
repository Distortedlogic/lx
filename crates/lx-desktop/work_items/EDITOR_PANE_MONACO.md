# Editor Pane — Monaco Implementation

## Goal

Implement the Editor pane surface using Monaco Editor for syntax-highlighted code editing within panes. File contents load from the server via a file I/O server function, edits write back on Ctrl+S, and the widget bridge handles all communication. This replaces the stub EditorView with a functional code editor.

## Why

- The tri-surface model (editor + terminal + browser) is the standard agent-native workspace pattern identified across Antigravity, Cursor, and Devin
- Monaco is the VS Code editor engine — syntax highlighting, multi-cursor, find/replace, minimap, and keybindings with zero custom code
- Agents need to display code in a real editor rather than cat output in a terminal pane

## How it works

EditorView reads file content via a server function, then calls use_ts_widget("editor", config) with the content, language, and file path. The TS widget creates a Monaco instance configured with the industrial console dark theme. Monaco's Ctrl+S keybinding (via editor.addCommand) sends a save message containing the full editor content back to Rust. The Rust side receives the save and writes the file via a server function.

## Files affected

| File | Change |
|------|--------|
| `ts/desktop/package.json` | Add monaco-editor dependency |
| `ts/desktop/src/widgets/editor.ts` | New file: Monaco editor widget |
| `ts/desktop/src/index.ts` | Side-effect import to register editor widget |
| `apps/desktop/src/terminal/view.rs` or new `editor_view.rs` | Replace stub EditorView with real implementation |
| `apps/desktop/src/server/files.rs` | New file: read_file and write_file server functions |
| `apps/desktop/src/server/mod.rs` | Export files module |

## Task List

### Task 1: Add monaco-editor dependency

Run `pnpm add monaco-editor` in the `ts/desktop` directory. Verify the dependency appears in `ts/desktop/package.json`.

### Task 2: Create editor widget TypeScript implementation

Create `ts/desktop/src/widgets/editor.ts` implementing the Widget interface. The mount function creates a container div with full height and width. Import monaco from monaco-editor. Define a custom theme via monaco.editor.defineTheme matching industrial console: background "#1a1a1a" (surface-lowest), foreground "#e0e0e0" (on-surface), selection background "rgba(255,184,123,0.2)" (primary accent at 20% opacity), line number foreground "#666". Call monaco.editor.create on the container with value set to config.content, language set to config.language, theme set to the custom theme name, minimap enabled false, automaticLayout true, fontSize 14, scrollBeyondLastLine false. Register a Ctrl+S command via editor.addCommand using KeyMod.CtrlCmd bitwise-or KeyCode.KeyS that calls dx.send with type "save" and content from editor.getValue(). The update function: if data has a content field, call editor.setValue(data.content). The resize function calls editor.layout(). The dispose function calls editor.dispose() and removes the container. Import registerWidget and call registerWidget("editor", editorWidget).

### Task 3: Register editor widget in exports

Edit `ts/desktop/src/index.ts`. Add a side-effect import for the editor widget file. Run `just ts-build` to verify the bundle compiles.

### Task 4: Add file I/O server functions

Create `apps/desktop/src/server/files.rs`. Add a server function read_file that takes a path String parameter and returns Result of String. It reads the file using tokio::fs::read_to_string. Add a server function write_file that takes path (String) and content (String) parameters and returns Result of unit. It writes using tokio::fs::write. Both functions should canonicalize the path and verify it does not traverse above the current working directory to prevent path traversal attacks. Register the files module in `apps/desktop/src/server/mod.rs`.

### Task 5: Implement EditorView Rust component

Replace the stub EditorView. The component takes editor_id (String), file_path (String), and language (Option of String) as props. Use use_resource or use_future to call the read_file server function on mount, storing the result in a signal. Once content is available, call use_ts_widget("editor", serde_json::json!({ "content": content, "language": language.clone().unwrap_or("plaintext".into()), "filePath": file_path })) to get element_id and widget handle. Spawn an async loop on widget.recv: when the message type is "save", extract the content string and call the write_file server function with file_path and content. The component renders a div with id element_id and class "w-full h-full". While file content is loading, show a "Loading..." placeholder.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
