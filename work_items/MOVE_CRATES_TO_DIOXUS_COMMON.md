# Move non-lx crates to dioxus-common

10 Rust crates + 4 TypeScript packages in the lx workspace are generic (no dependency on lx-core: `lx`, `lx-cli`, `lx-macros`). All are consumed exclusively by `lx-desktop`. Move them to `~/repos/dioxus-common`.

## Rust crates

| lx crate | dioxus-common name | Action |
|---|---|---|
| `audio-core` | `common-audio` | Move + rename |
| `inference-client` | `common-inference` | Move + rename |
| `kokoro-client` | `common-kokoro` | Move + rename |
| `whisper-client` | `common-whisper` | Move + rename |
| `voice-agent` | `common-voice` | Move + rename |
| `browser-cdp` | `common-cdp` | Move + rename |
| `pane-tree` | `common-pane-tree` | Move + rename |
| `pty-mux` | `common-pty` | Move + rename |
| `widget-bridge` | `dioxus-widget-bridge` | Move + rename (has Dioxus dep) |
| `dx-charts` | — | Delete entirely; `common-charts` already exists and is more mature |

### Internal dependency chain (preserved in dioxus-common)

```
common-voice
  ├── common-kokoro → common-inference
  ├── common-whisper → common-inference
  └── common-audio
```

All other crates are standalone (no inter-crate deps).

## TypeScript packages

| lx package | dioxus-common target | Action |
|---|---|---|
| `ts/dx-charts` (`@lx/dx-charts`) | — | Delete; `ts/charts` already exists in dioxus-common |
| `ts/widget-bridge` (`@lx/widget-bridge`) | `ts/widget-bridge` | Move + rename package |
| `ts/audio-capture` (`@lx/audio-capture`) | `ts/audio-capture` | Move + rename package |
| `ts/audio-playback` (`@lx/audio-playback`) | `ts/audio-playback` | Move + rename package |

`ts/widget-bridge/package.json` has workspace deps on `@lx/audio-capture` and `@lx/audio-playback` — these must be updated to the new package names. `ts/widget-bridge/tsconfig.json` has path aliases for `@lx/audio-capture` and `@lx/audio-playback` — these must also be updated.

