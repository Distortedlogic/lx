# Goal

Add nine developer experience features to the lx VS Code extension — document outline/symbols, run-file command, hover information for builtins, inline diagnostics via `lx check`, CodeLens run buttons, bracket pair colorization, status bar integration, task provider for justfile recipes, and semantic token highlighting. The extension currently only provides syntax highlighting (TextMate grammar) and auto-diagram generation on save. These features bring it to parity with what developers expect from a modern language extension.

# Why

- No way to run an lx file from the editor without manually typing `lx run <path>` in the terminal
- No inline error feedback — errors only visible after switching to the terminal and running `lx check`
- No outline view or breadcrumbs — no quick navigation to functions, protocols, or type definitions in larger files
- No hover information for the 60+ built-in functions — developers must look up signatures externally
- No visual indication of file health in the editor chrome (status bar)
- No CodeLens affordance to run a file — common in VS Code language extensions
- No task integration — running `just test`, `just diagnose`, etc. requires manually switching to the terminal
- Bracket pair colorization works natively in VS Code but benefits from explicit `colorizedBracketPairs` hints in the language configuration
- Semantic tokens provide richer highlighting context than TextMate grammar alone (e.g., distinguishing local variables from function parameters)

# What changes

**language-configuration.json — bracket pair colorization:** Add `colorizedBracketPairs` array listing the three bracket pairs `()`, `[]`, `{}` so VS Code's native bracket pair colorizer applies distinct colors per nesting level.

**package.json — contribution points for all new features:** Register `lx.runFile` command with editor title run button and keybinding. Add configuration properties for `lx.diagnostics.onSave` (boolean, default true), `lx.diagnostics.onType` (boolean, default false), and `lx.binaryPath` (string, replaces the diagram-specific binary path to share across all features). Register the semantic token provider with token types and modifiers. Register the task provider for `lx` task type.

**src/diagram.ts — extract existing diagram code:** Move `lxBinary`, `mmdPath`, and `generateDiagram` from extension.ts into their own module. Export them. The `lxBinary` function changes to read from the new shared `lx.binaryPath` config key.

**src/run.ts — run file command:** Export an `activate` function that registers the `lx.runFile` command. On invocation, get the active editor's file path, create or reuse a VS Code terminal named "lx", and send `lx run <filepath>` to it. If no `.lx` file is open, show a warning.

**src/statusbar.ts — status bar integration:** Export an `activate` function that creates a `StatusBarItem` aligned left. The item shows "lx" with a check icon when the last `lx check` succeeded, or "lx: N errors" with a warning icon when errors exist. The item updates whenever diagnostics change. Clicking it runs `lx.runFile`.

**src/symbols.ts — document symbol provider:** Export a `DocumentSymbolProvider` that parses the document text with regex to find top-level let bindings (as Function symbols), Protocol declarations (as Interface symbols), tagged union type definitions (as Enum symbols), use imports (as Module symbols), and exported bindings prefixed with `+` (as Function symbols with export detail). Each symbol includes its full range (from declaration to the end of its body block or the next top-level declaration). The provider registers for the `lx` language selector.

**src/builtins.ts — built-in function catalog:** Export a lookup map of every stdlib module and function with name, module path, arity, and a one-line description. This data is consumed by the hover provider. The catalog covers all modules from `std/json`, `std/ctx`, `std/math`, `std/fs`, `std/env`, `std/re`, `std/time`, `std/git`, `std/http`, `std/md`, `std/ai`, `std/agent`, `std/mcp`, `std/user`, `std/test`, `std/trace`, `std/describe`, `std/diag`, and the collection builtins (`map`, `filter`, `fold`, `flat_map`, `each`, `take`, `drop`, `zip`, `enumerate`, `find`, `sort_by`, `partition`, `group_by`, `chunks`, `windows`, `scan`, `tap`, `pmap`, `pmap_n`, etc.).

**src/hover.ts — hover provider:** Export a `HoverProvider` that checks the word under the cursor against the builtins catalog. On match, render a Markdown hover showing the module path, function signature with arity, and description. Also handle keywords (`par`, `sel`, `loop`, `break`, `yield`, `assert`, `use`) and operators (`~>`, `~>?`, `??`, `^`, `|`) with brief explanations.

