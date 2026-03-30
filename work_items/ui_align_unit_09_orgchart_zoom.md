# Unit 09: OrgChart wheel zoom + auto-center

## Goal

Add mouse-wheel zoom toward cursor, auto-center/fit on mount, and hover states to OrgChart cards.

## Preconditions

- No other unit dependencies
- CSS variables `--surface-container`, `--outline-variant`, `--on-surface`, `--outline` exist in `tailwind.css`

## Files to Modify

- `crates/lx-desktop/src/pages/org/chart.rs`
- `crates/lx-desktop/src/pages/org/chart_layout.rs`

## Steps

### 1. Add bounding-box computation to chart_layout.rs

In `crates/lx-desktop/src/pages/org/chart_layout.rs`, add a public struct and function after the existing `collect_edges` function:

```rust
pub struct BoundingBox {
  pub min_x: f64,
  pub min_y: f64,
  pub max_x: f64,
  pub max_y: f64,
}

pub fn compute_bounding_box(nodes: &[&LayoutNode]) -> Option<BoundingBox> {
  if nodes.is_empty() {
    return None;
  }
  let mut min_x = f64::MAX;
  let mut min_y = f64::MAX;
  let mut max_x = f64::MIN;
  let mut max_y = f64::MIN;
  for n in nodes {
    min_x = min_x.min(n.x);
    min_y = min_y.min(n.y);
    max_x = max_x.max(n.x + CARD_W);
    max_y = max_y.max(n.y + CARD_H);
  }
  Some(BoundingBox { min_x, min_y, max_x, max_y })
}
```

### 2. Update chart.rs imports

In `crates/lx-desktop/src/pages/org/chart.rs`, change the import line from:

```rust
use super::chart_layout::{CARD_H, CARD_W, collect_edges, flatten_layout, layout_forest};
```

to:

```rust
use super::chart_layout::{CARD_H, CARD_W, collect_edges, compute_bounding_box, flatten_layout, layout_forest};
```

### 3. Add onwheel handler to the container div

In `crates/lx-desktop/src/pages/org/chart.rs`, inside the `OrgChart` component, add an `onwheel` handler to the outer container `div` (the one with class `"w-full flex-1 min-h-0 overflow-hidden relative"`). Place it after the `onmouseleave` handler.

The handler must zoom toward the mouse cursor position. The math: when zooming, the point under the cursor should remain stationary. Given the current transform `translate(pan_x, pan_y) scale(zoom)`, a world-space point `wx` maps to screen-space `sx = wx * zoom + pan_x`. To keep `sx` constant when zoom changes from `old_z` to `new_z`: `new_pan_x = sx - wx * new_z = pan_x + (old_z - new_z) * wx` where `wx = (sx - pan_x) / old_z`.

```rust
onwheel: move |evt| {
    let old_z = zoom();
    let delta = evt.delta();
    let dy = match delta {
        dioxus::prelude::WheelDelta::Pixels(p) => p.y,
        dioxus::prelude::WheelDelta::Lines(l) => l.y * 40.0,
        dioxus::prelude::WheelDelta::Pages(p) => p.y * 400.0,
    };
    let factor = if dy < 0.0 { 1.1 } else { 1.0 / 1.1 };
    let new_z = (old_z * factor).clamp(0.1, 3.0);
    let coords = evt.client_coordinates();
    let mouse_x = coords.x;
    let mouse_y = coords.y;
    let world_x = (mouse_x - pan_x()) / old_z;
    let world_y = (mouse_y - pan_y()) / old_z;
    pan_x.set(mouse_x - world_x * new_z);
    pan_y.set(mouse_y - world_y * new_z);
    zoom.set(new_z);
},
```

Note: `evt.client_coordinates()` returns coordinates relative to the viewport. This is correct because the container starts at the viewport edge (it is `absolute inset-0` inside the page). If the container has an offset, the math still works acceptably for this use case.

### 4. Implement auto-center (fit-to-viewport) on mount

Replace the existing "Fit" button `onclick` handler and add a `use_effect` that runs once on mount to auto-center. The logic:

1. Compute the bounding box of all laid-out nodes
2. Calculate the scale needed to fit the bounding box into a viewport of assumed size 800x600 (we cannot easily read the container size in Dioxus without JS interop, so use a reasonable default)
3. Set pan to center the bounding box

In the `OrgChart` component, **after** the line `let edges = collect_edges(&layout);` and **before** the signal declarations, add:

`flat` is already defined before `edges` in the current code (line 90 before line 91). No reorder needed. Add after `flat`:

```rust
let bbox = compute_bounding_box(&flat);
```

Add a `use_effect` right after the signal declarations that runs on mount to auto-fit. Use a `mut mounted` signal to ensure it runs only once:

```rust
let mut mounted = use_signal(|| false);
```

Add this after all other signal declarations:

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
            let fit_z = scale_x.min(scale_y).min(1.5).max(0.2);
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

### 5. Update "Fit" button to use the same auto-center logic

Replace the "Fit" button's `onclick` handler (lines 162-166 in the current file) with:

```rust
onclick: move |_| {
    if let Some(ref bb) = bbox {
        let content_w = bb.max_x - bb.min_x;
        let content_h = bb.max_y - bb.min_y;
        let vw = 800.0_f64;
        let vh = 600.0_f64;
        let margin = 40.0;
        let scale_x = (vw - margin * 2.0) / content_w;
        let scale_y = (vh - margin * 2.0) / content_h;
        let fit_z = scale_x.min(scale_y).min(1.5).max(0.2);
        let cx = bb.min_x + content_w / 2.0;
        let cy = bb.min_y + content_h / 2.0;
        pan_x.set(vw / 2.0 - cx * fit_z);
        pan_y.set(vh / 2.0 - cy * fit_z);
        zoom.set(fit_z);
    } else {
        zoom.set(1.0);
        pan_x.set(0.0);
        pan_y.set(0.0);
    }
},
```

### 6. Add hover states to node cards

In `crates/lx-desktop/src/pages/org/chart.rs`, find the card `div` for each node (the one with class `"absolute bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg shadow-sm select-none"`). Change its class to:

```
"absolute bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg shadow-sm select-none transition-shadow hover:shadow-md hover:border-[var(--on-surface)]/20"
```

This adds:
- `transition-shadow` for smooth shadow transition
- `hover:shadow-md` to elevate on hover
- `hover:border-[var(--on-surface)]/20` to subtly brighten the border

### 7. Verify file stays under 300 lines

After all changes, `chart.rs` should be approximately 260-275 lines. `chart_layout.rs` should be approximately 120 lines. Both are under the 300 line limit.

If `chart.rs` exceeds 300 lines, extract the `fit_to_viewport` logic into a helper function at file scope:

```rust
fn fit_to_viewport(bb: &BoundingBox) -> (f64, f64, f64) {
    let content_w = bb.max_x - bb.min_x;
    let content_h = bb.max_y - bb.min_y;
    let vw = 800.0_f64;
    let vh = 600.0_f64;
    let margin = 40.0;
    let scale_x = (vw - margin * 2.0) / content_w;
    let scale_y = (vh - margin * 2.0) / content_h;
    let fit_z = scale_x.min(scale_y).min(1.5).max(0.2);
    let cx = bb.min_x + content_w / 2.0;
    let cy = bb.min_y + content_h / 2.0;
    (vw / 2.0 - cx * fit_z, vh / 2.0 - cy * fit_z, fit_z)
}
```

Then the `use_effect`, Fit button, and mount logic each call `fit_to_viewport(bb)` and destructure the tuple.

## Verification

1. Run `just diagnose` -- must compile with no warnings and no clippy errors.
2. Open the desktop app and navigate to the Org Chart page.
3. With agents displayed, verify:
   - Mouse wheel scrolling zooms in/out and the point under the cursor stays stable.
   - On page load, the chart auto-centers to fit all visible nodes.
   - Clicking the "Fit" button re-centers the chart.
   - Hovering over a node card shows an elevated shadow and subtly brighter border.
   - The +/- zoom buttons still work.
   - Pan by click-drag still works.
4. With no agents displayed, verify the empty state message still renders correctly.
5. Both modified files are under 300 lines.
