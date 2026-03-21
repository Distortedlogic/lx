# Goal

Copy the charting infrastructure from `~/repos/deap-rs/crates/evolution-studio/` into this repo as a shared `lx-charts` crate, integrate it into lx-desktop via `document::Script` + `document::eval` (matching deap-rs's pattern), set up a TypeScript build pipeline for ECharts rendering, and build a flow graph chart that projects lx program ASTs into an Enso-style interactive node-edge visualization with live execution state.

# Why

- lx-desktop has zero charting capability. The widget system (terminal, agent, browser, editor, markdown, json-viewer, log-viewer) has no chart type. Profiling data, execution timelines, agent metrics, and cost breakdowns have no visual representation.
- deap-rs already has a mature charting stack: `charming` (Rust ECharts bindings) → generic Dioxus components → TypeScript init/formatters/custom renderers → ECharts 5.5.1. 37+ chart components built on 4 generic bases (line, bar, scatter, pie) plus a universal `CharmingChart` wrapper. This infrastructure is reusable.
- `std/diag` already walks the AST and extracts a `Graph` of `{nodes, edges, subgraphs}` with typed node kinds (agent, tool, decision, fork/join, loop, resource, user, io, type) and exports Mermaid flowcharts. The graph model is ready to be projected into an interactive visual — the rendering layer is missing.
- A flow graph visualization is the natural IDE surface for lx programs. lx programs are DAGs of agents, pipes, parallel blocks, and message channels. Showing these as a node-edge graph with live execution state gives developers the observability they need. Enso (formerly Luna) proved that dual-representation (text ↔ graph) works for functional/pipeline languages.

# Architecture Decisions

## Two integration patterns coexist — they don't conflict

**lx-desktop has two TypeScript compilation paths.** They serve different purposes and don't interfere:

1. **Dioxus-managed TypeScript** (`assets/*.ts`): Files like `terminal.ts`, `widgets/*.ts`, `index.ts` are compiled by `dx serve` via its built-in bundler. These use ES module imports, are bundled into the app entry point, and communicate with Rust via the `use_ts_widget` bridge (bidirectional message passing via `document::eval("await LxDesktop.runWidgetBridge(dioxus)")`). Used for widgets that need persistent state and bidirectional communication (terminals, agents, editors).

2. **tsc-compiled TypeScript** (`ts/src/*.ts`): New files like `chart_init.ts`, `formatters.ts`, `flamegraph.ts` are compiled by `tsc` into `assets/js/*.js` as IIFE-wrapped namespace globals (e.g., `var LxCharts; (function(LxCharts) { ... })(LxCharts || ...);`). These are loaded via `document::Script { src: asset!(...) }` tags in `app.rs` before the dx-managed entry point. No module system — just globals on `window`. This matches exactly how deap-rs does it.

These don't conflict: dx handles `assets/` as the asset directory (per `Dioxus.toml`), and the `assets/js/` subdirectory is just static files that dx serves. The IIFE namespace globals are available to all code in the WebView.

## Charts use `document::eval` directly — no widget bridge needed

Standard charts (line, bar, scatter, pie, flamegraph) use the `CharmingChart` component from lx-charts, which calls `document::eval(&format!("LxCharts.initChart('{id}', {json})"))` directly. This matches deap-rs exactly. `chart_init.ts` attaches a `ResizeObserver` for responsive resizing. `use_drop` calls `LxCharts.disposeChart()`. No widget bridge (`use_ts_widget`) is involved — charts don't need bidirectional communication.

The **flow graph** is the exception: it uses `use_ts_widget("flow-graph", ...)` because it needs bidirectional communication for click-to-source (ECharts click event → Dioxus → open editor pane) and live execution state updates (Rust EventBus → widget → ECharts node styling).

## CSS custom properties required for theming

`chart_init.ts` reads CSS custom properties from `document.documentElement` to theme charts:
- `--foreground` → text color (default `#e5e7eb`)
- `--color-chart-axis` → axis line color (default `#404040`)
- `--color-chart-split` → grid line color (default `#333333`)
- `--color-chart-tooltip` → tooltip background (default `#171717`)

lx-desktop must define these in its CSS (Tailwind config or a `<style>` block in `app.rs`). The defaults match a dark theme.

## Formatter wiring via data attributes

`CharmingChart` renders a div with `data-x-fmt`, `data-y-fmt`, `data-tooltip-fmt`, `data-label-fmt`, and `data-extra` attributes. `chart_init.ts` reads these at init time and applies the corresponding formatter functions from `formatters.ts`. For example, `data-y-fmt="money"` causes y-axis labels to render as `$1.2M`, `$500k`, etc. Available formatters: `identity`, `duration`, `fitness`, `percent`, `money`, `moneyFull`, `abbreviate`, `fixed2`, `fixed4`, `round`, `abbreviateCategory`, `megabytes`. Tooltip formatters: `cumulativeGrowth`, `money`, `genTime`, `scatter`, `alps`, `megabytes`.

## ECharts graph series for flow graph (not custom renderItem)

ECharts `type: 'graph'` series supports nodes + edges natively:
- **Node customization**: `symbol` per node (circle, rect, diamond, triangle, pin, arrow, or `'path://SVG_PATH'` for custom shapes), `symbolSize`, `itemStyle` (color, borderColor, borderWidth), `label` (text, position, fontSize).
- **Edge customization**: `lineStyle` (color, width, type: solid/dashed/dotted), `curveness` for curved edges, `symbol` for arrow markers.
- **Categories**: Array of category definitions (name, itemStyle). Each node references a category by index → consistent styling by node kind.
- **Layout**: `layout: 'none'` with explicit x/y per node (Rust computes positions), or `layout: 'force'` for force-directed (simpler but less predictable).
- **Interaction**: `roam: true` for pan/zoom. `chart.on('click', handler)` for click events. Tooltip on hover.

No `renderItem` needed — the graph series handles node/edge rendering. Custom node shapes use `symbol: 'path://M...'` with SVG path data for agent (rounded rect), decision (diamond), loop (rounded rect with cycle), etc.

## Flow graph layout algorithm

v1 uses a simple Sugiyama-style layered layout computed in Rust:
1. **Topological sort** to assign layers (x-axis position). Nodes with no incoming edges go to layer 0. Each node's layer = max(predecessor layers) + 1.
2. **Within each layer**, order nodes to minimize edge crossings using barycenter heuristic (average position of connected nodes in adjacent layers).
3. **Position assignment**: `x = layer_index * LAYER_SPACING` (e.g., 200px), `y = position_in_layer * NODE_SPACING` (e.g., 120px).
4. Positions are serialized into the ECharts graph data as `x`/`y` fields per node.

## Live execution state on the flow graph

The flow graph shows real-time execution status by subscribing to the EventBus:
- `FlowGraphView` component subscribes to the EventBus (already available in Dioxus context from `Shell`).
- A mapping function converts `RuntimeEvent` variants to node IDs:
  - `AgentSpawned { name }` → find node with matching label → status: "running"
  - `AiCallStart { agent }` → find agent node → status: "active"
  - `AiCallComplete { agent }` → find agent node → status: "completed"
  - `Log { level: "error" }` → current context node → status: "error"
  - `ProgramFinished` → all nodes → status: "done"
- Status updates are sent to the flow graph widget via `widget.send_update()`:
  ```json
  { "type": "node-status", "nodeId": "agent_3", "status": "running" }
  ```
- The flow graph TS code updates node styling based on status:
  - `"idle"` → default styling
  - `"running"` → yellow border, subtle glow animation
  - `"completed"` → green border
  - `"error"` → red border
  - `"active"` → pulsing yellow border (AI call in progress)

## Click-to-source navigation

When a user clicks a node in the flow graph:
1. ECharts fires a click event with the node data (including `source_line` from the enriched DiagNode).
2. The flow graph TS code sends a message back to Rust via the widget bridge: `dx.send({ type: 'node-click', nodeId: '...', sourceLine: 42 })`.
3. Rust side receives via `widget.recv()` in the `use_future` loop.
4. Creates an Editor pane at the clicked source line (or focuses an existing editor pane for that file and scrolls to the line).

## How FlowGraphView calls the parser

The FlowGraphView component runs the full parse pipeline in Rust, inside a `use_future`:
1. Read the `.lx` file via `std::fs::read_to_string(&source_path)`.
2. Lex: `lx::lexer::lex(&source)` → `Vec<Token>` (or error).
3. Parse: `lx::parser::parse(tokens)` → `Program` (or error).
4. Walk: `let mut walker = Walker::new(); walker.visit_program(&program);`
5. Extract: `let graph = walker.into_graph();`
6. Layout: compute Sugiyama positions (new function in `diag.rs`).
7. Serialize: convert graph + positions to ECharts graph series JSON.
8. Send to widget via `widget.send_update()`.

On parse/lex error, display the error message in the pane instead of the graph. On file change (watched via `use_future` polling or file watcher), re-parse and update.

# Source Files Reference

## deap-rs files to copy

All paths relative to `~/repos/deap-rs/crates/evolution-studio/`:

| Source file | Destination | Modifications |
|---|---|---|
| `src/shared/components/charts/types.rs` | `crates/lx-charts/src/types.rs` | Remove deap imports. Add `LegendPosition` enum. |
| `src/shared/theme.rs` | `crates/lx-charts/src/theme.rs` | Keep generic colors. Remove TREE_NODE_*, AGE_GRADIENT_*, PROFILING_*, INDICATOR_*. Add FLOW_* node colors. |
| `src/shared/components/charts/charming_wrapper.rs` | `crates/lx-charts/src/charming_wrapper.rs` | Namespace `DeapCharts` → `LxCharts`. Remove `crate::shared::*` imports. |
| `src/shared/components/charts/line_chart.rs` | `crates/lx-charts/src/line.rs` | Namespace → `LxCharts`. Replace `crate::shared::{ChartExpanded, EmptyState, constants::*}` with local equivalents. |
| `src/shared/components/charts/bar_chart.rs` | `crates/lx-charts/src/bar.rs` | Same as above. |
| `src/shared/components/charts/scatter_chart.rs` | `crates/lx-charts/src/scatter.rs` | Same as above. |
| `src/shared/components/charts/pie_chart.rs` | `crates/lx-charts/src/pie.rs` | Same as above. |
| `src/shared/components/expandable_chart.rs` | `crates/lx-charts/src/expandable.rs` | Replace `dioxus_free_icons` icons with inline SVG or text buttons. Replace `dioxus_primitives::dialog` with a simple div overlay. Remove `crate::shared::constants::FLEX_CENTER_GAP` (inline the class string). |
| `ts/src/echarts.d.ts` | `crates/lx-desktop/ts/src/echarts.d.ts` | Verbatim. |
| `ts/src/chart_init.ts` | `crates/lx-desktop/ts/src/chart_init.ts` | Namespace `DeapCharts` → `LxCharts`. |
| `ts/src/formatters.ts` | `crates/lx-desktop/ts/src/formatters.ts` | Namespace `DeapCharts` → `LxCharts`. |
| `ts/src/flamegraph.ts` | `crates/lx-desktop/ts/src/flamegraph.ts` | Namespace `DeapCharts` → `LxCharts`. |
| `ts/src/candlestick_render_item.ts` | `crates/lx-desktop/ts/src/candlestick_render_item.ts` | Verbatim. |
| `ts/package.json` | `crates/lx-desktop/ts/package.json` | Change name to `lx-desktop-ts`. |
| `ts/tsconfig.json` | `crates/lx-desktop/ts/tsconfig.json` | Verbatim. |
| `assets/echarts-5.5.1.min.js` | `crates/lx-desktop/assets/echarts-5.5.1.min.js` | Verbatim. |

## deap-rs dependencies needed

The generic chart components reference types from deap-rs shared modules that don't exist in lx-charts. These must be created locally:

- **`EmptyState` component**: Used by GenericLineChart, GenericBarChart, GenericScatterChart, GenericPieChart when all series are empty. Create a simple `EmptyState` component in `crates/lx-charts/src/empty_state.rs`: a centered div with muted text showing the message.
- **`NO_DATA_AVAILABLE` constant**: `"No data available"` — define in `theme.rs` or inline.
- **`ChartExpanded` context signal**: Used by GenericLineChart to conditionally show DataZoom. Keep it in `expandable.rs` and import from there.
- **`dioxus_free_icons`**: ExpandableChart uses `LdMaximize2` and `LdX` icons. Replace with inline SVG strings or simple `"⤢"` / `"✕"` text buttons to avoid adding the dependency.
- **`dioxus_primitives::dialog`**: ExpandableChart uses `DialogRoot`/`DialogContent` for the expanded modal. Replace with a simple fixed-position full-screen overlay div. No need for the primitives crate.
- **`FLEX_CENTER_GAP`**: Just a Tailwind class string like `"flex items-center gap-2"`. Inline it.

## Existing lx infrastructure to use

- **`std/diag` (Rust)**: `crates/lx/src/stdlib/diag.rs`, `diag_types.rs`, `diag_walk.rs`, `diag_walk_expr.rs`, `diag_helpers.rs`. The `Walker` implements `AstVisitor` and produces `Graph { nodes: Vec<DiagNode>, edges: Vec<DiagEdge>, subgraphs: Vec<Subgraph> }`. Current `DiagNode` has `id`, `label`, `kind`, `children`. Current `DiagEdge` has `from`, `to`, `label`, `style`. Node kinds: `agent`, `tool`, `decision`, `fork`, `join`, `loop`, `resource`, `user`, `io`, `type`. Edge styles: `solid`, `dashed`, `dotted`. The Walker tracks `agent_vars`, `mcp_vars`, `fn_nodes`, `handler_maps`, `resource_vars`, `imported_modules` for context-aware edge creation.
- **`diag.extract(source_str)`**: Parses lx source string → Graph value (Record with nodes/edges lists).
- **`diag.extract_file(path)`**: Reads file → parses → Graph value.
- **`diag.to_mermaid(graph)`**: Converts Graph value → Mermaid flowchart string.
- **`extract_mermaid(program: &Program)`**: Rust-only function that takes a parsed Program and returns Mermaid string directly.
- **`ts_widget.rs`**: `use_ts_widget(widget_type, config)` returns `(element_id, TsWidgetHandle)`. Handle has `send_update()`, `send_resize()`, `recv::<T>()`. Used for widgets needing bidirectional communication.
- **`EventBus`**: Broadcast channel (`Arc<EventBus>`) provided via Dioxus context in `Shell`. `bus.subscribe()` returns a receiver for `RuntimeEvent` variants. Available events: Log, ShellExec, AiCallStart, AiCallComplete, Progress, AgentSpawned, etc.
- **`PaneNode`**: Enum in `lx-ui/pane_tree/mod.rs` with variants Terminal, Browser, Editor, Agent, Canvas, Split. Operations: split, close, convert, set_ratio, compute_pane_rects, compute_dividers.
- **`CanvasView`**: Existing generic pane that takes a widget_type + config and renders via `use_ts_widget`. Already used for log-viewer, markdown, json-viewer.
- **charming reference**: `~/repos/deap-rs/reference/charming/` submodule contains API examples and gallery code for the charming crate.

# Files Affected

**New crate `crates/lx-charts/`:**
- `Cargo.toml`
- `src/lib.rs`
- `src/types.rs`
- `src/theme.rs`
- `src/empty_state.rs`
- `src/charming_wrapper.rs`
- `src/expandable.rs`
- `src/line.rs`
- `src/bar.rs`
- `src/scatter.rs`
- `src/pie.rs`

**New TypeScript pipeline `crates/lx-desktop/ts/`:**
- `package.json`
- `tsconfig.json`
- `src/echarts.d.ts`
- `src/chart_init.ts`
- `src/formatters.ts`
- `src/flamegraph.ts`
- `src/candlestick_render_item.ts`
- `src/flow_graph.ts`

**New/modified in `crates/lx-desktop/`:**
- `assets/echarts-5.5.1.min.js`
- `assets/js/` — compiled TypeScript output (6 .js files)
- `assets/widgets/flow-graph.ts` — flow graph widget (bidirectional)
- `assets/index.ts` — register flow-graph widget
- `src/app.rs` — load ECharts + JS scripts, add CSS custom properties
- `src/terminal/view.rs` — add ChartView, FlowGraphView components
- `src/pages/run.rs` — add Flow Graph button
- `src/pages/terminals.rs` — handle Chart and FlowGraph pane rendering
- `src/terminal/tab_bar.rs` — add Flow Graph to new-pane dropdown
- `.gitignore` — add `ts/node_modules/`

**Modified in `crates/lx/`:**
- `src/stdlib/diag_types.rs` — add DiagPort, enrich DiagNode, DiagEdge
- `src/stdlib/diag_walk.rs` — populate new fields (ports, source_line, edge_type)
- `src/stdlib/diag_walk_expr.rs` — set edge_type on agent/pipe/exec edges
- `src/stdlib/diag.rs` — add `diag.to_graph_chart`, layout computation, update serialization
- `src/stdlib/mod.rs` — register `to_graph_chart` if not auto-registered via `build()`

**Modified:**
- `Cargo.toml` (workspace root) — add lx-charts to members, add charming dependency
- `crates/lx-ui/src/pane_tree/mod.rs` — add Chart, FlowGraph pane variants
- `crates/lx-desktop/Cargo.toml` — add lx-charts, lx dependencies
- `justfile` — add ts-build recipe

# What Changes

## Phase 1: Create lx-charts crate

Extract the generic charting infrastructure from deap-rs into `crates/lx-charts/`. See "Source Files Reference" above for exact file mappings and required modifications.

The crate provides:
- Data types: `LineSeries`, `BarSeries`, `ScatterSeries`, `PieSlice`, `DataZoomConfig`, `LegendPosition`, `LineData`
- Theme constants: generic chart colors + flow graph node kind colors
- Generic Dioxus components: `GenericLineChart`, `GenericBarChart`, `GenericScatterChart`, `GenericPieChart`
- `CharmingChart`: universal wrapper — serializes `charming::Chart` to JSON, renders div with data-format attributes, calls `LxCharts.initChart()` via `document::eval`, disposes on drop
- `ExpandableChart`: collapse/expand wrapper with full-screen modal
- `EmptyState`: placeholder for empty charts

Dependencies: `charming = { git = "https://github.com/yuankunzhang/charming.git" }`, `dioxus = "0.7"`, `serde`, `serde_json`, `uuid`.

## Phase 2: TypeScript build pipeline

Set up `crates/lx-desktop/ts/` with raw tsc compilation (no bundler). TypeScript sources in `ts/src/` compile to `assets/js/` as IIFE namespace globals. Copy all TS source files from deap-rs (chart_init, formatters, flamegraph, candlestick_render_item, echarts.d.ts) with `DeapCharts` → `LxCharts` namespace rename. Copy `echarts-5.5.1.min.js` to `assets/`.

## Phase 3: Chart integration in lx-desktop

Load ECharts and compiled JS files via `document::Script` in `app.rs`. Add CSS custom properties for chart theming. Add `Chart` variant to `PaneNode`. Create `ChartView` component that renders `CharmingChart` inside a pane. Standard charts use `CharmingChart` + `document::eval` directly — no widget bridge needed.

## Phase 4: Flow graph visualization

Enrich `std/diag`'s graph model with ports, source line references, and edge types. Add Sugiyama layout computation in Rust. Create a `"flow-graph"` widget using `use_ts_widget` for bidirectional communication (click events back to Rust, execution state updates to ECharts). The widget uses ECharts `type: 'graph'` series with `layout: 'none'` and Rust-computed positions. Node shapes use custom SVG path symbols per kind. Edge styles vary by type (data=solid gray, exec=solid dark, agent=solid orange, stream=dashed cyan). Live execution state from EventBus updates node border colors/glow in real-time.

# Task List

### Task 1: Create lx-charts crate with types, theme, and empty state

**Subject:** Create lx-charts crate with Cargo.toml, types.rs, theme.rs, empty_state.rs

**Description:** Create `crates/lx-charts/`.

In its `Cargo.toml`: package name `lx-charts`, edition 2024, lib-only. Dependencies: `charming = { git = "https://github.com/yuankunzhang/charming.git" }`, `dioxus = "0.7"`, `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `uuid = { version = "1", features = ["v4"] }`.

Create `src/lib.rs` exporting: `pub mod types;`, `pub mod theme;`, `pub mod empty_state;`.

Create `src/types.rs`. Copy from `~/repos/deap-rs/crates/evolution-studio/src/shared/components/charts/types.rs`. Contains:
- `LineData` enum (Pairs `Vec<Vec<f64>>` / Values `Vec<f64>`) with `len()`, `is_empty()`, `From` impls
- `LineSeries` struct: name (String), data (LineData), color (&'static str), width (Option<u32>), area (bool), stack (Option<String>), show_symbol (bool)
- `BarSeries` struct: name (String), data (Vec<f64>), color (Option<&'static str>), stack (Option<String>)
- `ScatterSeries` struct: name (String), data (Vec<Vec<f64>>), color (&'static str), symbol_size (Option<f64>)
- `PieSlice` struct: name (String), value (f64), color (Option<&'static str>)
- `DataZoomConfig` struct: start (f64), end (f64), slider (bool), inside (bool) — with `last_n()` helper and Default impl
- `LegendPosition` enum: Top, Bottom, Right (Copy, Default = Top)
All derive `Clone`, `PartialEq`. No deap-specific imports.

Create `src/theme.rs`. Copy from `~/repos/deap-rs/crates/evolution-studio/src/shared/theme.rs`. Keep: GREEN, BLUE, RED, PURPLE, VIOLET, AMBER, ORANGE, PINK, EMERALD, CYAN, YELLOW, DARK_AMBER, CREAM, BLUE_LIGHT, BLUE_DARK, STONE, GRAY_700, NEUTRAL_700, NEUTRAL_900, NEUTRAL_50, GRAY_200, GRAY_400, STONE_900, CHART_PRIMARY, CHART_SECONDARY, CHART_LABEL_PRIMARY, CHART_LABEL_SECONDARY, LAYER_COLORS, LAYER_COLOR_DEFAULT, FLAMEGRAPH_COLORS. Remove: TREE_NODE_COLOR, TREE_NODE_BORDER, TREE_COLORS, AGE_GRADIENT_START/END, PROFILING_FG/BORDER/TOOLTIP_BG, INDICATOR_COLORS. Add flow graph node kind colors (fill + border) matching std/diag's Mermaid classDef:
- `FLOW_AGENT: (&str, &str) = ("#e1f5fe", "#0288d1")`
- `FLOW_TOOL: (&str, &str) = ("#f3e5f5", "#7b1fa2")`
- `FLOW_DECISION: (&str, &str) = ("#fff3e0", "#ef6c00")`
- `FLOW_LOOP: (&str, &str) = ("#e8f5e9", "#388e3c")`
- `FLOW_RESOURCE: (&str, &str) = ("#fce4ec", "#c62828")`
- `FLOW_USER: (&str, &str) = ("#ede7f6", "#4527a0")`
- `FLOW_IO: (&str, &str) = ("#e0f2f1", "#00695c")`
- `FLOW_TYPE: (&str, &str) = ("#f5f5f5", "#616161")`
Add `NO_DATA_AVAILABLE: &str = "No data available"`.

Create `src/empty_state.rs`. Simple Dioxus component:
```rust
#[component]
pub fn EmptyState(message: String) -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center h-full text-gray-500 text-sm",
            "{message}"
        }
    }
}
```

Add `"crates/lx-charts"` to the root workspace members. Run `just diagnose`.

**ActiveForm:** Creating lx-charts crate with types, theme, and empty state

---

### Task 2: Copy CharmingChart wrapper and ExpandableChart

**Subject:** Copy charming_wrapper.rs and expandable.rs from deap-rs, adapting dependencies

**Description:** Create two files in `crates/lx-charts/src/`:

`charming_wrapper.rs` — Copy from `~/repos/deap-rs/crates/evolution-studio/src/shared/components/charts/charming_wrapper.rs`. The `CharmingChart` Dioxus component takes: `chart: Chart`, `title: Option<&'static str>`, `x_fmt: Option<&'static str>`, `y_fmt: Option<&'static str>`, `tooltip_fmt: Option<&'static str>`, `label_fmt: Option<&'static str>`, `extra_data: Option<String>`. It:
1. Generates a unique div ID via `use_memo(|| format!("chart-{}", Uuid::new_v4().simple()))`.
2. If title is Some, adds `Title::new().text(t)` to the chart.
3. Serializes chart to JSON via `chart.to_string()`, stores in a signal.
4. In `use_effect`, calls `document::eval(&format!("LxCharts.initChart('{id}', {json})"))`  when JSON changes.
5. In `use_drop`, calls `document::eval(&format!("LxCharts.disposeChart('{id}')"))`.
6. Renders a div with `id`, `class: "w-full h-full min-h-32"`, and data attributes: `data-x-fmt`, `data-y-fmt`, `data-tooltip-fmt`, `data-label-fmt`, `data-extra`.

