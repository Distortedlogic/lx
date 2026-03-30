# WU-06: Org chart edge labels

## Fixes
- Fix 17: Change `connected_to: Vec<String>` to `Vec<(String, String)>` to support edge labels. Render SVG text labels on edges.

## Dependencies
WU-05 MUST run first. All file paths and line references assume the post-WU-05 state.

## Files Modified
- `crates/lx-desktop/src/pages/routines/types.rs` (~41 lines post-WU-05)
- `crates/lx-desktop/src/pages/org/chart_helpers.rs` (~75 lines post-WU-05, contains `nodes_from_events`, `build_children_map`, `status_dot_color`)
- `crates/lx-desktop/src/pages/org/chart.rs` (~240 lines post-WU-05)
- `crates/lx-desktop/src/pages/org/chart_layout.rs` (~129 lines post-WU-05)

## Preconditions
- `OrgNode.connected_to` is `Vec<String>` at `types.rs`.
- `connected_to` is populated in `chart_helpers.rs` inside `nodes_from_events`: `node.connected_to.push(to_id);`.
- `connected_to` is checked in `chart_helpers.rs`: `!node.connected_to.contains(&to_id)`.
- The edges rendered in `chart.rs` only draw tree edges (parent-child from `reports_to`), NOT `connected_to` lateral edges. The `connected_to` edges are not yet rendered -- this WU adds that rendering.
- `chart_layout.rs` `LayoutNode` does not carry `connected_to` data. The flat node list has `x`, `y` positions needed for drawing lateral edges.

## Steps

### Step 1: Change connected_to type in OrgNode
- Open `crates/lx-desktop/src/pages/routines/types.rs`
- Find (post-WU-05 state, with icon field present):
```rust
  #[serde(default)]
  pub connected_to: Vec<String>,
  #[serde(default)]
  pub icon: Option<String>,
```
- Replace with:
```rust
  #[serde(default)]
  pub connected_to: Vec<(String, String)>,
  #[serde(default)]
  pub icon: Option<String>,
```
- Why: Each entry is now `(target_node_id, label)`. The label describes the relationship (e.g., "tell", "ask", "delegates").

### Step 2: Update nodes_from_events to populate labeled edges
- Open `crates/lx-desktop/src/pages/org/chart_helpers.rs` (post-WU-05 split file)
- In `nodes_from_events`, find the message-handling match arm:
```rust
      k if k == "tell" || k == "ask" || k.contains("message") => {
        let parts: Vec<&str> = event.message.splitn(2, "->").collect();
        if parts.len() == 2 {
          let from_name = parts[0].trim();
          let to_name = parts[1].trim();
          let from_id = from_name.to_lowercase().replace(' ', "-");
          let to_id = to_name.to_lowercase().replace(' ', "-");
          if let Some(node) = nodes_map.get_mut(&from_id)
            && !node.connected_to.contains(&to_id)
          {
            node.connected_to.push(to_id);
          }
        }
      },
```
- Replace with:
```rust
      k if k == "tell" || k == "ask" || k.contains("message") => {
        let parts: Vec<&str> = event.message.splitn(2, "->").collect();
        if parts.len() == 2 {
          let from_name = parts[0].trim();
          let to_name = parts[1].trim();
          let from_id = from_name.to_lowercase().replace(' ', "-");
          let to_id = to_name.to_lowercase().replace(' ', "-");
          let label = k.to_string();
          if let Some(node) = nodes_map.get_mut(&from_id)
            && !node.connected_to.iter().any(|(id, _)| id == &to_id)
          {
            node.connected_to.push((to_id, label));
          }
        }
      },
```
- The `connected_to: Vec::new()` in the OrgNode constructor is fine -- `Vec::new()` works for both `Vec<String>` and `Vec<(String, String)>`.

### Step 3: Add connected_to to LayoutNode
- Open `crates/lx-desktop/src/pages/org/chart_layout.rs`
- In the `LayoutNode` struct (post-WU-05 state, with `icon` field), find:
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
  pub connected_to: Vec<(String, String)>,
}
```
- In `layout_tree` where `LayoutNode` is constructed (post-WU-05 state, with `icon` field), find:
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
    connected_to: node.connected_to.clone(),
  }
```