Both repos use pnpm workspaces with `ts/*` glob and vite for builds. The TS packages do NOT use `vite.base.ts` — they have inline vite configs. The tsconfig files extend `../../tsconfig.base.json` (both repos have one; lx targets ES2022, dioxus-common targets ES2020 — the packages will inherit dioxus-common's config after move, which is fine).

## Steps

### 1. Copy FLOW_* colors to common-charts, then delete dx-charts

`common-charts` already has all the chart types dx-charts has plus 5 more. The shared types (`LineData`, `BarSeries`, `ScatterSeries`, `PieSlice`, `DataZoomConfig`, `LegendPosition`) are identical. The only unique content in dx-charts is these theme constants — append them to `~/repos/dioxus-common/crates/common-charts/src/theme.rs`:

```rust
pub const FLOW_AGENT: (&str, &str) = ("#e1f5fe", "#0288d1");
pub const FLOW_TOOL: (&str, &str) = ("#f3e5f5", "#7b1fa2");
pub const FLOW_DECISION: (&str, &str) = ("#fff3e0", "#ef6c00");
pub const FLOW_LOOP: (&str, &str) = ("#e8f5e9", "#388e3c");
pub const FLOW_RESOURCE: (&str, &str) = ("#fce4ec", "#c62828");
pub const FLOW_USER: (&str, &str) = ("#ede7f6", "#4527a0");
pub const FLOW_IO: (&str, &str) = ("#e0f2f1", "#00695c");
pub const FLOW_TYPE: (&str, &str) = ("#f5f5f5", "#616161");
pub const NO_DATA_AVAILABLE: &str = "No data available";
```

dx-charts also exports `EmptyState` and `ChartExpanded` — common-charts has no equivalents but lx-desktop does not import either of them, so nothing to migrate.

### 2. Copy 9 Rust crates to dioxus-common

For each crate, copy to `~/repos/dioxus-common/crates/<new-name>/` and update the `Cargo.toml`:
- Set `name` to the new name
- Add `[lib] test = false` and `doctest = false` (dioxus-common convention)
- Add `[lints] workspace = true`
- Convert all non-workspace deps to `{ workspace = true }` (see dep table below)
- Convert inter-crate path deps to new names: `inference-client` → `common-inference`, `audio-core` → `common-audio`, `kokoro-client` → `common-kokoro`, `whisper-client` → `common-whisper`

### 3. Update dioxus-common workspace Cargo.toml

**Add to `[workspace.members]`:**
```
"crates/common-audio",
"crates/common-inference",
"crates/common-kokoro",
"crates/common-whisper",
"crates/common-voice",
"crates/common-cdp",
"crates/common-pane-tree",
"crates/common-pty",
"crates/dioxus-widget-bridge",
```

**Add to `[workspace.dependencies]`** (new entries — these are not currently in dioxus-common):
```toml
async-trait = { version = "0.1" }
chromiumoxide = { version = "0.9" }
common-audio = { path = "crates/common-audio" }
common-cdp = { path = "crates/common-cdp" }
common-inference = { path = "crates/common-inference" }
common-kokoro = { path = "crates/common-kokoro" }
common-pty = { path = "crates/common-pty" }
common-pane-tree = { path = "crates/common-pane-tree" }
common-voice = { path = "crates/common-voice" }
common-whisper = { path = "crates/common-whisper" }
dioxus-widget-bridge = { path = "crates/dioxus-widget-bridge" }
parking_lot = { version = "0.12.5" }
portable-pty = { version = "0.8" }
tokio-tungstenite = { version = "0.29.0", features = ["native-tls"] }
url = { version = "2" }
```

**Update existing entries:**
- `reqwest`: change from `{ version = "0.12.15", features = ["json"] }` to `{ version = "0.13.2", features = ["json", "query"] }` — inference-client uses 0.13.2 and needs the `query` feature. This is a semver-minor bump; check that existing dioxus-common crates using reqwest still compile.

**Already present and compatible (no changes needed):**
- `anyhow` (1.0.102 satisfies "1")
- `axum` (0.8 — voice-agent adds `features = ["ws"]` locally, not in workspace)
- `base64` (0.22.1 satisfies "0.22")
- `dashmap` (6.1.0)
- `futures` (0.3.32)
- `regex` (1.11.1)
- `serde` (1.0.228)
- `serde_json` (1.0.149)
- `tokio` (1.50.0 — has all needed features: macros, rt-multi-thread, sync, time)
- `tokio-util` (0.7.18)
- `tracing` (0.1.44)
- `uuid` (1.22.0 with serde+v4)

### 4. Move 3 TS packages to dioxus-common

Copy (excluding `node_modules/` and `dist/`):
- `ts/audio-capture/` → `~/repos/dioxus-common/ts/audio-capture/`
- `ts/audio-playback/` → `~/repos/dioxus-common/ts/audio-playback/`
- `ts/widget-bridge/` → `~/repos/dioxus-common/ts/widget-bridge/`

In each `package.json`, rename from `@lx/<name>` to drop the scope (just `audio-capture`, `audio-playback`, `widget-bridge`) or use a shared scope — match whatever convention dioxus-common's existing packages (`ts/charts`, `ts/primitives`) use (check their package.json names).

In `ts/widget-bridge/package.json`, update the workspace deps:
```json
"@lx/audio-capture": "workspace:*"  →  "<new-name>": "workspace:*"
"@lx/audio-playback": "workspace:*"  →  "<new-name>": "workspace:*"
```

In `ts/widget-bridge/tsconfig.json`, update the path aliases:
```json
"@lx/audio-capture": ["../audio-capture/src/index.ts"]  →  "<new-name>": [...]
"@lx/audio-playback": ["../audio-playback/src/index.ts"]  →  "<new-name>": [...]
```

Also update any source `.ts` files in `ts/widget-bridge/` that import from `@lx/audio-capture` or `@lx/audio-playback`.

Run `pnpm install` in dioxus-common root after moving.

### 5. Remove from lx workspace

**Delete directories:**
- `crates/audio-core/`
- `crates/browser-cdp/`
- `crates/dx-charts/`
- `crates/inference-client/`
- `crates/kokoro-client/`
- `crates/pane-tree/`
- `crates/pty-mux/`
- `crates/voice-agent/`
- `crates/whisper-client/`
- `crates/widget-bridge/`
- `ts/dx-charts/`
- `ts/widget-bridge/`
- `ts/audio-capture/`
- `ts/audio-playback/`

**Update `~/repos/lx/Cargo.toml`** — remove these from `[workspace.members]`:
```
"crates/pane-tree",
"crates/pty-mux",
"crates/widget-bridge",
"crates/dx-charts",
"crates/inference-client",
"crates/whisper-client",
"crates/kokoro-client",
"crates/audio-core",
"crates/voice-agent",
"crates/browser-cdp",
```

**Workspace deps: keep ALL 33.** Every workspace dependency is used by at least one lx-core crate (`lx`, `lx-cli`, `lx-macros`, or `lx-desktop`). None can be removed.

### 6. Update lx-desktop Cargo.toml

Replace the 8 path deps in `crates/lx-desktop/Cargo.toml` with relative path deps to dioxus-common (both repos are siblings under `~/repos/`):

```toml
# Replace these lines:
pane-tree = { path = "../pane-tree" }
pty-mux = { path = "../pty-mux" }
widget-bridge = { path = "../widget-bridge" }
audio-core = { path = "../audio-core" }
whisper-client = { path = "../whisper-client" }
kokoro-client = { path = "../kokoro-client" }
voice-agent = { path = "../voice-agent" }
browser-cdp = { path = "../browser-cdp" }

# With:
common-pane-tree = { path = "../../../dioxus-common/crates/common-pane-tree" }
common-pty = { path = "../../../dioxus-common/crates/common-pty" }
dioxus-widget-bridge = { path = "../../../dioxus-common/crates/dioxus-widget-bridge" }
common-audio = { path = "../../../dioxus-common/crates/common-audio" }
common-whisper = { path = "../../../dioxus-common/crates/common-whisper" }
common-kokoro = { path = "../../../dioxus-common/crates/common-kokoro" }
common-voice = { path = "../../../dioxus-common/crates/common-voice" }
common-cdp = { path = "../../../dioxus-common/crates/common-cdp" }
common-charts = { path = "../../../dioxus-common/crates/common-charts" }
```

Note: path is `../../../dioxus-common/...` because it's relative from `crates/lx-desktop/Cargo.toml` → `~/repos/lx/crates/lx-desktop/` up to `~/repos/`.

### 7. Update lx-desktop Rust imports

Exact file-by-file changes:

**`crates/lx-desktop/src/terminal/view.rs`:**
- `use pane_tree::TabsState;` → `use common_pane_tree::TabsState;`
- `use pane_tree::{NotificationLevel, PaneNotification};` → `use common_pane_tree::{NotificationLevel, PaneNotification};`
- `use widget_bridge::use_ts_widget;` → `use dioxus_widget_bridge::use_ts_widget;`
- `pty_mux::get_or_create(` → `common_pty::get_or_create(`
- `DxCharts.initChart(` → `DioxusCharts.initChart(` (line 145, JS eval string)
- `DxCharts.disposeChart(` → `DioxusCharts.disposeChart(` (line 149, JS eval string)

**`crates/lx-desktop/src/terminal/mod.rs`:**
- `use pane_tree::{PaneNode, Tab, TabsState};` → `use common_pane_tree::{PaneNode, Tab, TabsState};`

**`crates/lx-desktop/src/terminal/browser_view.rs`:**
- `use widget_bridge::use_ts_widget;` → `use dioxus_widget_bridge::use_ts_widget;`
- `browser_cdp::get_or_create_session(` → `common_cdp::get_or_create_session(`
- `browser_cdp::remove_session(` → `common_cdp::remove_session(`

**`crates/lx-desktop/src/pages/terminals.rs`:**
- `use pane_tree::{DividerInfo, Pane, PaneNode, Rect, SplitDirection, TabsState};` → `use common_pane_tree::{DividerInfo, Pane, PaneNode, Rect, SplitDirection, TabsState};`

**`crates/lx-desktop/src/terminal/toolbar.rs`:**
- `use pane_tree::PaneNode;` → `use common_pane_tree::PaneNode;`

**`crates/lx-desktop/src/layout/shell.rs`:**
- `use pane_tree::{PaneNode, TabsState};` → `use common_pane_tree::{PaneNode, TabsState};`
- `const DX_CHARTS_JS: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/dx-charts.js"));` → change to load `charts.js` from common-charts instead (see build.rs changes below)
- `document::eval(DX_CHARTS_JS);` → `document::eval(CHARTS_JS);`

**`crates/lx-desktop/src/panes.rs`:**
- `use pane_tree::Pane;` → `use common_pane_tree::Pane;`

**`crates/lx-desktop/src/voice_backend.rs`:**
- `use voice_agent::AgentBackend;` → `use common_voice::AgentBackend;`

**`crates/lx-desktop/src/pages/agents/voice_banner.rs`:**
- `use kokoro_client::SpeechRequest;` → `use common_kokoro::SpeechRequest;`
- `use voice_agent::AgentBackend as _;` → `use common_voice::AgentBackend as _;`
- `use whisper_client::InferenceClient as _;` → `use common_inference::InferenceClient as _;` (the `InferenceClient` trait is re-exported from inference-client)
- `use whisper_client::TranscribeRequest;` → `use common_whisper::TranscribeRequest;`
- `use widget_bridge::use_ts_widget;` → `use dioxus_widget_bridge::use_ts_widget;`
- `audio_core::wrap_pcm_as_wav(` → `common_audio::wrap_pcm_as_wav(`
- `audio_core::SAMPLE_RATE` → `common_audio::SAMPLE_RATE`
- `audio_core::CHANNELS` → `common_audio::CHANNELS`
- `audio_core::BITS_PER_SAMPLE` → `common_audio::BITS_PER_SAMPLE`
- `audio_core::chunk_wav(` → `common_audio::chunk_wav(`
- `whisper_client::WHISPER.infer(` → `common_whisper::WHISPER.infer(`
- `kokoro_client::KOKORO.infer(` → `common_kokoro::KOKORO.infer(`
- `widget_bridge::TsWidgetHandle` → `dioxus_widget_bridge::TsWidgetHandle`

**`crates/lx-desktop/src/app.rs`:**
- Remove: `static _DX_CHARTS_JS: Asset = asset!("/assets/dx-charts.js", ...);`
- Keep: `static _WIDGET_BRIDGE_JS` (widget-bridge.js is still built, just sourced from dioxus-common now)

### 8. Update lx-desktop build.rs

The build.rs at `crates/lx-desktop/build.rs` currently builds `ts/widget-bridge` and `ts/dx-charts` and copies their dist JS into `assets/`. After the move:

- Remove `dx-charts` from the `ts_packages` array (common-charts has its own build.rs that handles this)
- Update `widget-bridge` path from `root.join("ts/widget-bridge")` to point at `~/repos/dioxus-common/ts/widget-bridge` instead. Use the manifest dir to compute the relative path: the build.rs computes `root` as the repo root of lx — change the widget-bridge reference to go through `../../dioxus-common/ts/widget-bridge` relative to root
- Remove the `dx-charts.js` copy entry
- Add a copy entry for `charts.js` from `~/repos/dioxus-common/crates/common-charts/assets/charts.js` to `assets/charts.js` (or rely on common-charts' own build.rs if lx-desktop depends on common-charts as a crate dep)
- Delete `assets/dx-charts.js` (the built artifact)

The `echarts-5.5.1.min.js` asset stays — it's a vendored dependency used by both dx-charts and common-charts.

### 9. Update lx-desktop shell.rs JS loading

In `crates/lx-desktop/src/layout/shell.rs`:
- `const DX_CHARTS_JS` → rename to `const CHARTS_JS` and point at `charts.js` (from common-charts assets)
- `document::eval(DX_CHARTS_JS)` → `document::eval(CHARTS_JS)`

## Verification

1. In `~/repos/dioxus-common`: run `cargo check` — all 9 new crates must compile
2. In `~/repos/dioxus-common`: run `pnpm build` — all 3 new TS packages must build
3. In `~/repos/lx`: run `just diagnose` — lx-desktop must compile with the new path deps
4. Grep both repos for old crate names (`audio_core`, `browser_cdp`, `dx_charts`, `inference_client`, `kokoro_client`, `pane_tree`, `pty_mux`, `voice_agent`, `whisper_client`, `widget_bridge`, `DxCharts`, `@lx/`) to confirm no stale references