Change namespace from `DeapCharts` to `LxCharts` in the eval strings. Import from `charming::{Chart, component::Title}`, `dioxus::prelude::*`, `uuid::Uuid`.

`expandable.rs` — Adapted from `~/repos/deap-rs/crates/evolution-studio/src/shared/components/expandable_chart.rs`. The `ExpandableChart` wraps chart content in a card with a maximize button. **Adaptations to avoid extra dependencies:**
- Replace `dioxus_free_icons::Icon { icon: LdMaximize2 }` with a simple button containing the text `"⤢"` styled as `"text-lg"`.
- Replace `dioxus_free_icons::Icon { icon: LdX }` with a button containing `"✕"`.
- Replace `dioxus_primitives::dialog::{DialogRoot, DialogContent}` with a simple fixed-position overlay div: `div { class: "fixed inset-0 z-50 bg-black/80 flex items-center justify-center", ... }`.
- Replace `crate::shared::constants::FLEX_CENTER_GAP` with the inline class `"flex items-center gap-2"`.
- Keep `ChartExpanded(pub bool)` context signal for DataZoom visibility control.

Add `pub mod charming_wrapper;`, `pub mod expandable;` to `lib.rs`. Run `just diagnose`.

**ActiveForm:** Copying CharmingChart wrapper and ExpandableChart

