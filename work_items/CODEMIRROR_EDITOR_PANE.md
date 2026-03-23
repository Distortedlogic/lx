# Goal

Replace the contentEditable div editor widget with CodeMirror 6, providing a proper code editor with syntax highlighting, line numbers, bracket matching, search/replace, and a dark theme matching lx's palette. CodeMirror is installed as an npm dependency in the widget-bridge TS package and bundled via vite into the existing IIFE output. No Rust changes needed — the widget protocol (content in config, save via dx.send) stays the same.

# Why

- The current editor is a contentEditable div with no syntax highlighting, no line numbers, no code editing features — it's a text area, not an editor
- Users expect IDE-level editing when they open an "Editor" pane
- CodeMirror 6 is the correct choice over Monaco: it's modular (~200-300KB vs Monaco's 5-10MB), bundles cleanly as IIFE (proven by xterm.js already bundling as ESM -> IIFE in the same vite config), and has proper tree-shaking
- The integration pattern is already proven: xterm.js in `ts/widget-bridge/src/terminal.ts` demonstrates instance management, CSS injection via `?inline`, and widget lifecycle — CodeMirror follows the same pattern exactly

# What changes

**Install CodeMirror packages:** Add `codemirror`, `@codemirror/view`, `@codemirror/state`, `@codemirror/theme-one-dark`, and language packs (`lang-javascript`, `lang-python`, `lang-json`, `lang-html`, `lang-css`, `lang-rust`) to `ts/widget-bridge/package.json`.

**New implementation file `ts/widget-bridge/src/editor.ts`:** Following `src/terminal.ts` pattern — instance Map keyed by element ID, `mountEditor(elementId, config, dx)` creates an `EditorView` with extensions (basicSetup, language detection from file extension, oneDark theme), `updateEditor(elementId, content)` replaces document content, `disposeEditor(elementId)` destroys the view. Language detection maps file extension from `config.filePath` to the appropriate language pack.

**Rewrite `ts/widget-bridge/widgets/editor.ts`:** Replace contentEditable implementation with calls to `mountEditor`/`updateEditor`/`disposeEditor`. Ctrl+S handler uses `EditorView.state.doc.toString()` to get content and sends via `dx.send({ type: "save", content })`.

# How it works

1. User creates an Editor pane -> `EditorView` (Rust) reads file content from disk, passes `{ content, language, filePath }` to widget
2. Widget `mount` calls `mountEditor` which creates a CodeMirror `EditorView` with:
   - `basicSetup` extension (line numbers, bracket matching, indentation, search, etc.)
   - Language extension detected from `filePath` extension (`.js` -> lang-javascript, `.py` -> lang-python, `.rs` -> lang-rust, `.json` -> lang-json, etc.)
   - `oneDark` theme customized to match lx's color palette
3. Content is set from config
4. User edits code with full IDE features
5. Ctrl+S -> `dx.send({ type: "save", content })` -> Rust can write back to disk

# Files affected

- EDIT: `ts/widget-bridge/package.json` — add codemirror and @codemirror/* dependencies
- NEW: `ts/widget-bridge/src/editor.ts` — CodeMirror mounting/update/dispose implementation
- EDIT: `ts/widget-bridge/widgets/editor.ts` — replace contentEditable with CodeMirror widget calls

# Task List

### Task 1: Install CodeMirror 6 packages

**Subject:** Add CodeMirror 6 dependencies to widget-bridge

**Description:** Run `pnpm add codemirror @codemirror/view @codemirror/state @codemirror/theme-one-dark @codemirror/lang-javascript @codemirror/lang-python @codemirror/lang-json @codemirror/lang-html @codemirror/lang-css @codemirror/lang-rust` in the `ts/widget-bridge/` directory. Note: `basicSetup` is exported from the `codemirror` package (not `@codemirror/basic-setup`, which was the old name).

**ActiveForm:** Installing CodeMirror 6 packages

### Task 2: Create CodeMirror editor implementation

**Subject:** Implement editor mount/update/dispose following terminal.ts pattern

**Description:** Create `ts/widget-bridge/src/editor.ts`. Import `EditorView`, `keymap` from `@codemirror/view`, `EditorState` from `@codemirror/state`, `basicSetup` from `codemirror`, `oneDark` from `@codemirror/theme-one-dark`, `import type { Dioxus } from "./types"`, and all language packs. Create a `Map<string, EditorView>` for instance tracking. Export four functions:

`mountEditor(elementId, config, dx)`: (1) detect language from file extension, (2) build extensions array with `basicSetup`, `oneDark`, language extension, `EditorView.theme([{ "&": { height: "100%" }, ".cm-scroller": { overflow: "auto" } }])`, and `keymap.of([{ key: "Mod-s", run: (view) => { dx.send({ type: "save", content: view.state.doc.toString() }); return true; } }])`. Note: the `run` callback receives the `EditorView` as its first parameter from CodeMirror — do NOT reference an outer `view` variable. (3) Create state via `EditorState.create({ doc: config.content ?? "", extensions })`, (4) create `new EditorView({ state, parent: document.getElementById(elementId) })`, store in map.

`updateEditor(elementId, content)`: get view from map, call `view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: content } })` to replace the entire document.

`resizeEditor(elementId)`: get view from map, call `view.requestMeasure()` so CodeMirror recalculates layout after container resize.

`disposeEditor(elementId)`: get view from map, call `view.destroy()`, delete from map.

No CSS injection file is needed — CodeMirror 6 injects all styling via JavaScript.

**ActiveForm:** Implementing CodeMirror editor mount/update/dispose

### Task 3: Rewrite editor widget to use CodeMirror

**Subject:** Replace contentEditable editor with CodeMirror implementation

**Description:** In `ts/widget-bridge/widgets/editor.ts`, replace the entire file content. Remove the `editors` Map, the contentEditable div creation, the placeholder handling, and the classList.add calls. Import `mountEditor`, `updateEditor`, `resizeEditor`, `disposeEditor` from `../src/editor`. The widget becomes: `mount(elementId, config, dx)` calls `mountEditor(elementId, config as { content?: string; language?: string; filePath?: string }, dx)`. `update(elementId, data)` calls `updateEditor(elementId, (data as { content?: string }).content ?? "")`. `resize(elementId)` calls `resizeEditor(elementId)` (exported from `src/editor.ts`, looks up the EditorView from the instance map and calls `requestMeasure()`). `dispose(elementId)` calls `disposeEditor(elementId)`. Register with `registerWidget("editor", editorWidget)`.

**ActiveForm:** Replacing contentEditable editor with CodeMirror widget

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
