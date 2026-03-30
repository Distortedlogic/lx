# WU-05: Org chart dynamic viewport and card enhancements

## Fixes
- Fix 7: Replace hardcoded 800x600 viewport dimensions with actual container dimensions measured via JS interop.
- Fix 16: Add an `icon` field to `OrgNode` and render it on org chart cards.

## Files Modified
- `crates/lx-desktop/src/pages/org/chart.rs` (287 lines)
- `crates/lx-desktop/src/pages/org/chart_layout.rs` (127 lines)
- `crates/lx-desktop/src/pages/routines/types.rs` (39 lines)

## Preconditions
- `chart.rs` has hardcoded `800.0_f64` and `600.0_f64` at lines 110-111 (initial mount) and lines 209-210 (fit button handler).
- `OrgNode` struct at `types.rs` lines 19-28 has fields: `id`, `name`, `role`, `status`, `reports_to`, `connected_to`. No `icon` field.
- `chart.rs` creates `OrgNode` at line 19 in `nodes_from_events`. Card rendering at lines 258-281.
- `chart_layout.rs` `LayoutNode` struct at lines 11-19 has `id`, `name`, `role`, `status`, `x`, `y`, `children`. No `icon` field.
- `chart.rs` is currently 287 lines — adding the viewport measurement and icon rendering will push it over 300 lines.

## Steps

### Step 1: Add icon field to OrgNode
- Open `crates/lx-desktop/src/pages/routines/types.rs`
- At lines 19-28, find:
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub reports_to: Option<String>,
  #[serde(default)]
  pub connected_to: Vec<String>,
}
```
- Replace with:
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub reports_to: Option<String>,
  #[serde(default)]
  pub connected_to: Vec<String>,
  #[serde(default)]
  pub icon: Option<String>,
}
```
- Why: Optional icon field allows nodes to display a Material Symbols icon. `serde(default)` keeps backward compatibility with serialized data.

### Step 2: Add icon field to LayoutNode in chart_layout.rs
- Open `crates/lx-desktop/src/pages/org/chart_layout.rs`
- At lines 11-19, find:
```rust
pub struct LayoutNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub x: f64,
  pub y: f64,
  pub children: Vec<LayoutNode>,
}
```
- Replace with:
```rust
pub struct LayoutNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub icon: Option<String>,
  pub x: f64,
  pub y: f64,
  pub children: Vec<LayoutNode>,
}
```
- Why: LayoutNode mirrors OrgNode fields for rendering; it needs to carry the icon through the layout pass.

### Step 3: Pass icon through layout_tree
- In `chart_layout.rs`, at lines 50-58, find:
```rust
  LayoutNode {
    id: node.id.clone(),
    name: node.name.clone(),
    role: node.role.clone(),
    status: node.status.clone(),
    x: x + (total_w - CARD_W) / 2.0,
    y,
    children: layout_children,
  }
```
- Replace with:
```rust
  LayoutNode {
    id: node.id.clone(),
    name: node.name.clone(),
    role: node.role.clone(),
    status: node.status.clone(),
    icon: node.icon.clone(),
    x: x + (total_w - CARD_W) / 2.0,
    y,
    children: layout_children,
  }
```
- Why: Forward the icon from OrgNode to LayoutNode during layout computation.

### Step 4: Set default icon in nodes_from_events
- Open `crates/lx-desktop/src/pages/org/chart.rs`
- At lines 19-25, find:
```rust
        nodes_map.entry(id.clone()).or_insert_with(|| OrgNode {
          id,
          name,
          role: "Agent".into(),
          status: if event.kind == "agent_running" { "running".into() } else { "active".into() },
          reports_to: None,
          connected_to: Vec::new(),
        });
```
- Replace with:
```rust
        nodes_map.entry(id.clone()).or_insert_with(|| OrgNode {
          id,
          name,
          role: "Agent".into(),
          status: if event.kind == "agent_running" { "running".into() } else { "active".into() },
          reports_to: None,
          connected_to: Vec::new(),
          icon: None,
        });
```
- Why: Initialize the new field. `None` means the card will show the status dot only (no icon).

### Step 5: Add container size measurement via onmounted
- At line 95 (after `let mut pan_y`), find:
```rust
  let mut pan_y = use_signal(|| 0.0f64);
  let mut zoom = use_signal(|| 1.0f64);
```
- After line 97 (`let mut zoom = ...`), add:
```rust
  let mut container_w = use_signal(|| 800.0f64);
  let mut container_h = use_signal(|| 600.0f64);
```
- Find the container div at line 144:
```rust
    div {
      class: "w-full flex-1 min-h-0 overflow-hidden relative",
      style: "cursor: {cursor}",
```
- Add an `onmounted` handler:
```rust
    div {
      class: "w-full flex-1 min-h-0 overflow-hidden relative",
      style: "cursor: {cursor}",
      onmounted: move |evt: MountedEvent| {
          spawn(async move {
              if let Ok(rect) = evt.data().get_client_rect().await {
                  let w = rect.width();
                  let h = rect.height();
                  if w > 0.0 && h > 0.0 {
                      container_w.set(w);
                      container_h.set(h);
                  }
              }
          });
      },
```
- Why: `onmounted` fires once when the div enters the DOM, providing the actual rendered size. This replaces the need for hardcoded 800x600.