---

### Task 3: Copy generic chart components from deap-rs

**Subject:** Copy GenericLineChart, GenericBarChart, GenericScatterChart, GenericPieChart

**Description:** Create four files in `crates/lx-charts/src/`:

`line.rs` — Copy from `~/repos/deap-rs/crates/evolution-studio/src/shared/components/charts/line_chart.rs`. `GenericLineChart` component. **Adaptations:**
- Replace `use crate::shared::{ChartExpanded, EmptyState, constants::NO_DATA_AVAILABLE}` with `use crate::expandable::ChartExpanded`, `use crate::empty_state::EmptyState`, `use crate::theme::NO_DATA_AVAILABLE`.
- Replace `use super::charming_wrapper::CharmingChart` with `use crate::charming_wrapper::CharmingChart`.
- Replace `use super::types::*` with `use crate::types::*`.
- Keep the full charming imports: `Chart, component::{Axis, DataZoom, DataZoomType, Legend, LegendSelectedMode}, element::{AreaStyle, AxisType, Color, Emphasis, EmphasisFocus, ItemStyle, LineStyle, Orient, Tooltip, Trigger}, series::Line`.

`bar.rs` — Copy from `~/repos/deap-rs/crates/evolution-studio/src/shared/components/charts/bar_chart.rs`. Same adaptation pattern. Uses `charming::series::Bar`, `element::AxisLabel`.

