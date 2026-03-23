# Goal

Replace the contentEditable div editor widget with CodeMirror 6, providing a proper code editor with syntax highlighting, line numbers, bracket matching, search/replace, and a dark theme matching lx's palette. CodeMirror is installed as an npm dependency in the widget-bridge TS package and bundled via vite into the existing IIFE output. No Rust changes needed — the widget protocol (content in config, save via dx.send) stays the same.

# Why

- The current editor is a contentEditable div with no syntax highlighting, no line numbers, no code editing features — it's a text area, not an editor
- Users expect IDE-level editing when they open an "Editor" pane
- CodeMirror 6 is the correct choice over Monaco: it's modular (~200-300KB vs Monaco's 5-10MB), bundles cleanly as IIFE (proven by xterm.js already bundling as ESM → IIFE in the same vite config), and has proper tree-shaking
- The integration pattern is already proven: xterm.js in `ts/widget-bridge/src/terminal.ts` demonstrates instance management, CSS injection via `?inline`, and widget lifecycle — CodeMirror follows the same pattern exactly

# What changes

**Install CodeMirror packages:** Add `@codemirror/view`, `@codemirror/state`, `@codemirror/basic-setup`, `@codemirror/theme-one-dark`, and language packs (`lang-javascript`, `lang-python`, `lang-json`, `lang-html`, `lang-css`, `lang-rust`) to `ts/widget-bridge/package.json`.

**New implementation file `ts/widget-bridge/src/editor.ts`:** Following `src/terminal.ts` pattern — instance Map keyed by element ID, `mountEditor(elementId, config, dx)` creates an `EditorView` with extensions (basicSetup, language detection from file extension, oneDark theme), `updateEditor(elementId, content)` replaces document content, `disposeEditor(elementId)` destroys the view. Language detection maps file extension from `config.filePath` to the appropriate language pack.

**CSS injection `ts/widget-bridge/src/inject-codemirror-css.ts`:** Following `inject-css.ts` pattern — imports CodeMirror CSS via `?inline`, injects into `<head>` once. Called from `mountEditor`.

**Rewrite `ts/widget-bridge/widgets/editor.ts`:** Replace contentEditable implementation with calls to `mountEditor`/`updateEditor`/`disposeEditor`. Ctrl+S handler uses `EditorView.state.doc.toString()` to get content and sends via `dx.send({ type: "save", content })`.

# How it works

1. User creates an Editor pane → `EditorView` (Rust) reads file content from disk, passes `{ content, language, filePath }` to widget
2. Widget `mount` calls `mountEditor` which creates a CodeMirror `EditorView` with:
   - `basicSetup` extension (line numbers, bracket matching, indentation, search, etc.)
   - Language extension detected from `filePath` extension (`.js` → lang-javascript, `.py` → lang-python, `.rs` → lang-rust, `.json` → lang-json, etc.)
   - `oneDark` theme customized to match lx's color palette
3. Content is set from config
4. User edits code with full IDE features
5. Ctrl+S → `dx.send({ type: "save", content })` → Rust can write back to disk

# Files affected

- EDIT: `ts/widget-bridge/package.json` — add @codemirror/* dependencies
- NEW: `ts/widget-bridge/src/editor.ts` — CodeMirror mounting/update/dispose implementation
- NEW: `ts/widget-bridge/src/inject-codemirror-css.ts` — CSS injection for CodeMirror
- EDIT: `ts/widget-bridge/widgets/editor.ts` — replace contentEditable with CodeMirror widget calls

# Task List

### Task 1: Install CodeMirror 6 packages

**Subject:** Add CodeMirror 6 dependencies to widget-bridge

**Description:** Run `pnpm add @codemirror/view @codemirror/state @codemirror/basic-setup @codemirror/theme-one-dark @codemirror/lang-javascript @codemirror/lang-python @codemirror/lang-json @codemirror/lang-html @codemirror/lang-css @codemirror/lang-rust` in the `ts/widget-bridge/` directory.

**ActiveForm:** Installing CodeMirror 6 packages

### Task 2: Create CodeMirror CSS injection

**Subject:** Add CSS injection for CodeMirror following xterm pattern

**Description:** Create `ts/widget-bridge/src/inject-codemirror-css.ts`. Follow the exact pattern in `ts/widget-bridge/src/inject-css.ts`. Import CodeMirror's base CSS using Vite's `?inline` query. CodeMirror 6 injects its own base styles via JavaScript (it doesn't ship a separate CSS file like xterm does), so this file should import the oneDark theme CSS if it has one, or simply export a no-op `ensureCodemirrorCss()` function if CodeMirror handles all styling internally via JS. Check `node_modules/@codemirror/view/` and `node_modules/@codemirror/theme-one-dark/` for any `.css` files after installing. If none exist, CodeMirror handles styling via JS and no CSS injection is needed — in that case, create the file with just an empty `ensureCodemirrorCss` export for consistency.

**ActiveForm:** Creating CodeMirror CSS injection module

### Task 3: Create CodeMirror editor implementation

**Subject:** Implement editor mount/update/dispose following terminal.ts pattern

**Description:** Create `ts/widget-bridge/src/editor.ts`. Import `EditorView` from `@codemirror/view`, `EditorState` from `@codemirror/state`, `basicSetup` from `@codemirror/basic-setup`, `oneDark` from `@codemirror/theme-one-dark`, and all language packs. Create a `Map<string, EditorView>` for instance tracking. Export `mountEditor(elementId: string, config: { content?: string; language?: string; filePath?: string }, dx: Dioxus)`: (1) call `ensureCodemirrorCss()`, (2) detect language from `config.filePath` extension — map `.js/.ts/.jsx/.tsx` to `javascript()`, `.py` to `python()`, `.rs` to `rust()`, `.json` to `json()`, `.html` to `html()`, `.css` to `css()`, default to empty extensions, (3) create `EditorState.create({ doc: config.content ?? "", extensions: [basicSetup, oneDark, languageExtension, EditorView.updateListener.of(update => { if (update.docChanged) { /* could debounce and send updates */ } }), keymap.of([{ key: "Mod-s", run: () => { dx.send({ type: "save", content: view.state.doc.toString() }); return true; } }])] })`, (4) create `new EditorView({ state, parent: document.getElementById(elementId) })`, (5) store in map. Export `updateEditor(elementId, content)`: get view from map, dispatch a transaction replacing the entire doc. Export `disposeEditor(elementId)`: get view from map, call `view.destroy()`, delete from map.

**ActiveForm:** Implementing CodeMirror editor mount/update/dispose

### Task 4: Rewrite editor widget to use CodeMirror

**Subject:** Replace contentEditable editor with CodeMirror implementation

**Description:** In `ts/widget-bridge/widgets/editor.ts`, replace the entire file content. Remove the `editors` Map, the contentEditable div creation, the placeholder handling, and the classList.add calls. Import `mountEditor`, `updateEditor`, `disposeEditor` from `../src/editor`. The widget becomes: `mount(elementId, config, dx)` calls `mountEditor(elementId, config as { content?: string; language?: string; filePath?: string }, dx)`. `update(elementId, data)` calls `updateEditor(elementId, (data as { content?: string }).content ?? "")`. `resize` is a no-op (CodeMirror handles its own resize). `dispose(elementId)` calls `disposeEditor(elementId)`. Register with `registerWidget("editor", editorWidget)`.

**ActiveForm:** Replacing contentEditable editor with CodeMirror widget

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.