### Step 6: Replace all hardcoded 800.0/600.0 with container_w/container_h
- Replace the entire initial fit `use_effect` block (lines 105-124):
```rust
  use_effect(move || {
    if !mounted() {
      if let Some(ref bb) = bbox {
        let content_w = bb.max_x - bb.min_x;
        let content_h = bb.max_y - bb.min_y;
        let vw = 800.0_f64;
        let vh = 600.0_f64;
        let margin = 40.0;
        let scale_x = (vw - margin * 2.0) / content_w;
        let scale_y = (vh - margin * 2.0) / content_h;
        let fit_z = scale_x.min(scale_y).clamp(0.2, 1.5);
        let cx = bb.min_x + content_w / 2.0;
        let cy = bb.min_y + content_h / 2.0;
        pan_x.set(vw / 2.0 - cx * fit_z);
        pan_y.set(vh / 2.0 - cy * fit_z);
        zoom.set(fit_z);
      }
      mounted.set(true);
    }
  });
```
- With:
```rust
  use_effect(move || {
    let cw = container_w();
    let ch = container_h();
    if !mounted() && cw > 0.0 && ch > 0.0 {
      if let Some(ref bb) = bbox {
        let content_w = bb.max_x - bb.min_x;
        let content_h = bb.max_y - bb.min_y;
        let margin = 40.0;
        let scale_x = (cw - margin * 2.0) / content_w;
        let scale_y = (ch - margin * 2.0) / content_h;
        let fit_z = scale_x.min(scale_y).clamp(0.2, 1.5);
        let cx = bb.min_x + content_w / 2.0;
        let cy = bb.min_y + content_h / 2.0;
        pan_x.set(cw / 2.0 - cx * fit_z);
        pan_y.set(ch / 2.0 - cy * fit_z);
        zoom.set(fit_z);
      }
      mounted.set(true);
    }
  });
```
- This effect now reads `container_w()` and `container_h()`, making it reactive to container size changes. The `cw > 0.0 && ch > 0.0` guard ensures it waits until `onmounted` has measured the actual container.
- In the "Fit" button handler (lines 206-224), find:
```rust
                  let vw = 800.0_f64;
                  let vh = 600.0_f64;
```
- Replace with:
```rust
                  let vw = container_w();
                  let vh = container_h();
```

### Step 7: Render icon on cards
- In the card rendering section (lines 268-278), find:
```rust
                  div { class: "flex items-center px-4 py-3 gap-3",
                    span {
                      class: "h-3 w-3 rounded-full shrink-0",
                      style: "background-color: {dot_color}",
                    }
                    div { class: "flex flex-col min-w-0 flex-1",
                      span { class: "text-sm font-semibold text-[var(--on-surface)] leading-tight",
                        "{node.name}"
                      }
                      span { class: "text-[11px] text-[var(--outline)] leading-tight mt-0.5", "{node.role}" }
                    }
                  }
```
- Replace with:
```rust
                  div { class: "flex items-center px-4 py-3 gap-3",
                    if let Some(ref icon) = node.icon {
                      span {
                        class: "material-symbols-outlined text-base shrink-0 text-[var(--on-surface-variant)]",
                        "{icon}"
                      }
                    } else {
                      span {
                        class: "h-3 w-3 rounded-full shrink-0",
                        style: "background-color: {dot_color}",
                      }
                    }
                    div { class: "flex flex-col min-w-0 flex-1",
                      span { class: "text-sm font-semibold text-[var(--on-surface)] leading-tight",
                        "{node.name}"
                      }
                      span { class: "text-[11px] text-[var(--outline)] leading-tight mt-0.5", "{node.role}" }
                    }
                  }
```
- Why: If the node has an icon, render it as a Material Symbol. Otherwise fall back to the status dot.

### Step 8: Split chart.rs to stay under 300 lines
- After all edits, `chart.rs` will be approximately 310-320 lines. Split the helper functions into a separate file.
- Create `crates/lx-desktop/src/pages/org/chart_helpers.rs` containing:
  - `nodes_from_events` (function, lines 10-59 of chart.rs)
  - `build_children_map` (function, lines 61-69 of chart.rs)
  - `status_dot_color` (function, lines 71-80 of chart.rs)
  - Required imports:
    ```rust
    use std::collections::HashMap;
    use crate::contexts::activity_log::ActivityLog;
    use crate::pages::routines::types::OrgNode;
    ```
    `ActivityLog` is needed because `nodes_from_events` takes `&ActivityLog` and reads `log.events`. `OrgNode` is the return type and is constructed inside both `nodes_from_events` and `build_children_map`. `HashMap` is used by both `nodes_from_events` (for `nodes_map`) and `build_children_map` (for the return type).
- In `chart.rs`, remove lines 1-80 (everything before `#[component]`) and replace with:
```rust
use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

use super::chart_helpers::{build_children_map, nodes_from_events, status_dot_color};
use super::chart_layout::{CARD_H, CARD_W, collect_edges, compute_bounding_box, flatten_layout, layout_forest};
use crate::contexts::activity_log::ActivityLog;
```
- In `crates/lx-desktop/src/pages/org/mod.rs`, add:
```rust
mod chart_helpers;
```
- Why: The 300-line file limit requires splitting. Helper functions are a natural extraction.

## File Size Check
- `chart.rs`: was 287 lines, after edits ~320 lines, after split ~240 lines (under 300)
- `chart_helpers.rs`: new file, ~75 lines (under 300)
- `chart_layout.rs`: was 127 lines, now ~129 lines (under 300)
- `types.rs`: was 39 lines, now ~41 lines (under 300)

## Verification
- Run `just diagnose` to confirm no compilation errors.
- Open the org chart page. Resize the window/panel and confirm the chart fits to the actual container dimensions on first load, not a fixed 800x600 box.
- Click the "Fit" button and confirm it re-fits to the current container size.
- Add a node with `icon: Some("smart_toy".into())` in test data and confirm the Material Symbol icon renders on the card instead of the status dot.