**src/diagnostics.ts — inline diagnostics from lx check:** Export an `activate` function that creates a `DiagnosticCollection`. On file save (and optionally on text change with debounce), spawn `lx check <filepath>` as a child process. Parse the miette-formatted stderr output by extracting error locations from the `╭─[filepath:line:col]` pattern and error messages from lines starting with `×`. Convert each to a VS Code `Diagnostic` at the parsed range with the extracted message. Clear and repopulate the collection per file. On process error (binary not found), log to the output channel without spamming the user.

**src/codelens.ts — CodeLens run buttons:** Export a `CodeLensProvider` that scans for lines matching `+main` or top-level exported bindings. Place a "Run" CodeLens above each match that triggers the `lx.runFile` command. Refresh on document change.

**src/tasks.ts — task provider for lx workflows:** Export a `TaskProvider` for the `lx` task type. Provide predefined tasks: "lx: run" (runs current file), "lx: test" (runs `lx test`), "lx: check" (runs `lx check`), and "lx: diagram" (runs `lx diagram` on current file). Each task uses a `ShellExecution` with the appropriate command. If a `justfile` is found at the workspace root, also provide "lx: just test", "lx: just diagnose", and "lx: just fmt" tasks.

**src/semantic.ts — semantic token provider:** Export a `DocumentSemanticTokensProvider` with a `SemanticTokensLegend` defining token types (keyword, function, variable, parameter, type, property, operator, string, number, comment, namespace, enumMember) and modifiers (declaration, definition, readonly, defaultLibrary). Parse the document with regex to identify tokens and assign semantic types. This provides enhanced highlighting beyond the TextMate grammar — for example, distinguishing `Protocol` names as types vs. keywords, or identifying function call targets.

**src/extension.ts — wiring hub:** Slim down to import and call `activate` on each feature module. The `onDidSaveTextDocument` listener delegates to both diagram generation and diagnostics. All subscriptions flow through the extension context for proper disposal.

# How it works

The extension follows a modular architecture where each DX feature lives in its own file exporting an `activate` function (or a provider class). The main `extension.ts` calls each module's activate in sequence, passing the extension context and shared resources (output channel, binary path helper).

Diagnostics parse miette's fancy terminal output. The key extraction regex targets the `╭─[file:line:col]` source location marker and the `×` error description line. Since `lx check` can operate on a single file, each save triggers a per-file check rather than a workspace-wide scan.

The builtins catalog is a static data structure compiled from the stdlib module registry in `crates/lx/src/stdlib/mod.rs`. It does not need to stay perfectly in sync — the catalog provides discoverability, not correctness guarantees. When stdlib functions change, the catalog should be updated manually.

The document symbol provider uses line-by-line regex scanning rather than a full parser. It looks for patterns: `name =` at the start of a line (binding), `+name =` (exported binding), `Protocol Name` (protocol), `Name =` followed by `|` on the next line (tagged union), and `use path` (import). This is imperfect but sufficient for outline navigation.

Semantic tokens layer on top of the TextMate grammar. VS Code merges both — semantic tokens take precedence where they exist, and the grammar fills in everywhere else. The regex-based semantic token provider is a stepping stone toward a future LSP-backed provider.

# Files affected

**Modified files:**
- `editors/vscode/package.json` — new commands, settings, task definition, keybinding, menus
- `editors/vscode/language-configuration.json` — add `colorizedBracketPairs`
- `editors/vscode/src/extension.ts` — refactor to hub that imports and activates feature modules
- `editors/vscode/vite.config.ts` — may need adjustment if new source files aren't picked up by the SSR entry point

**New files:**
- `editors/vscode/src/diagram.ts` — extracted diagram generation logic
- `editors/vscode/src/run.ts` — run file command
- `editors/vscode/src/statusbar.ts` — status bar item
- `editors/vscode/src/symbols.ts` — document symbol provider
- `editors/vscode/src/builtins.ts` — builtin function catalog data
- `editors/vscode/src/hover.ts` — hover provider
- `editors/vscode/src/diagnostics.ts` — inline diagnostics via lx check
- `editors/vscode/src/codelens.ts` — CodeLens run buttons
- `editors/vscode/src/tasks.ts` — task provider
- `editors/vscode/src/semantic.ts` — semantic token provider

# Task List

## Task 1: Update package.json with all new contribution points

**Subject:** Register all new commands, settings, tasks, keybindings, and menus in package.json
**ActiveForm:** Updating package.json contribution points

