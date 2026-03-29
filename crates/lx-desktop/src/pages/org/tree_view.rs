use std::collections::HashMap;

use dioxus::prelude::*;

use crate::pages::routines::types::OrgNode;

#[component]
pub fn OrgTreeView(nodes: Vec<OrgNode>) -> Element {
  let children_map = build_children_map(&nodes);
  let roots: Vec<&OrgNode> = nodes.iter().filter(|n| n.reports_to.is_none()).collect();

  rsx! {
    div { class: "border border-[var(--outline-variant)]/30 rounded-lg py-1",
      for root in roots {
        OrgTreeNode {
          key: "{root.id}",
          node: root.clone(),
          children_map: children_map.clone(),
          all_nodes: nodes.clone(),
          depth: 0,
        }
      }
    }
  }
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

fn status_dot_class(status: &str) -> &'static str {
  match status {
    "active" => "bg-green-400",
    "paused" => "bg-yellow-400",
    "error" => "bg-red-400",
    _ => "bg-neutral-400",
  }
}

#[component]
fn OrgTreeNode(node: OrgNode, children_map: HashMap<String, Vec<OrgNode>>, all_nodes: Vec<OrgNode>, depth: u32) -> Element {
  let mut expanded = use_signal(|| true);
  let children = children_map.get(&node.id).cloned().unwrap_or_default();
  let has_children = !children.is_empty();
  let pad = depth * 16 + 12;
  let dot_cls = status_dot_class(&node.status);

  rsx! {
    div {
      div {
        class: "flex items-center gap-2 px-3 py-2 text-sm transition-colors hover:bg-white/5",
        style: "padding-left: {pad}px",
        if has_children {
          button {
            class: "p-0.5",
            onclick: move |_| expanded.set(!expanded()),
            span {
              class: format!(
                  "material-symbols-outlined text-xs transition-transform {}",
                  if expanded() { "rotate-90" } else { "" },
              ),
              "chevron_right"
            }
          }
        } else {
          span { class: "w-4" }
        }
        span { class: "h-2 w-2 rounded-full shrink-0 {dot_cls}" }
        span { class: "font-medium text-[var(--on-surface)] flex-1", "{node.name}" }
        span { class: "text-xs text-[var(--outline)]", "{node.role}" }
      }
      if has_children && expanded() {
        for child in children.iter() {
          OrgTreeNode {
            key: "{child.id}",
            node: child.clone(),
            children_map: children_map.clone(),
            all_nodes: all_nodes.clone(),
            depth: depth + 1,
          }
        }
      }
    }
  }
}
