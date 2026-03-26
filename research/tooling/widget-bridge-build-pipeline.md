# Widget Bridge Build Pipeline Analysis

Investigation date: 2026-03-26

## Architecture Overview

The lx desktop app loads a compiled JavaScript bundle (`widget-bridge.js`) that provides
all the TypeScript widget implementations (terminal, voice, editor, etc.). The bundle is
built from a multi-package pnpm workspace in the `dioxus-common` repo, then copied into
the lx repo's assets directory by a Cargo build script.

## The Build Chain Step by Step

### Step 1: Cargo invokes build.rs

When `cargo check` or `cargo build` runs for `lx-desktop`, Cargo executes
`crates/lx-desktop/build.rs`.

### Step 2: build.rs registers change-detection paths

```rust
let widget_bridge_dir = dioxus_common.join("ts/widget-bridge");
for dir in &["src", "widgets"] {
    println!("cargo:rerun-if-changed={}", widget_bridge_dir.join(dir).display());
}
```

This emits:
- `cargo:rerun-if-changed=<path>/dioxus-common/ts/widget-bridge/src`
- `cargo:rerun-if-changed=<path>/dioxus-common/ts/widget-bridge/widgets`

And also:
- `cargo:rerun-if-changed=build.rs`

### Step 3: build.rs runs `pnpm build`

```rust
Command::new("pnpm").arg("build").current_dir(&widget_bridge_dir).status();
```

This invokes `vite build` (per widget-bridge's package.json scripts), which:
- Reads `vite.config.ts` (entry: `src/index.ts`, IIFE format, output: `dist/widget-bridge.js`)
- Resolves all imports including workspace dependencies
- Bundles everything into a single IIFE file

### Step 4: build.rs copies the dist output into lx assets

```rust
(widget_bridge_dir.join("dist/widget-bridge.js"), assets.join("widget-bridge.js")),
```

The compiled `dist/widget-bridge.js` is copied to `crates/lx-desktop/assets/widget-bridge.js`.

## Dependency Graph

```
widget-bridge/src/index.ts
  -> imports widgets/voice.ts
       -> imports @dioxus-common/audio-playback  (workspace:*)
       -> imports @dioxus-common/audio-capture   (workspace:*)
  -> imports widgets/browser.ts
  -> imports widgets/agent.ts
  -> imports widgets/editor.ts
  -> imports widgets/log-viewer.ts
  -> imports widgets/markdown.ts
  -> imports widgets/json-viewer.ts
  -> imports widgets/terminal.ts
```

The `voice.ts` widget imports directly from the audio workspace packages:
```typescript
import { AudioCapture } from "@dioxus-common/audio-capture";
import { AudioPlayback } from "@dioxus-common/audio-playback";
```

These resolve via:
1. pnpm workspace protocol (`workspace:*` in package.json dependencies)
2. tsconfig.json paths mapping:
   ```json
   "@dioxus-common/audio-capture": ["../audio-capture/src/index.ts"],
   "@dioxus-common/audio-playback": ["../audio-playback/src/index.ts"]
   ```

Vite follows these into the sibling packages and **inlines** their source code into the
final bundle. The compiled `widget-bridge.js` contains sections like:
```
//#region ../audio-playback/src/playback.ts
//#region ../audio-capture/src/capture.ts
```

## Answers to the Four Questions

### 1. Does `cargo check` / `cargo build` of lx-desktop trigger a rebuild of the TypeScript bundle?

**YES, but only when Cargo decides to re-run build.rs.** When build.rs runs, it
unconditionally executes `pnpm build` (there is no staleness check on the TS side --
Vite always rebuilds). The question is whether Cargo decides to re-run build.rs at all.

### 2. Does the build script detect changes in audio-playback source files?

**NO.** This is the critical bug. The `rerun-if-changed` directives only watch:
- `dioxus-common/ts/widget-bridge/src` (directory)
- `dioxus-common/ts/widget-bridge/widgets` (directory)
- `build.rs` itself

It does NOT watch:
- `dioxus-common/ts/audio-playback/src/` (or any files within)
- `dioxus-common/ts/audio-capture/src/` (or any files within)

Furthermore, Cargo's `rerun-if-changed` on a **directory path** only triggers a rebuild
when the directory listing itself changes (files added or removed), NOT when the contents
of existing files inside that directory change. So even the `widget-bridge/src` and
`widget-bridge/widgets` watches are weaker than they appear -- editing an existing
`.ts` file in those directories does not trigger a rebuild unless a file is added or
removed.

### 3. Is the compiled widget-bridge.js in the assets directory a STALE copy, or does it get refreshed on each build?

**It is POTENTIALLY STALE.** It only gets refreshed when build.rs runs, and build.rs only
runs when Cargo's change detection triggers it. Since the change detection has the gaps
described in #2, the assets copy can be stale with respect to:
- Any edit to audio-playback source files (never detected)
- Any edit to audio-capture source files (never detected)
- Any edit to existing files in widget-bridge/src or widget-bridge/widgets (only detected
  if a file was added/removed from the directory, not content changes)

At the time of this investigation, both files are identical (verified with `diff`), both
1,676,652 bytes, with matching modification times from the same build run on 2026-03-26.

### 4. When we edit `dioxus-common/ts/audio-playback/src/playback.ts`, does that change propagate into the widget-bridge.js bundle?

**NO, not automatically.** The change will NOT propagate because:

1. Cargo does not know to re-run build.rs (no `rerun-if-changed` covers audio-playback)
2. Even if build.rs ran, `pnpm build` in widget-bridge WOULD correctly pick up the change
   (Vite resolves workspace deps and re-bundles from source)
3. But since build.rs never runs, the stale `dist/widget-bridge.js` persists, and the
   stale copy in `assets/widget-bridge.js` is what the desktop app loads

**Workarounds that currently force a rebuild:**
- `cargo clean -p lx-desktop` then rebuild
- `touch crates/lx-desktop/build.rs` (forces Cargo to re-run it)
- Add/remove a file in `widget-bridge/src/` or `widget-bridge/widgets/` (triggers the
  directory-level change detection)

## Recommended Fix

The build.rs should be modified to:

1. Watch individual source files (not directories) using `rerun-if-changed` on each `.ts`
   file, or use a glob-walk to emit `rerun-if-changed` for every `.ts` file found.

2. Include the workspace dependency source directories:
   - `dioxus-common/ts/audio-playback/src/`
   - `dioxus-common/ts/audio-capture/src/`

Example fix:
```rust
let watch_dirs = [
    widget_bridge_dir.join("src"),
    widget_bridge_dir.join("widgets"),
    dioxus_common.join("ts/audio-playback/src"),
    dioxus_common.join("ts/audio-capture/src"),
];
for dir in &watch_dirs {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }
}
```

This watches every individual file, so content changes (not just directory listing
changes) trigger a rebuild. And it covers the transitive workspace dependencies that
Vite bundles into the final output.