**Description:** Edit `editors/vscode/package.json`. Promote `lx.diagram.binaryPath` to a shared `lx.binaryPath` at the top of the configuration properties (keep `lx.diagram.autoGenerate` and `lx.diagram.binaryPath` as-is for backward compatibility, but the new features should read from `lx.binaryPath`). Add configuration properties: `lx.diagnostics.onSave` (boolean, default true, description "Run lx check on save"), `lx.diagnostics.onType` (boolean, default false, description "Run lx check on text change with debounce"). Add commands: `lx.runFile` with title "lx: Run File". Add keybindings: `ctrl+shift+r` (mac: `cmd+shift+r`) for `lx.runFile` when `editorLangId == lx`. Add `menus` entry: `editor/title/run` with `lx.runFile` when `resourceLangId == lx`. Add `taskDefinitions` with type `lx` and no required properties. Add `semanticTokenTypes` and `semanticTokenModifiers` if needed, or rely on the built-in legend (VS Code provides standard types). Bump version to `0.3.0`.

---

## Task 2: Add colorizedBracketPairs to language configuration

**Subject:** Enable bracket pair colorization hints in language-configuration.json
**ActiveForm:** Adding bracket pair colorization hints

**Description:** Edit `editors/vscode/language-configuration.json`. Add a `colorizedBracketPairs` array at the top level containing three entries: `["(", ")"]`, `["[", "]"]`, `["{", "}"]`. This tells VS Code's native bracket pair colorizer which pairs to colorize. No other changes needed — VS Code handles the rendering.

---

## Task 3: Extract diagram logic to diagram.ts

**Subject:** Move diagram generation code from extension.ts to diagram.ts
**ActiveForm:** Extracting diagram logic to separate module

**Description:** Create `editors/vscode/src/diagram.ts`. Move the `lxBinary`, `mmdPath`, and `generateDiagram` functions from `extension.ts` into this new file. Also move the `log` output channel reference — export a function `setLog(channel: vscode.OutputChannel)` or accept the channel as a parameter to `generateDiagram`. Export all three functions plus a `activate(ctx: vscode.ExtensionContext, log: vscode.OutputChannel)` function that registers the `onDidSaveTextDocument` listener for auto-diagram and the `lx.generateDiagram` command. Update `extension.ts` to import and call `diagram.activate(ctx, log)` instead of inlining the logic. Create a shared `lxBinary()` helper that reads `lx.binaryPath` first, falls back to `lx.diagram.binaryPath`, falls back to `"lx"`. Export this from diagram.ts (other modules will import it). Verify the extension builds: `cd editors/vscode && pnpm build`.

---

## Task 4: Implement run file command

**Subject:** Add lx.runFile command with terminal integration
**ActiveForm:** Implementing run file command

**Description:** Create `editors/vscode/src/run.ts`. Export an `activate(ctx: vscode.ExtensionContext)` function. Inside, register the `lx.runFile` command. The handler: get the active text editor, if none or not lx language, show warning "Open a .lx file first" and return. Otherwise, get the file path from `editor.document.uri.fsPath`. Find or create a terminal named "lx" using `vscode.window.terminals.find(t => t.name === "lx")` or `vscode.window.createTerminal("lx")`. Show the terminal and send the text `lx run "<filepath>"` (quoted to handle spaces) via `terminal.sendText`. Import the shared `lxBinary` function from diagram.ts and use it in the command string instead of hardcoded `"lx"`. Wire into `extension.ts` by importing and calling `run.activate(ctx)`.

---

## Task 5: Implement status bar item

**Subject:** Add status bar item showing lx parse status
**ActiveForm:** Implementing status bar integration

**Description:** Create `editors/vscode/src/statusbar.ts`. Export an `activate(ctx: vscode.ExtensionContext)` function. Create a `StatusBarItem` using `vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100)`. Set its command to `lx.runFile` so clicking it runs the file. Set initial text to `"$(check) lx"` (VS Code codicon syntax). Push it to context subscriptions. Export an `update(errorCount: number)` function: if errorCount is 0, set text to `"$(check) lx"` and tooltip to "lx: no errors"; otherwise set text to `"$(warning) lx: ${errorCount}"` and tooltip to `"lx: ${errorCount} error(s)"`. Show the item when the active editor is an lx file, hide otherwise — register `onDidChangeActiveTextEditor` to toggle visibility. Wire into `extension.ts`.

