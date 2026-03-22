use std::collections::{HashMap, VecDeque};

use super::diag_walk::{DiagEdge, DiagNode, EdgeType, Graph, NodeKind};

pub(crate) fn graph_to_echart_json(graph: &Graph) -> String {
  let mut in_degree: HashMap<&str, usize> = HashMap::new();
  let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
  for node in &graph.nodes {
    in_degree.entry(&node.id).or_insert(0);
  }
  for edge in &graph.edges {
    adj.entry(edge.from.as_str()).or_default().push(&edge.to);
    *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
  }
  let mut layers: HashMap<&str, usize> = HashMap::new();
  let mut queue: VecDeque<&str> = in_degree.iter().filter(|(_, d)| **d == 0).map(|(id, _)| *id).collect();
  for &id in &queue {
    layers.insert(id, 0);
  }
  while let Some(id) = queue.pop_front() {
    let layer = layers[id];
    if let Some(neighbors) = adj.get(id) {
      for &next in neighbors {
        let next_layer = layers.entry(next).or_insert(0);
        if layer + 1 > *next_layer {
          *next_layer = layer + 1;
        }
        let deg = in_degree.get_mut(next).expect("node in in_degree");
        *deg -= 1;
        if *deg == 0 {
          queue.push_back(next);
        }
      }
    }
  }
  for node in &graph.nodes {
    layers.entry(&node.id).or_insert(0);
  }
  let mut layer_positions: HashMap<usize, usize> = HashMap::new();
  let kind_to_category = |k: NodeKind| -> usize {
    match k {
      NodeKind::Agent => 0,
      NodeKind::Tool => 1,
      NodeKind::Decision => 2,
      NodeKind::Fork | NodeKind::Join => 3,
      NodeKind::Loop => 4,
      NodeKind::Resource => 5,
      NodeKind::User => 6,
      NodeKind::Io => 7,
      NodeKind::Type => 8,
    }
  };
  let kind_to_symbol = |k: NodeKind| -> &str {
    match k {
      NodeKind::Agent => "roundRect",
      NodeKind::Tool => "circle",
      NodeKind::Decision => "diamond",
      NodeKind::Fork | NodeKind::Join => "rect",
      NodeKind::Loop => "roundRect",
      NodeKind::Resource => "circle",
      NodeKind::User => "triangle",
      NodeKind::Io => "arrow",
      NodeKind::Type => "circle",
    }
  };
  let categories = serde_json::json!([
      {"name": "agent", "itemStyle": {"color": "#e1f5fe", "borderColor": "#0288d1"}},
      {"name": "tool", "itemStyle": {"color": "#f3e5f5", "borderColor": "#7b1fa2"}},
      {"name": "decision", "itemStyle": {"color": "#fff3e0", "borderColor": "#ef6c00"}},
      {"name": "fork", "itemStyle": {"color": "#e1f5fe", "borderColor": "#0288d1"}},
      {"name": "loop", "itemStyle": {"color": "#e8f5e9", "borderColor": "#388e3c"}},
      {"name": "resource", "itemStyle": {"color": "#fce4ec", "borderColor": "#c62828"}},
      {"name": "user", "itemStyle": {"color": "#ede7f6", "borderColor": "#4527a0"}},
      {"name": "io", "itemStyle": {"color": "#e0f2f1", "borderColor": "#00695c"}},
      {"name": "type", "itemStyle": {"color": "#f5f5f5", "borderColor": "#616161"}},
      {"name": "default", "itemStyle": {"color": "#f5f5f5", "borderColor": "#616161"}}
  ]);
  let nodes_json: Vec<serde_json::Value> = graph
    .nodes
    .iter()
    .map(|n| {
      let layer = layers.get(n.id.as_str()).copied().unwrap_or(0);
      let pos = layer_positions.entry(layer).or_insert(0);
      let y_pos = *pos;
      *pos += 1;
      let symbol_size = if matches!(n.kind, NodeKind::Fork | NodeKind::Join) { serde_json::json!([40, 10]) } else { serde_json::json!(40) };
      serde_json::json!({
          "name": n.id,
          "x": layer as f64 * 200.0,
          "y": y_pos as f64 * 120.0,
          "symbol": kind_to_symbol(n.kind),
          "symbolSize": symbol_size,
          "category": kind_to_category(n.kind),
          "label": {"show": true, "formatter": n.label, "position": "bottom", "fontSize": 11},
          "value": {"kind": n.kind.as_str(), "label": n.label, "sourceOffset": n.source_offset}
      })
    })
    .collect();
  let edges_json: Vec<serde_json::Value> = graph
    .edges
    .iter()
    .map(|e| {
      let (line_type, color, width) = match e.edge_type {
        EdgeType::Agent => ("solid", "#f97316", 2),
        EdgeType::Stream => ("dashed", "#06b6d4", 2),
        EdgeType::Data => ("solid", "#9ca3af", 1),
        EdgeType::Io => ("solid", "#00695c", 1),
        EdgeType::Exec => ("solid", "#666666", 1),
      };
      let mut edge = serde_json::json!({
          "source": e.from,
          "target": e.to,
          "lineStyle": {"type": line_type, "color": color, "width": width, "curveness": 0.2}
      });
      if !e.label.is_empty() {
        edge["label"] = serde_json::json!({"show": true, "formatter": e.label, "fontSize": 10});
      }
      edge
    })
    .collect();
  serde_json::json!({
      "tooltip": {"trigger": "item"},
      "series": [{
          "type": "graph",
          "layout": "none",
          "roam": true,
          "label": {"show": true},
          "categories": categories,
          "data": nodes_json,
          "links": edges_json,
          "lineStyle": {"curveness": 0.2}
      }]
  })
  .to_string()
}
