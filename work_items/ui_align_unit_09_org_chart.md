# UNIT 9: Wire OrgChart to ActivityLog Event Data

## Goal

Replace the hardcoded `default_org_nodes()` and `dioxus_storage::use_persistent` in `chart.rs` with real agent topology derived from `ActivityLog` events. Agent spawn events provide node data; communication events provide edges.

## Files Modified

| File | Action |
|------|--------|
| `crates/lx-desktop/src/pages/org/chart.rs` | Replace hardcoded nodes with ActivityLog-derived data |
| `crates/lx-desktop/src/pages/routines/types.rs` | Add `connected_to` field to OrgNode |

## Reference Files (read-only)

| File | Why |
|------|-----|
| `crates/lx-desktop/src/pages/org/chart_layout.rs` | `LayoutNode`, `layout_forest`, `flatten_layout`, `collect_edges`, constants |
| `crates/lx-desktop/src/contexts/activity_log.rs` | ActivityLog context, `ActivityEvent { timestamp, kind, message }` |
| `crates/lx-api/src/types.rs` | ActivityEvent definition |

---

## Current State

### `chart.rs` lines 8-15 (hardcoded nodes)
```rust
fn default_org_nodes() -> Vec<OrgNode> {
  vec![
    OrgNode { id: "ceo-1".into(), name: "Atlas".into(), role: "CEO".into(), status: "active".into(), reports_to: None },
    OrgNode { id: "eng-1".into(), name: "Nova".into(), role: "Engineering Lead".into(), status: "active".into(), reports_to: Some("ceo-1".into()) },
    OrgNode { id: "ops-1".into(), name: "Orbit".into(), role: "Operations".into(), status: "paused".into(), reports_to: Some("ceo-1".into()) },
    OrgNode { id: "dev-1".into(), name: "Spark".into(), role: "Developer".into(), status: "active".into(), reports_to: Some("eng-1".into()) },
  ]
}
```

### `chart.rs` line 40 (use_persistent)
```rust
  let nodes = dioxus_storage::use_persistent("lx_org_nodes", default_org_nodes);
```

### `types.rs` lines 19-26 (OrgNode struct)
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub reports_to: Option<String>,
}
```

---

## Step 1: Add `connected_to` field to OrgNode

In `crates/lx-desktop/src/pages/routines/types.rs`:

Old text (lines 19-26):
```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OrgNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub reports_to: Option<String>,
}
```

New text:
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

The `#[serde(default)]` ensures backward compatibility with any serialized data that lacks this field.

---

## Step 2: Update all OrgNode construction sites

The only place that constructs `OrgNode` values inline is `chart.rs` `default_org_nodes()`. That function will be deleted in Step 3. No other files construct `OrgNode` literals.

Verify by searching for `OrgNode {` across the codebase. The only hit is `chart.rs` lines 10-14.

---

## Step 3: Rewrite `chart.rs`

Replace the entire file `crates/lx-desktop/src/pages/org/chart.rs` with:

```rust
use std::collections::HashMap;

use dioxus::prelude::*;

use super::chart_layout::{CARD_H, CARD_W, collect_edges, flatten_layout, layout_forest};
use crate::contexts::activity_log::ActivityLog;
use crate::pages::routines::types::OrgNode;

fn nodes_from_events(log: &ActivityLog) -> Vec<OrgNode> {
  let events = log.events.read();
  let mut nodes_map: HashMap<String, OrgNode> = HashMap::new();

  for event in events.iter() {
    match event.kind.as_str() {
      "agent_start" | "agent_running" | "agent_spawn" => {
        let name = event.message.clone();
        let id = name.to_lowercase().replace(' ', "-");
        nodes_map.entry(id.clone()).or_insert_with(|| OrgNode {
          id,
          name,
          role: "Agent".into(),
          status: if event.kind == "agent_running" { "running".into() } else { "active".into() },
          reports_to: None,
          connected_to: Vec::new(),
        });
      }
      "agent_reports_to" => {
        let parts: Vec<&str> = event.message.splitn(2, "->").collect();
        if parts.len() == 2 {
          let child_name = parts[0].trim();
          let parent_name = parts[1].trim();
          let child_id = child_name.to_lowercase().replace(' ', "-");
          let parent_id = parent_name.to_lowercase().replace(' ', "-");
          if let Some(node) = nodes_map.get_mut(&child_id) {
            node.reports_to = Some(parent_id);
          }
        }
      }
      k if k == "tell" || k == "ask" || k.contains("message") => {
        let parts: Vec<&str> = event.message.splitn(2, "->").collect();
        if parts.len() == 2 {
          let from_name = parts[0].trim();
          let to_name = parts[1].trim();
          let from_id = from_name.to_lowercase().replace(' ', "-");
          let to_id = to_name.to_lowercase().replace(' ', "-");
          if let Some(node) = nodes_map.get_mut(&from_id) {
            if !node.connected_to.contains(&to_id) {
              node.connected_to.push(to_id);
            }
          }
        }
      }
      _ => {}
    }
  }

  nodes_map.into_values().collect()
}

fn build_children_map(nodes: &[OrgNode]) -> HashMap<String, Vec<OrgNode>> {
  let mut map: HashMap<String, Vec<OrgNode>> = HashMap::new();
  for node in nodes {
    if let Some(parent_id) = &node.reports_to {
      map.entry(parent_id.clone()).or_default().push(node.clone());
    }
  }
  map
}

fn status_dot_color(status: &str) -> &'static str {
  match status {
    "running" => "#22d3ee",
    "active" => "#4ade80",
    "paused" | "idle" => "#facc15",
    "error" => "#f87171",
    "terminated" => "#a3a3a3",
    _ => "#a3a3a3",
  }
}

#[component]
pub fn OrgChart() -> Element {
  let log = use_context::<ActivityLog>();
  let all = nodes_from_events(&log);

  let has_nodes = !all.is_empty();
  let children_map = build_children_map(&all);
  let roots: Vec<OrgNode> = all.iter().filter(|n| n.reports_to.is_none()).cloned().collect();
  let layout = layout_forest(&roots, &children_map);
  let flat = flatten_layout(&layout);
  let edges = collect_edges(&layout);

  let mut pan_x = use_signal(|| 0.0f64);
  let mut pan_y = use_signal(|| 0.0f64);
  let mut zoom = use_signal(|| 1.0f64);
  let mut dragging = use_signal(|| false);
  let mut drag_start_x = use_signal(|| 0.0f64);
  let mut drag_start_y = use_signal(|| 0.0f64);
  let mut drag_start_pan_x = use_signal(|| 0.0f64);
  let mut drag_start_pan_y = use_signal(|| 0.0f64);

  let px = pan_x();
  let py = pan_y();
  let z = zoom();
  let is_dragging = dragging();

  let cursor = if is_dragging { "grabbing" } else { "grab" };

  if !has_nodes {
    return rsx! {
      div { class: "flex items-center justify-center h-64",
        p { class: "text-sm text-[var(--outline)]",
          "No agents detected. Run an agent to populate the org chart."
        }
      }
    };
  }

  rsx! {
    div {
      class: "w-full flex-1 min-h-0 overflow-hidden relative",
      style: "cursor: {cursor}",
      onmousedown: move |evt| {
          dragging.set(true);
          let coords = evt.client_coordinates();
          drag_start_x.set(coords.x);
          drag_start_y.set(coords.y);
          drag_start_pan_x.set(pan_x());
          drag_start_pan_y.set(pan_y());
      },
      onmousemove: move |evt| {
          if !dragging() {
              return;
          }
          let coords = evt.client_coordinates();
          let dx = coords.x - drag_start_x();
          let dy = coords.y - drag_start_y();
          pan_x.set(drag_start_pan_x() + dx);
          pan_y.set(drag_start_pan_y() + dy);
      },
      onmouseup: move |_| dragging.set(false),
      onmouseleave: move |_| dragging.set(false),
      div { class: "absolute top-3 right-3 z-10 flex flex-col gap-1",
        button {
          class: "w-7 h-7 flex items-center justify-center bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded text-sm hover:brightness-110 transition-colors",
          onclick: move |_| {
              let new_z = (zoom() * 1.2).min(2.0);
              zoom.set(new_z);
          },
          "+"
        }
        button {
          class: "w-7 h-7 flex items-center justify-center bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded text-sm hover:brightness-110 transition-colors",
          onclick: move |_| {
              let new_z = (zoom() * 0.8).max(0.2);
              zoom.set(new_z);
          },
          "\u{2212}"
        }
        button {
          class: "w-7 h-7 flex items-center justify-center bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded text-[10px] hover:brightness-110 transition-colors",
          onclick: move |_| {
              zoom.set(1.0);
              pan_x.set(0.0);
              pan_y.set(0.0);
          },
          "Fit"
        }
      }
      svg {
        class: "absolute inset-0 pointer-events-none",
        width: "100%",
        height: "100%",
        g { transform: "translate({px}, {py}) scale({z})",
          for (parent , child) in edges.iter() {
            {
                let x1 = parent.x + CARD_W / 2.0;
                let y1 = parent.y + CARD_H;
                let x2 = child.x + CARD_W / 2.0;
                let y2 = child.y;
                let mid_y = (y1 + y2) / 2.0;
                let d = format!("M {x1} {y1} L {x1} {mid_y} L {x2} {mid_y} L {x2} {y2}");
                rsx! {
                  path {
                    key: "{parent.id}-{child.id}",
                    d: "{d}",
                    fill: "none",
                    stroke: "var(--outline-variant)",
                    stroke_width: "1.5",
                  }
                }
            }
          }
        }
      }
      div {
        class: "absolute inset-0",
        style: "transform: translate({px}px, {py}px) scale({z}); transform-origin: 0 0",
        for node in flat.iter() {
          {
              let dot_color = status_dot_color(&node.status);
              let card_w = CARD_W;
              let card_h = CARD_H;
              rsx! {
                div {
                  key: "{node.id}",
                  class: "absolute bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg shadow-sm select-none",
                  style: "left: {node.x}px; top: {node.y}px; width: {card_w}px; min-height: {card_h}px",
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
                }
              }
          }
        }
      }
    }
  }
}
```