---

## Task 6: Implement document symbol provider

**Subject:** Add document symbol provider for outline and breadcrumbs
**ActiveForm:** Implementing document symbol provider

**Description:** Create `editors/vscode/src/symbols.ts`. Export a class `LxDocumentSymbolProvider` implementing `vscode.DocumentSymbolProvider`. In `provideDocumentSymbols`, iterate over document lines and match these patterns: (1) lines matching `^(\+?\w+)\s*=` — top-level bindings, using SymbolKind.Function (or Variable if followed by a simple literal rather than params/block); (2) lines matching `^Protocol\s+(\w+)` — protocol declarations, SymbolKind.Interface; (3) lines matching `^(\w+)\s*=\s*$` followed by a line starting with `\s*\|` — tagged union type definitions, SymbolKind.Enum; (4) lines matching `^use\s+(.+)` — imports, SymbolKind.Module. For each match, create a `DocumentSymbol` with the name, kind, and range spanning from the match line to the end of the block (find the next top-level declaration or end of file). Export an `activate(ctx)` function that registers the provider for the `lx` language selector. Wire into `extension.ts`.

---

## Task 7: Implement builtins catalog

**Subject:** Create static builtin function catalog for hover and completions
**ActiveForm:** Building builtin function catalog

**Description:** Create `editors/vscode/src/builtins.ts`. Define and export an interface `BuiltinEntry` with fields: `module` (string), `name` (string), `arity` (number), `description` (string). Export a `Map<string, BuiltinEntry[]>` named `BUILTINS` keyed by module name (e.g., `"json"`, `"fs"`, `"math"`). Also export a flat `Map<string, BuiltinEntry>` named `BUILTIN_FUNCTIONS` keyed by qualified name (e.g., `"json.parse"`, `"fs.read"`). Populate with entries from the stdlib registry. Cover at minimum these modules with all their functions: `json` (parse, encode, encode_pretty), `ctx` (empty, load, save, get, set, remove, keys, merge), `math` (abs, ceil, floor, round, pow, sqrt, min, max, plus constants pi, e, inf), `fs` (read, write, append, exists, remove, mkdir, ls, stat), `env` (get, vars, args, cwd, home), `re` (match, find_all, replace, replace_all, split, is_match), `time` (now, sleep, format, parse), `http` (get, post, put, delete, patch), `test` (assert, assert_eq, assert_ne, assert_err). Also include the global collection builtins as entries with module `"global"`: map, filter, fold, flat_map, each, take, drop, zip, enumerate, find, sort_by, partition, group_by, chunks, windows, scan, tap, pmap, pmap_n, len, head, tail, last, reverse, contains?, empty?, keys, values, join, split, trim, upper, lower, starts?, ends?, replace, type_of, to_str, to_int, to_float, print, println, debug, assert.

---

## Task 8: Implement hover provider

**Subject:** Add hover provider for built-in functions and keywords
**ActiveForm:** Implementing hover provider

**Description:** Create `editors/vscode/src/hover.ts`. Export a class `LxHoverProvider` implementing `vscode.HoverProvider`. In `provideHover`, get the word at the position using `document.getWordRangeAtPosition(position)` and `document.getText(range)`. Also check for a dotted prefix by looking at the character before the word range — if it's a dot, extend backward to get the module name (e.g., `json.parse` → module `json`, name `parse`). Look up the word in the `BUILTIN_FUNCTIONS` map (qualified) or `BUILTINS` map (by module). If found, return a `Hover` with markdown content showing: module path as a code header, function name with arity as a signature line, and the description. Also handle keywords — build a small static map of lx keywords (`par`, `sel`, `loop`, `break`, `yield`, `assert`, `use`, `Protocol`) and operators (`~>`, `~>?`, `??`, `^`, `|`) to brief descriptions. Register for the `lx` selector. Export `activate(ctx)` to register the provider. Wire into `extension.ts`.

---

## Task 9: Implement inline diagnostics

**Subject:** Add diagnostic provider running lx check on save
**ActiveForm:** Implementing inline diagnostics via lx check