`scatter.rs` — Copy from `~/repos/deap-rs/crates/evolution-studio/src/shared/components/charts/scatter_chart.rs`. Same pattern. Uses `charming::series::Scatter`.

`pie.rs` — Copy from `~/repos/deap-rs/crates/evolution-studio/src/shared/components/charts/pie_chart.rs`. Same pattern. Uses `charming::series::Pie`, `charming::datatype::DataPointItem`, `element::{Formatter, Label}`.

Add `pub mod line;`, `pub mod bar;`, `pub mod scatter;`, `pub mod pie;` to `lib.rs`. Run `just diagnose`.

**ActiveForm:** Copying generic chart components from deap-rs

---

### Task 4: Set up TypeScript build pipeline in lx-desktop

**Subject:** Create ts/ directory with package.json, tsconfig.json, copy TS source and ECharts from deap-rs

**Description:** Create `crates/lx-desktop/ts/`.

Create `ts/package.json`:
```json
{
  "name": "lx-desktop-ts",
  "private": true,
  "scripts": {
    "build": "tsc",
    "check": "tsc --noEmit"
  },
  "devDependencies": {
    "typescript": "^5.7.0"
  }
}
```

Create `ts/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES5",
    "outDir": "../assets/js",
    "rootDir": "src",
    "strict": true,
    "removeComments": true,
    "sourceMap": false,
    "declaration": false,
    "lib": ["ES5", "DOM"]
  },
  "include": ["src/**/*.ts"]
}
```