**What changed vs. the original `chart.rs`:**

1. Deleted `default_org_nodes()` function entirely.
2. Removed `use dioxus_storage` import. Added `use crate::contexts::activity_log::ActivityLog` import.
3. Added `nodes_from_events(log: &ActivityLog) -> Vec<OrgNode>` function that:
   - Scans all events for `agent_start`, `agent_running`, `agent_spawn` kinds and creates `OrgNode` entries keyed by lowercased-hyphenated name.
   - Scans for `agent_reports_to` events with format `"ChildName->ParentName"` and sets `reports_to`.
   - Scans for `tell`, `ask`, or any `kind` containing `"message"` with format `"FromName->ToName"` and populates `connected_to`.
4. `OrgChart` component: replaced `dioxus_storage::use_persistent("lx_org_nodes", default_org_nodes)` with `use_context::<ActivityLog>()` + `nodes_from_events(&log)`.
5. Added empty state when no nodes are derived from events.
6. All pan/zoom/drag logic and SVG rendering remain identical.

---

## Verification

After all changes:
- `chart.rs` is ~195 lines (under 300).
- `types.rs` grows by 2 lines to ~38 (under 300).
- No code comments or docstrings.
- No `#[allow(...)]` macros.
- `dioxus_storage` is no longer imported in `chart.rs` (the crate is still a dependency for other files).
- Empty ActivityLog shows the "No agents detected" empty state.
- Agent events create nodes; `reports_to` events create hierarchy; `tell`/`ask` events populate `connected_to`.
- The `connected_to` field on OrgNode is populated from `tell`/`ask` events.
