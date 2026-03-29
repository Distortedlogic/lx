use std::collections::HashMap;

use dioxus::prelude::*;

use super::chart_layout::{CARD_H, CARD_W, collect_edges, flatten_layout, layout_forest};
use crate::pages::routines::types::OrgNode;

fn default_org_nodes() -> Vec<OrgNode> {
  vec![
    OrgNode { id: "ceo-1".into(), name: "Atlas".into(), role: "CEO".into(), status: "active".into(), reports_to: None },
    OrgNode { id: "eng-1".into(), name: "Nova".into(), role: "Engineering Lead".into(), status: "active".into(), reports_to: Some("ceo-1".into()) },
    OrgNode { id: "ops-1".into(), name: "Orbit".into(), role: "Operations".into(), status: "paused".into(), reports_to: Some("ceo-1".into()) },
    OrgNode { id: "dev-1".into(), name: "Spark".into(), role: "Developer".into(), status: "active".into(), reports_to: Some("eng-1".into()) },
  ]
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
  let nodes = dioxus_storage::use_persistent("lx_org_nodes", default_org_nodes);

  let all = nodes();
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