Copy TypeScript source files from `~/repos/deap-rs/crates/evolution-studio/ts/src/` to `crates/lx-desktop/ts/src/`:
- `echarts.d.ts` — Verbatim. Contains type declarations for `echarts.init`, `echarts.getInstanceByDom`, `echarts.graphic.clipRectByRect`, `ECharts` interface (setOption, dispose, resize, dispatchAction), `RenderItemParams`, `RenderItemApi`, `RenderItemShape`, `RenderItemElement`.
- `chart_init.ts` — Copy, replace all occurrences of `DeapCharts` with `LxCharts`. The file defines `LxCharts.initChart(id, opts)` which: reads CSS custom properties for theming, applies axis formatters from data attributes, creates/reuses ECharts instance, attaches ResizeObserver. Also `LxCharts.disposeChart(id)` and `LxCharts.restoreChart(id)`.
- `formatters.ts` — Copy, replace `DeapCharts` with `LxCharts`. Contains 13 axis formatters and 5 tooltip formatters.
- `flamegraph.ts` — Copy, replace `DeapCharts` with `LxCharts`. Defines `LxCharts.setupFlamegraph(id, data, maxY)` with custom renderItem for rect-based flame charts.
- `candlestick_render_item.ts` — Copy verbatim (standalone function, no namespace).