**Description:** Create `editors/vscode/src/diagnostics.ts`. Export an `activate(ctx, log, onErrorCount)` function where `onErrorCount` is a callback taking a number (for status bar updates). Create a `DiagnosticCollection` via `vscode.languages.createDiagnosticCollection("lx")`. Register an `onDidSaveTextDocument` listener: if the saved document is lx and `lx.diagnostics.onSave` is enabled, call an internal `runCheck(doc)` function. `runCheck` spawns `lx check <filepath>` via `execFile` from `child_process`, capturing stderr. Parse the stderr output to extract diagnostics: scan for lines matching `╭─\[(.+?):(\d+):(\d+)\]` to get file, line, col. Scan for lines matching `×\s*(.+)` to get error messages. Pair each location with its message. Convert to `vscode.Diagnostic` objects: the range starts at the parsed line/col (convert to 0-indexed) and spans the remainder of the line. Set severity to `Error`. Clear the collection for this file's URI, then set the new diagnostics. Call `onErrorCount(diagnostics.length)`. If `execFile` returns an error with code ENOENT (binary not found), log once to the output channel and do not re-attempt until config changes. Also register `onDidCloseTextDocument` to clear diagnostics for closed files.

---

## Task 10: Implement CodeLens provider

**Subject:** Add CodeLens run buttons above entry points
**ActiveForm:** Implementing CodeLens provider

**Description:** Create `editors/vscode/src/codelens.ts`. Export a class `LxCodeLensProvider` implementing `vscode.CodeLensProvider`. In `provideCodeLenses`, scan document lines for patterns indicating runnable entry points: lines matching `^\+main\s*=` or `^\+\w+\s*=` (any exported binding). For each match, create a `CodeLens` at that line's range with command `lx.runFile` and title `"$(play) Run"`. Return the array. Export `activate(ctx)` to register the provider for the `lx` selector and push to subscriptions. Wire into `extension.ts`.

---

## Task 11: Implement task provider

**Subject:** Add task provider for lx and justfile workflows
**ActiveForm:** Implementing task provider

**Description:** Create `editors/vscode/src/tasks.ts`. Export a class `LxTaskProvider` implementing `vscode.TaskProvider`. In `provideTasks`, build a list of `vscode.Task` objects. Import `lxBinary` from diagram.ts. Create tasks: (1) "lx: run file" — ShellExecution of `${lxBinary()} run ${activeFile}` with problemMatcher; (2) "lx: check workspace" — ShellExecution of `${lxBinary()} check`; (3) "lx: test" — ShellExecution of `${lxBinary()} test`; (4) "lx: list" — ShellExecution of `${lxBinary()} list`. Then check if a `justfile` exists at the workspace root using `vscode.workspace.findFiles("justfile", null, 1)`. If found, add tasks: "lx: just test" (`just test`), "lx: just diagnose" (`just diagnose`), "lx: just fmt" (`just fmt`), "lx: just build" (`just build`). Each task uses group `TaskGroup.Build` or `TaskGroup.Test` as appropriate. `resolveTask` returns undefined (VS Code handles it). Export `activate(ctx)` to register the provider with task type `"lx"`. Wire into `extension.ts`.

---

## Task 12: Implement semantic token provider

**Subject:** Add semantic token provider for enhanced highlighting
**ActiveForm:** Implementing semantic token provider

**Description:** Create `editors/vscode/src/semantic.ts`. Define a `SemanticTokensLegend` with token types: `comment`, `string`, `keyword`, `number`, `operator`, `function`, `variable`, `parameter`, `type`, `namespace`, `enumMember`, `property`. Modifiers: `declaration`, `definition`, `readonly`, `defaultLibrary`. Export a class `LxSemanticTokensProvider` implementing `vscode.DocumentSemanticTokensProvider`. In `provideDocumentSemanticTokens`, create a `SemanticTokensBuilder` with the legend. Iterate over document lines. For each line: skip comment lines (starting with `--`) by pushing the entire line as `comment` type. Scan for string literals (double-quoted, backtick-quoted) and push as `string`. Scan for keywords (`par`, `sel`, `loop`, `break`, `yield`, `assert`, `use`, `Protocol`) and push as `keyword`. Scan for number literals and push as `number`. Scan for identifiers that are known builtins (from builtins.ts) and push as `function` with `defaultLibrary` modifier. Scan for identifiers followed by `=` at line start and push as `variable` with `declaration` modifier. Scan for capitalized identifiers and push as `type`. Return `builder.build()`. Export `activate(ctx)` to register with `registerDocumentSemanticTokensProvider` for `lx` selector with the legend and full-document trigger. Wire into `extension.ts`.

---

