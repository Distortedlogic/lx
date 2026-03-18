# lx VS Code Extension — Troubleshooting

## What the extension does

On save of any `.lx` file, runs `lx diagram <file>.lx -o <file>.mmd` to generate a Mermaid diagram alongside the source. Also provides syntax highlighting via TextMate grammar.

## Architecture

- `src/extension.ts` → Vite 8 (SSR mode, CJS output) → `dist/extension.js`
- `package.json` declares language `lx`, grammar `source.lx`
- Extension host calls `exports.activate(ctx)` which registers `onDidSaveTextDocument` listener
- Listener shells out to `lx` binary via `child_process.execFile`

## How to install the extension

**Use the VS Code UI, NOT the CLI.**

`code --install-extension` does NOT work for unsigned local vsix extensions. The CLI install path adds the extension to `extensions.json` with `"source": "vsix"` which triggers marketplace signature verification. Since this extension isn't published on the marketplace, VS Code silently blocks activation.

### Working install method

1. Build: `cd editors/vscode && pnpm package`
2. In VS Code: `Ctrl+Shift+P` → `Extensions: Install from VSIX...`
3. Navigate to `editors/vscode/lx-lang-0.2.0.vsix` and click Install
4. Reload VS Code when prompted

This uses VS Code's internal install path which handles unsigned local extensions correctly.

### Install methods that do NOT work

- `code --install-extension lx-lang-0.2.0.vsix` — installs files but extension never activates
- `code --install-extension ... --force` — same result

### Why `code --install-extension` fails

- The CLI adds an entry to `~/.vscode/extensions/extensions.json` with `"source": "vsix"` and `"pinned": true`
- This code path triggers marketplace verification: VS Code tries to fetch `marketplace.visualstudio.com/_apis/public/gallery/vscode/lx-lang/lx-lang/latest` and gets a 404
- The extension is installed (files present, shows in `code --list-extensions`) but never sent to the extension host for activation
- `exthost.log` has zero mention of the extension
- `"extensions.verifySignature": false` does NOT fix this
- This affects ALL unsigned vsix extensions installed via CLI, not just lx-lang (confirmed with a minimal test extension)

### Other working alternatives

- **Dev mode:** `code --extensionDevelopmentPath=/home/entropybender/repos/lx/editors/vscode /home/entropybender/repos/lx`
- **Symlink:** `ln -s /path/to/editors/vscode ~/.vscode/extensions/lx-lang.lx-lang-0.2.0`

## Other issues

### Extension activates but no .mmd generated

Check Output panel → "lx" channel. Common causes: `lx` not on PATH, stale binary, file languageId not `"lx"`.

### Syntax highlighting broken

Check status bar (bottom right) shows "lx" language mode. If "Plain Text", click → select "lx".

## Build from source

```bash
cd editors/vscode
pnpm install
pnpm package   # builds + creates .vsix
```

## Key files

| File | Purpose |
|------|---------|
| `src/extension.ts` | On-save hook, manual command, output channel logging |
| `vite.config.ts` | SSR/Node build, CJS output, vscode externalized |
| `package.json` | Extension manifest, settings, commands |
| `dist/extension.js` | Built bundle |
| `syntaxes/lx.tmLanguage.json` | TextMate grammar for syntax highlighting |

## VS Code settings

| Setting | Default | Description |
|---------|---------|-------------|
| `lx.diagram.autoGenerate` | `true` | Auto-generate .mmd on save |
| `lx.diagram.binaryPath` | `""` | Path to lx binary (falls back to `lx` on PATH) |