**IMPORTANT**: In `chart_init.ts`, the formatter references (`DeapCharts.formatIdentity`, etc.) in the `AXIS_FORMATTERS` map must also be updated to `LxCharts.formatIdentity`, etc. Same for tooltip formatter references like `DeapCharts.cumulativeGrowthTooltip` → `LxCharts.cumulativeGrowthTooltip`, `DeapCharts.formatMoneyFull` → `LxCharts.formatMoneyFull`, etc.

Copy `~/repos/deap-rs/crates/evolution-studio/assets/echarts-5.5.1.min.js` to `crates/lx-desktop/assets/echarts-5.5.1.min.js`.

Add `ts/node_modules/` to `crates/lx-desktop/.gitignore` (create the file if it doesn't exist).

Run `cd crates/lx-desktop/ts && npm install && npm run build` to compile TypeScript and generate `assets/js/chart_init.js`, `assets/js/formatters.js`, `assets/js/flamegraph.js`, `assets/js/candlestick_render_item.js`.

Add justfile recipe:
```
[group('build')]
ts-build:
    cd crates/lx-desktop/ts && npm run build
```

**ActiveForm:** Setting up TypeScript build pipeline

---

### Task 5: Load ECharts scripts and add CSS custom properties in lx-desktop

**Subject:** Add document::Script elements and CSS custom properties to app.rs

**Description:** Edit `crates/lx-desktop/src/app.rs`. Add `document::Script` elements inside the `rsx!` block to load ECharts and the compiled JS files. These must appear before the `Router` so globals are available when chart components mount:

```rust
document::Script { src: asset!("/assets/echarts-5.5.1.min.js") }
document::Script { src: asset!("/assets/js/formatters.js") }
document::Script { src: asset!("/assets/js/chart_init.js") }
document::Script { src: asset!("/assets/js/flamegraph.js") }
```

Add a `document::Style` element with CSS custom properties for chart theming (dark theme defaults):
```rust
document::Style {
    r#"
    :root {
        --foreground: #e5e7eb;
        --color-chart-axis: #404040;
        --color-chart-split: #333333;
        --color-chart-tooltip: #171717;
    }
    "#
}
```

(If lx-desktop already has Tailwind or a theme stylesheet that defines `--foreground`, these can be merged there instead of a separate Style block.)

Add `lx-charts = { path = "../lx-charts" }` to `crates/lx-desktop/Cargo.toml`.

Run `just diagnose`.

**ActiveForm:** Loading ECharts scripts and adding CSS custom properties

---

### Task 6: Add Chart pane type and ChartView component

**Subject:** Add Chart variant to PaneNode and create ChartView Dioxus component

**Description:** Edit `crates/lx-ui/src/pane_tree/mod.rs`. Add a `Chart` variant to `PaneNode`:
```rust
Chart {
    id: String,
    chart_json: String,
    title: Option<String>,
}
```
Update all match arms that handle PaneNode variants to include Chart (id extraction in `pane_id()`, display, all_pane_ids, first_terminal_id returns None, find_working_dir returns None, etc.).

Edit `crates/lx-desktop/src/terminal/view.rs`. Add `ChartView` component:
```rust
#[component]
pub fn ChartView(chart_id: String, chart_json: String, title: Option<String>) -> Element {
    let chart_json_owned = chart_json.clone();
    let chart: charming::Chart = serde_json::from_str(&chart_json_owned).unwrap_or_default();
    rsx! {
        lx_charts::charming_wrapper::CharmingChart {
            chart,
            title: title.as_deref(),
        }
    }
}
```

If `charming::Chart` doesn't implement `Deserialize` (it may not — check), then instead render the chart div manually and call `LxCharts.initChart` via `document::eval`, passing the raw JSON string directly:
```rust
#[component]
pub fn ChartView(chart_id: String, chart_json: String, title: Option<String>) -> Element {
    let div_id = use_hook(|| format!("chart-{}", uuid::Uuid::new_v4().simple()));
    let id = div_id.clone();
    use_effect(move || {
        let json = chart_json.clone();
        if json.is_empty() { return; }
        document::eval(&format!("LxCharts.initChart('{id}', {json})"));
    });
    let id_drop = div_id.clone();
    use_drop(move || {
        document::eval(&format!("LxCharts.disposeChart('{id_drop}')"));
    });
    rsx! {
        div { id: "{div_id}", class: "w-full h-full min-h-32" }
    }
}
```

Update the pane rendering logic in `crates/lx-desktop/src/pages/terminals.rs` to handle the Chart variant by rendering `ChartView`.

Run `just diagnose`.

**ActiveForm:** Adding Chart pane type and ChartView component

---

### Task 7: Enrich std/diag graph model for flow graph rendering

**Subject:** Add ports, edge types, source lines, and layout computation to DiagNode/DiagEdge

**Description:** Edit `crates/lx/src/stdlib/diag_types.rs`. Add new struct and enrich existing types:

```rust
pub(crate) struct DiagPort {
    pub id: String,
    pub label: String,
    pub direction: String,
    pub port_type: String,
}

pub(crate) struct DiagNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub children: Vec<DiagNode>,
    pub ports: Vec<DiagPort>,
    pub source_line: Option<usize>,
}

pub(crate) struct DiagEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub style: String,
    pub edge_type: String,
    pub from_port: Option<String>,
    pub to_port: Option<String>,
}
```

Edit `crates/lx/src/stdlib/diag_walk.rs`:
- Update `Walker::add_node` to accept an optional `source_line: Option<usize>` parameter. Initialize `ports: vec![]` and the passed `source_line`.
- Update `Walker::add_edge` to accept `edge_type: &str`. Default to `"exec"` for existing edges.

Edit `crates/lx/src/stdlib/diag_walk_expr.rs`:
- For `Expr::AgentSend`: set `edge_type` to `"agent"`.
- For `Expr::AgentAsk`: set `edge_type` to `"agent"`.
- For `Expr::StreamAsk`: set `edge_type` to `"stream"`.
- For `Expr::Pipe` (if visited): set `edge_type` to `"data"`.
- For all other edges (apply calls, sequential flow): keep `edge_type` as `"exec"`.
- Pass `span.start_line()` (or equivalent — check `Span` struct for line number access) to `add_node` for `source_line`.
- Add ports based on node kind: agent nodes get an `"out"` port with `port_type: "agent"`; function call nodes get `"in"` (data) and `"out"` (data) ports; decision/match nodes get `"in"` (exec) and multiple `"out"` (exec) ports for each arm.

Edit `crates/lx/src/stdlib/diag.rs`:
- Update `node_to_value` to include `"ports"` (list of port records with id, label, direction, port_type) and `"source_line"` (Int or None).
- Update `edge_to_value` to include `"edge_type"`, `"from_port"` (Str or None), `"to_port"` (Str or None).
- Update `value_to_node` and `value_to_edge` to read the new fields with defaults for backward compatibility (empty ports list, None source_line, "exec" edge_type, None from/to_port).
- Add a public Rust function `pub fn graph_to_echart_json(graph: &Graph) -> String` that takes a Graph struct and returns an ECharts graph series JSON string. This function contains the layout and serialization logic and is called by both the lx builtin (`bi_to_graph_chart`) and the `FlowGraphView` Dioxus component (task 9). Also add `bi_to_graph_chart` registered as `"to_graph_chart"` in `build()` with arity 1 — this lx-accessible wrapper takes a Graph value, converts it to a Graph struct via `value_to_graph`, calls `graph_to_echart_json`, and returns the JSON as `Value::Str`. Layout algorithm used by `graph_to_echart_json`:
  1. Build adjacency list from edges.
  2. Topological sort to assign layers (BFS from roots — nodes with no incoming edges).
  3. Within each layer, order by barycenter of connected nodes in previous layer.
  4. Assign positions: `x = layer * 200.0`, `y = position_in_layer * 120.0`.
  5. Build ECharts JSON: `{ series: [{ type: "graph", layout: "none", roam: true, nodes: [...], edges: [...], categories: [...] }] }`. Each node: `{ name, x, y, symbol, symbolSize, category, itemStyle, label, value: { kind, sourceLine } }`. Symbol per kind: agent=`"roundRect"`, tool=`"circle"`, decision=`"diamond"`, fork/join=`"rect"` (small), loop=`"roundRect"`, resource=`"circle"`, user=`"triangle"`, io=`"arrow"`, type=`"circle"`. Categories array: one per kind with fill/border colors from theme. Each edge: `{ source, target, lineStyle: { type, color, width } }`. Edge lineStyle varies by edge_type: data → solid gray, exec → solid `#666`, agent → solid `#f97316` (orange), stream → dashed `#06b6d4` (cyan).

Run `just diagnose` and `just test` (ensure existing diag tests still pass).

**ActiveForm:** Enriching std/diag graph model

---

### Task 8: Create flow graph widget in TypeScript

**Subject:** Create flow_graph.ts for ECharts graph interaction and flow-graph widget registration

**Description:** Create `crates/lx-desktop/ts/src/flow_graph.ts` in the `LxCharts` namespace:

```typescript
namespace LxCharts {
    export function initFlowGraph(id: string, graphJson: any): void {
        // Initialize ECharts graph series with the provided data
        // Enable roam (pan/zoom)
        // Apply theme colors from CSS custom properties
        // Set up click handler that stores a callback
    }
    export function updateFlowGraphStatus(id: string, nodeId: string, status: string): void {
        // Update a node's itemStyle based on status:
        // "idle" → default colors from category
        // "running" → yellow border (#eab308), borderWidth 3
        // "completed" → green border (#22c55e), borderWidth 2
        // "error" → red border (#ef4444), borderWidth 3
        // "active" → pulsing yellow (use animation or bright yellow)
        // Call chart.setOption with the updated node data
    }
}
```

Run `cd crates/lx-desktop/ts && npm run build` to generate `assets/js/flow_graph.js`.

Create `crates/lx-desktop/assets/widgets/flow-graph.ts`. This widget uses the `use_ts_widget` bridge for bidirectional communication. **IMPORTANT:** This file is compiled by the Dioxus bundler (not tsc), so it doesn't have access to the `echarts.d.ts` type declarations in `ts/src/`. Add `declare var` statements at the top for the globals loaded via `document::Script`:

```typescript
declare var LxCharts: {
    initFlowGraph(id: string, data: any): void;
    updateFlowGraphStatus(id: string, nodeId: string, status: string): void;
    disposeChart(id: string): void;
};
declare var echarts: {
    getInstanceByDom(el: HTMLElement): any;
};

import { registerWidget, Widget, Dioxus } from '../widgets/registry';

const flowGraphWidget: Widget = {
    mount(id: string, config: any, dx: Dioxus) {
        LxCharts.initFlowGraph(id, config.graphData || {});
        // Set up ECharts click handler to send node-click events back to Rust:
        var el = document.getElementById(id);
        if (el && typeof echarts !== 'undefined') {
            var inst = echarts.getInstanceByDom(el);
            if (inst) {
                inst.on('click', function(params: any) {
                    if (params.dataType === 'node') {
                        dx.send({
                            type: 'node-click',
                            nodeId: params.data.name,
                            sourceLine: params.data.value ? params.data.value.sourceLine : null
                        });
                    }
                });
            }
        }
    },
    update(id: string, data: any) {
        // Handle two update types:
        if (data.type === 'node-status') {
            LxCharts.updateFlowGraphStatus(id, data.nodeId, data.status);
        } else if (data.type === 'full-update') {
            LxCharts.initFlowGraph(id, data.graphData);
        }
    },
    resize(id: string) {
        var el = document.getElementById(id);
        if (el && typeof echarts !== 'undefined') {
            var inst = echarts.getInstanceByDom(el);
            if (inst) inst.resize();
        }
    },
    dispose(id: string) {
        LxCharts.disposeChart(id);
    }
};

registerWidget('flow-graph', flowGraphWidget);
```

Edit `crates/lx-desktop/assets/index.ts` — add `import "./widgets/flow-graph";` to register the widget.

Edit `crates/lx-desktop/src/app.rs` — add `document::Script { src: asset!("/assets/js/flow_graph.js") }` alongside the other JS script loads.

Run `just ts-build` then `just diagnose`.

**ActiveForm:** Creating flow graph widget in TypeScript

---

### Task 9: Create FlowGraphView component with live execution state

**Subject:** Create FlowGraphView Dioxus component, FlowGraph pane type, EventBus subscription

**Description:** Edit `crates/lx-ui/src/pane_tree/mod.rs`. Add a `FlowGraph` variant:
```rust
FlowGraph {
    id: String,
    source_path: String,
}
```
Update all match arms for PaneNode to include FlowGraph.

Edit `crates/lx-desktop/src/terminal/view.rs`. Add `FlowGraphView` component:

```rust
#[component]
pub fn FlowGraphView(graph_id: String, source_path: String) -> Element {
    let (element_id, widget) = use_ts_widget("flow-graph", serde_json::json!({}));
    let source = source_path.clone();

    // Parse .lx file and extract graph on mount
    use_future(move || {
        let source = source.clone();
        let element_id = element_id.clone();
        async move {
            // Step 1: Read file
            let src = match std::fs::read_to_string(&source) {
                Ok(s) => s,
                Err(e) => {
                    // TODO: show error in pane
                    tracing::error!("flow graph: read error: {e}");
                    return;
                }
            };
            // Step 2: Lex
            let tokens = match lx::lexer::lex(&src) {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("flow graph: lex error: {e}");
                    return;
                }
            };
            // Step 3: Parse
            let program = match lx::parser::parse(tokens) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("flow graph: parse error: {e}");
                    return;
                }
            };
            // Step 4: Walk AST to extract graph
            let mut walker = lx::stdlib::diag_walk::Walker::new();
            lx::visitor::AstVisitor::visit_program(&mut walker, &program);
            let graph = walker.into_graph();
            // Step 5: Convert to ECharts graph JSON via diag.to_graph_chart
            let graph_json = lx::stdlib::diag::graph_to_echart_json(&graph);
            // Step 6: Send to widget
            widget.send_update(serde_json::json!({
                "type": "full-update",
                "graphData": serde_json::from_str::<serde_json::Value>(&graph_json).unwrap_or_default()
            }));

            // Step 7: Listen for click events from the widget
            loop {
                match widget.recv::<serde_json::Value>().await {
                    Ok(msg) => {
                        if msg["type"].as_str() == Some("node-click") {
                            let source_line = msg["sourceLine"].as_u64();
                            if let Some(line) = source_line {
                                // TODO: Create/focus an Editor pane at this line
                                tracing::info!("flow graph: clicked node at line {line}");
                            }
                        }
                    }
                    Err(_) => break,
                }
            }
        }
    });

    // Subscribe to EventBus for live execution state
    // (Only active when a program is running)
    // TODO: Get EventBus from context, subscribe, map RuntimeEvent to node status updates
    // For each relevant event, call widget.send_update(json!({ "type": "node-status", "nodeId": ..., "status": ... }))

    rsx! {
        div {
            id: "{element_id}",
            class: "w-full h-full bg-gray-950",
        }
    }
}
```

**Note on public API**: `Walker`, `AstVisitor`, `Graph`, `graph_to_echart_json` must be pub-accessible from the `lx` crate. `Walker` is currently `pub(crate)` in `diag_walk.rs`. Make it `pub` and re-export from `lx::stdlib::diag` (or create a dedicated public module). `graph_to_echart_json` is the new function from task 7 that converts a Graph to ECharts JSON — make it `pub`.

Update `crates/lx-desktop/src/pages/terminals.rs` to handle FlowGraph pane rendering by calling `FlowGraphView`.

Run `just diagnose`.

**ActiveForm:** Creating FlowGraphView component with live execution state

---

### Task 10: Wire flow graph into Run page and tab system

**Subject:** Add flow graph toggle to Run page, tab bar dropdown, and toolbar support

**Description:** Edit `crates/lx-desktop/src/pages/run.rs`. Add a "Flow Graph" button next to the existing "Run" button. When clicked:
1. Read the current file path from the text input.
2. Create a new `TerminalTab` with a `FlowGraph` pane: `PaneNode::FlowGraph { id: Uuid::new_v4().to_string(), source_path: file_path.clone() }`.
3. Add the tab via `tabs_state.write().add_tab(tab)`.
4. Navigate to the Terminals page (or switch to the new tab if already there).

Edit `crates/lx-desktop/src/terminal/tab_bar.rs`. In the "new pane" dropdown menu (wherever pane type options are listed — Terminal, Browser, Editor, Agent, Canvas), add "Flow Graph" as an option. When selected:
1. Prompt for a `.lx` file path (simple text input or reuse the file path from the Run page).
2. Create a `FlowGraph` pane with the entered path.
3. Split the active pane or create a new tab with the FlowGraph pane.

Edit `crates/lx-desktop/src/terminal/toolbar.rs`. Ensure the per-pane toolbar handles FlowGraph panes: split and close operations should work. The toolbar can show the source file name as the pane title.

Run `just diagnose`. Run `just ts-build` to verify all TypeScript compiles. Run `just desktop` to verify the app launches without errors.

**ActiveForm:** Wiring flow graph into Run page and tab system

---

## CRITICAL REMINDERS — READ BEFORE EVERY TASK

Re-read before starting each task:

1. **Call `complete_task` after each task.** The MCP handles formatting, committing, and diagnostics automatically.
2. **Call `next_task` to get the next task.** Do not look ahead in the task list.
3. **Do not add tasks, skip tasks, reorder tasks, or combine tasks.** Execute the task list exactly as written.
4. **Tasks are implementation-only.** No commit, verify, format, or cleanup tasks — the MCP handles these.

---

## Task Loading Instructions

To execute this work item, load it with the workflow MCP:

```
mcp__workflow__load_work_item({ path: "work_items/DESKTOP_CHARTING.md" })
```

Then call `next_task` to begin.