### Step 4: Add a function to collect lateral edges with positions
- In `chart_layout.rs`, add after `compute_bounding_box` (after line 127):
```rust
pub fn collect_lateral_edges<'a>(flat: &[&'a LayoutNode]) -> Vec<(&'a LayoutNode, &'a LayoutNode, &'a str)> {
  let positions: HashMap<&str, &LayoutNode> = flat.iter().map(|n| (n.id.as_str(), *n)).collect();
  let mut edges = Vec::new();
  for node in flat {
    for (target_id, label) in &node.connected_to {
      if let Some(target) = positions.get(target_id.as_str()) {
        edges.push((*node, *target, label.as_str()));
      }
    }
  }
  edges
}
```
- Add `use std::collections::HashMap;` at the top of the file if not already present (it is already imported at line 1).
- Why: This collects all lateral (non-tree) edges with their labels and resolved positions, ready for SVG rendering.

### Step 5: Render lateral edges with labels in chart.rs
- Open `crates/lx-desktop/src/pages/org/chart.rs` (post-WU-05, ~240 lines, with helpers split out)
- Add `collect_lateral_edges` to the imports. Find the import line (post-WU-05 state):
```rust
use super::chart_helpers::{build_children_map, nodes_from_events, status_dot_color};
use super::chart_layout::{CARD_H, CARD_W, collect_edges, compute_bounding_box, flatten_layout, layout_forest};
```
- Replace the `chart_layout` import with:
```rust
use super::chart_helpers::{build_children_map, nodes_from_events, status_dot_color};
use super::chart_layout::{CARD_H, CARD_W, collect_edges, collect_lateral_edges, compute_bounding_box, flatten_layout, layout_forest};
```
- After `let edges = collect_edges(&layout);`, add:
```rust
  let lateral_edges = collect_lateral_edges(&flat);
```
- In the SVG rendering section, find the tree-edge loop and the `g` element that contains it:
```rust
        g { transform: "translate({px}, {py}) scale({z})",
          for (parent , child) in edges.iter() {
            {
                ...
            }
          }
        }
```
- Insert the lateral edge `for` loop after the `for (parent, child) in edges.iter() { ... }` block closes and before the `g` element's closing brace `}`:

```rust
          for (from, to, label) in lateral_edges.iter() {
            {
                let x1 = from.x + CARD_W / 2.0;
                let y1 = from.y + CARD_H / 2.0;
                let x2 = to.x + CARD_W / 2.0;
                let y2 = to.y + CARD_H / 2.0;
                let mid_x = (x1 + x2) / 2.0;
                let mid_y = (y1 + y2) / 2.0;
                let d = format!("M {x1} {y1} L {x2} {y2}");
                rsx! {
                  path {
                    key: "lateral-{from.id}-{to.id}",
                    d: "{d}",
                    fill: "none",
                    stroke: "var(--outline)",
                    stroke_width: "1",
                    stroke_dasharray: "4 3",
                  }
                  text {
                    x: "{mid_x}",
                    y: "{mid_y}",
                    text_anchor: "middle",
                    dy: "-4",
                    fill: "var(--outline)",
                    font_size: "10",
                    "{label}"
                  }
                }
            }
          }
```
- Why: Lateral edges are rendered as dashed lines (distinct from solid tree edges) connecting node centers. The label text sits at the midpoint of the line.

## File Size Check
- `types.rs`: ~41 lines (type change is same line count, under 300)
- `chart_layout.rs`: ~142 lines (under 300)
- `chart_helpers.rs`: ~75 lines (under 300)
- `chart.rs`: ~265 lines (post-WU-05 split + lateral edges, under 300)

## Verification
- Run `just diagnose` to confirm no compilation errors.
- Create test data where one agent sends a "tell" message to another (event kind "tell", message "AgentA -> AgentB"). Confirm:
  1. A dashed line appears between AgentA and AgentB's cards.
  2. The label "tell" appears at the midpoint of the dashed line.
  3. Tree edges (from `reports_to`) remain as solid lines.
  4. Multiple lateral edges from the same node render correctly without overlapping labels.
