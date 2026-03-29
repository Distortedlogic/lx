use std::collections::HashMap;

use crate::pages::routines::types::OrgNode;

pub const CARD_W: f64 = 200.0;
pub const CARD_H: f64 = 80.0;
pub const GAP_X: f64 = 32.0;
pub const GAP_Y: f64 = 80.0;
pub const PADDING: f64 = 60.0;

pub struct LayoutNode {
  pub id: String,
  pub name: String,
  pub role: String,
  pub status: String,
  pub x: f64,
  pub y: f64,
  pub children: Vec<LayoutNode>,
}

pub fn subtree_width(node_id: &str, children_map: &HashMap<String, Vec<OrgNode>>) -> f64 {
  let Some(children) = children_map.get(node_id) else {
    return CARD_W;
  };
  if children.is_empty() {
    return CARD_W;
  }
  let children_w: f64 = children.iter().map(|c| subtree_width(&c.id, children_map)).sum();
  let gaps = (children.len() as f64 - 1.0) * GAP_X;
  CARD_W.max(children_w + gaps)
}

pub fn layout_tree(node: &OrgNode, x: f64, y: f64, children_map: &HashMap<String, Vec<OrgNode>>) -> LayoutNode {
  let total_w = subtree_width(&node.id, children_map);
  let mut layout_children = Vec::new();

  if let Some(children) = children_map.get(&node.id)
    && !children.is_empty()
  {
    let children_w: f64 = children.iter().map(|c| subtree_width(&c.id, children_map)).sum();
    let gaps = (children.len() as f64 - 1.0) * GAP_X;
    let mut cx = x + (total_w - children_w - gaps) / 2.0;
    for child in children {
      let cw = subtree_width(&child.id, children_map);
      layout_children.push(layout_tree(child, cx, y + CARD_H + GAP_Y, children_map));
      cx += cw + GAP_X;
    }
  }

  LayoutNode {
    id: node.id.clone(),
    name: node.name.clone(),
    role: node.role.clone(),
    status: node.status.clone(),
    x: x + (total_w - CARD_W) / 2.0,
    y,
    children: layout_children,
  }
}

pub fn layout_forest(roots: &[OrgNode], children_map: &HashMap<String, Vec<OrgNode>>) -> Vec<LayoutNode> {
  if roots.is_empty() {
    return Vec::new();
  }
  let mut x = PADDING;
  let y = PADDING;
  let mut result = Vec::new();
  for root in roots {
    let w = subtree_width(&root.id, children_map);
    result.push(layout_tree(root, x, y, children_map));
    x += w + GAP_X;
  }
  result
}

pub fn flatten_layout(nodes: &[LayoutNode]) -> Vec<&LayoutNode> {
  let mut result = Vec::new();
  fn walk<'a>(n: &'a LayoutNode, out: &mut Vec<&'a LayoutNode>) {
    out.push(n);
    for child in &n.children {
      walk(child, out);
    }
  }
  for n in nodes {
    walk(n, &mut result);
  }
  result
}

pub fn collect_edges(nodes: &[LayoutNode]) -> Vec<(&LayoutNode, &LayoutNode)> {
  let mut edges = Vec::new();
  fn walk<'a>(n: &'a LayoutNode, out: &mut Vec<(&'a LayoutNode, &'a LayoutNode)>) {
    for child in &n.children {
      out.push((n, child));
      walk(child, out);
    }
  }
  for n in nodes {
    walk(n, &mut edges);
  }
  edges
}