## Task 13: Refactor extension.ts as activation hub

**Subject:** Rewire extension.ts to import and activate all feature modules
**ActiveForm:** Refactoring extension.ts as module hub

**Description:** Rewrite `editors/vscode/src/extension.ts`. Import activate functions from all modules: `diagram`, `run`, `statusbar`, `symbols`, `hover`, `diagnostics`, `codelens`, `tasks`, `semantic`. In the `activate` function: create the output channel. Call each module's activate in sequence, passing `ctx`, `log`, and any cross-module callbacks (e.g., pass `statusbar.update` as the `onErrorCount` callback to `diagnostics.activate`). Register the document symbol provider, hover provider, CodeLens provider, and semantic tokens provider by importing their classes and calling `vscode.languages.registerDocumentSymbolProvider`, etc. The deactivate function remains empty (VS Code disposes subscriptions via context). Verify the extension builds: `cd editors/vscode && pnpm build`.

---

## Task 14: Format

**Subject:** Format
**ActiveForm:** Formatting

**Description:** Run the following command verbatim, exactly as written, with no modifications: `just fmt`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way.

---

## Task 15: Build and verify extension

**Subject:** Build the extension and verify it compiles
**ActiveForm:** Building and verifying extension

**Description:** Run `cd editors/vscode && pnpm build`. Fix any TypeScript compilation errors. Re-run until the build succeeds with no errors. Then run `cd editors/vscode && pnpm package` to verify the VSIX packages correctly. If the package command fails due to version mismatch with the existing .vsix file, delete the old `lx-lang-0.2.0.vsix` and re-run.

---

## Task 16: Final commit

**Subject:** Commit all extension DX features
**ActiveForm:** Committing extension changes

**Description:** Run the following command verbatim, exactly as written, with no modifications: `git add -A && git commit -m "feat: add DX features to vscode extension — run, hover, diagnostics, outline, codelens, tasks, semantic tokens, status bar"`. Do NOT pipe, redirect, append shell operators (`| tail`, `| head`, `2>&1`, `> /dev/null`, `| grep`, etc.), or otherwise modify this command in any way. CRITICAL: Do NOT use `$()`, heredocs, `cat <<EOF`, or any subshell substitution — just a plain `-m "message"` flag. Do NOT append `--trailer`, `Co-authored-by`, `Signed-off-by`, or any other trailer/metadata.

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

These are rules you (the executor) are known to violate. Re-read before starting each task:

1. **This is TypeScript extension code, not Rust.** The source lives in `editors/vscode/src/`. Build with `cd editors/vscode && pnpm build`. Do not run `just diagnose` or `just test` for TypeScript changes (those are Rust-only).
2. **No code comments or doc strings** per CLAUDE.md. Do not add JSDoc, inline comments, or trailing explanations to TypeScript files.
3. **300 line file limit** per CLAUDE.md. Each new `.ts` file must stay under 300 lines.
4. **Run commands VERBATIM.** Copy-paste the exact command from the task description. Do not modify, "improve," or substitute commands. Do NOT append `--trailer`, `Co-authored-by`, or any metadata to commit commands.
5. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
6. **Do NOT modify verbatim commands.** No piping (`| tail`, `| head`, `| grep`), no redirects (`2>&1`, `> /dev/null`), no subshells. Run the exact command string as written — nothing appended, nothing prepended.

---

## Task Loading Instructions

To execute this work item, read each `## Task N:` entry from the Task List section above. For each task, call `TaskCreate` with:
- `subject`: The task heading text — copied VERBATIM, not paraphrased
- `description`: The full body text under that heading — copied VERBATIM, not paraphrased, summarized, or reworded
- `activeForm`: The ActiveForm value from the task

After creating all tasks, use `TaskUpdate` to set `addBlockedBy` on each task N (N > 1) pointing to task N-1, enforcing strict sequential execution.

Execution rules:
- Execute tasks strictly in order — mark each `in_progress` before starting and `completed` when done
- Run commands EXACTLY as written in the task description — do not substitute `cargo` for `just` or vice versa
- Do not run any command not specified in the current task
- Do not "pre-check" compilation between implementation tasks
- If a task says "Run the following command verbatim" then copy-paste that exact command — do not modify it
- Do NOT paraphrase, summarize, reword, combine, split, reorder, skip, or add tasks beyond what is in the Task List section
- Do NOT append shell operators to commands
