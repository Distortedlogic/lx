use std::collections::HashMap;

use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;

use super::chart_layout::{CARD_H, CARD_W, collect_edges, compute_bounding_box, flatten_layout, layout_forest};
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
      },
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
      },
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
      _ => {},
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
    "running" => "var(--tertiary)",
    "active" => "var(--success)",
    "paused" | "idle" => "var(--warning)",
    "error" => "var(--error)",
    "terminated" => "var(--outline)",
    _ => "var(--outline)",
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
  let bbox = compute_bounding_box(&flat);

  let mut pan_x = use_signal(|| 0.0f64);
  let mut pan_y = use_signal(|| 0.0f64);
  let mut zoom = use_signal(|| 1.0f64);
  let mut dragging = use_signal(|| false);
  let mut drag_start_x = use_signal(|| 0.0f64);
  let mut drag_start_y = use_signal(|| 0.0f64);
  let mut drag_start_pan_x = use_signal(|| 0.0f64);
  let mut drag_start_pan_y = use_signal(|| 0.0f64);
  let mut mounted = use_signal(|| false);

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
      onwheel: move |evt| {
          let old_z = zoom();
          let delta = evt.delta();
          let dy = match delta {
              WheelDelta::Pixels(p) => p.y,
              WheelDelta::Lines(l) => l.y * 40.0,
              WheelDelta::Pages(p) => p.y * 400.0,
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
              } else {
                  zoom.set(1.0);
                  pan_x.set(0.0);
                  pan_y.set(0.0);
              }
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
                  class: "absolute bg-[var(--surface-container)] border border-[var(--outline-variant)] rounded-lg shadow-sm select-none transition-shadow hover:shadow-md hover:border-[var(--on-surface)]/20",
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
