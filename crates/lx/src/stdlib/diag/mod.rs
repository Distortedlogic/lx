use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::span::Span;
use crate::value::LxVal;

use crate::ast::Program;
use crate::visitor::AstVisitor;

use super::diag_walk::{DiagEdge, DiagNode, Graph, Subgraph, Walker};

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("extract".into(), mk("diag.extract", 1, bi_extract));
  m.insert("extract_file".into(), mk("diag.extract_file", 1, bi_extract_file));
  m.insert("to_mermaid".into(), mk("diag.to_mermaid", 1, bi_to_mermaid));
  m.insert("to_graph_chart".into(), mk("diag.to_graph_chart", 1, bi_to_graph_chart));
  m
}

fn bi_extract(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let src = args[0].require_str("diag.extract", span)?;
  let graph = extract_graph(src, span)?;
  Ok(graph_to_value(&graph))
}

fn bi_extract_file(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("diag.extract_file", span)?;
  let src = std::fs::read_to_string(path).map_err(|e| LxError::runtime(format!("diag.extract_file: {e}"), span))?;
  let graph = extract_graph(&src, span)?;
  Ok(graph_to_value(&graph))
}

fn bi_to_mermaid(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let graph = value_to_graph(&args[0], span)?;
  Ok(LxVal::str(to_mermaid(&graph).as_str()))
}

fn bi_to_graph_chart(args: &[LxVal], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let graph = value_to_graph(&args[0], span)?;
  Ok(LxVal::str(graph_to_echart_json(&graph).as_str()))
}

pub fn extract_mermaid(program: &Program) -> String {
  let mut walker = Walker::new();
  walker.visit_program(program);
  to_mermaid(&walker.into_graph())
}

pub fn extract_echart_json(program: &Program) -> String {
  let mut walker = Walker::new();
  walker.visit_program(program);
  let graph = walker.into_graph();
  graph_to_echart_json(&graph)
}

pub fn graph_to_echart_json(graph: &Graph) -> String {
  let mut in_degree: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
  let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
  for node in &graph.nodes {
    in_degree.entry(&node.id).or_insert(0);
  }
  for edge in &graph.edges {
    adj.entry(edge.from.as_str()).or_default().push(&edge.to);
    *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
  }
  let mut layers: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
  let mut queue: std::collections::VecDeque<&str> = in_degree.iter().filter(|(_, d)| **d == 0).map(|(id, _)| *id).collect();
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
  let mut layer_positions: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
  let kind_to_category = |k: &str| -> usize {
    match k {
      "agent" => 0,
      "tool" => 1,
      "decision" => 2,
      "fork" | "join" => 3,
      "loop" => 4,
      "resource" => 5,
      "user" => 6,
      "io" => 7,
      "type" => 8,
      _ => 9,
    }
  };
  let kind_to_symbol = |k: &str| -> &str {
    match k {
      "agent" => "roundRect",
      "tool" => "circle",
      "decision" => "diamond",
      "fork" | "join" => "rect",
      "loop" => "roundRect",
      "resource" => "circle",
      "user" => "triangle",
      "io" => "arrow",
      "type" => "circle",
      _ => "circle",
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
      let symbol_size = if matches!(n.kind.as_str(), "fork" | "join") { serde_json::json!([40, 10]) } else { serde_json::json!(40) };
      serde_json::json!({
          "name": n.id,
          "x": layer as f64 * 200.0,
          "y": y_pos as f64 * 120.0,
          "symbol": kind_to_symbol(&n.kind),
          "symbolSize": symbol_size,
          "category": kind_to_category(&n.kind),
          "label": {"show": true, "formatter": n.label, "position": "bottom", "fontSize": 11},
          "value": {"kind": n.kind, "label": n.label, "sourceOffset": n.source_offset}
      })
    })
    .collect();
  let edges_json: Vec<serde_json::Value> = graph
    .edges
    .iter()
    .map(|e| {
      let (line_type, color, width) = match e.edge_type.as_str() {
        "agent" => ("solid", "#f97316", 2),
        "stream" => ("dashed", "#06b6d4", 2),
        "data" => ("solid", "#9ca3af", 1),
        "io" => ("solid", "#00695c", 1),
        _ => ("solid", "#666666", 1),
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

fn extract_graph(src: &str, span: Span) -> Result<Graph, LxError> {
  let tokens = crate::lexer::lex(src).map_err(|e| LxError::runtime(format!("diag: lex error: {e}"), span))?;
  let program = crate::parser::parse(tokens).map_err(|e| LxError::runtime(format!("diag: parse error: {e}"), span))?;
  let mut walker = Walker::new();
  walker.visit_program(&program);
  Ok(walker.into_graph())
}

fn node_to_value(node: &DiagNode) -> LxVal {
  let children: Vec<LxVal> = node.children.iter().map(node_to_value).collect();
  let offset_val = match node.source_offset {
    Some(o) => LxVal::int(o),
    None => LxVal::None,
  };
  record! {
      "id" => LxVal::str(node.id.as_str()),
      "label" => LxVal::str(node.label.as_str()),
      "kind" => LxVal::str(node.kind.as_str()),
      "children" => LxVal::list(children),
      "source_offset" => offset_val,
  }
}

fn edge_to_value(edge: &DiagEdge) -> LxVal {
  record! {
      "from" => LxVal::str(edge.from.as_str()),
      "to" => LxVal::str(edge.to.as_str()),
      "label" => LxVal::str(edge.label.as_str()),
      "style" => LxVal::str(edge.style.as_str()),
      "edge_type" => LxVal::str(edge.edge_type.as_str()),
  }
}

fn graph_to_value(graph: &Graph) -> LxVal {
  let nodes: Vec<LxVal> = graph.nodes.iter().map(node_to_value).collect();
  let edges: Vec<LxVal> = graph.edges.iter().map(edge_to_value).collect();
  record! {
      "nodes" => LxVal::list(nodes),
      "edges" => LxVal::list(edges),
  }
}

fn value_to_graph(val: &LxVal, span: Span) -> Result<Graph, LxError> {
  let LxVal::Record(rec) = val else {
    return Err(LxError::type_err("diag.to_mermaid expects Graph record", span));
  };
  let nodes_val = rec.get("nodes").ok_or_else(|| LxError::type_err("graph missing 'nodes'", span))?;
  let edges_val = rec.get("edges").ok_or_else(|| LxError::type_err("graph missing 'edges'", span))?;
  let nodes = match nodes_val {
    LxVal::List(l) => l.iter().map(|v| value_to_node(v, span)).collect::<Result<_, _>>()?,
    _ => return Err(LxError::type_err("graph.nodes must be List", span)),
  };
  let edges = match edges_val {
    LxVal::List(l) => l.iter().map(|v| value_to_edge(v, span)).collect::<Result<_, _>>()?,
    _ => return Err(LxError::type_err("graph.edges must be List", span)),
  };
  Ok(Graph { nodes, edges, subgraphs: vec![] })
}

fn value_to_node(val: &LxVal, span: Span) -> Result<DiagNode, LxError> {
  let LxVal::Record(rec) = val else {
    return Err(LxError::type_err("node must be Record", span));
  };
  let str_field = |k: &str| -> Result<String, LxError> {
    rec.get(k).and_then(|v| v.as_str()).map(String::from).ok_or_else(|| LxError::type_err(format!("node missing '{k}'"), span))
  };
  let children = match rec.get("children") {
    Some(LxVal::List(l)) => l.iter().map(|v| value_to_node(v, span)).collect::<Result<_, _>>()?,
    _ => vec![],
  };
  let source_offset = rec.get("source_offset").and_then(|v| {
    if let LxVal::Int(i) = v {
      use num_traits::ToPrimitive;
      i.to_u32()
    } else {
      None
    }
  });
  Ok(DiagNode { id: str_field("id")?, label: str_field("label")?, kind: str_field("kind")?, source_offset, children })
}

fn value_to_edge(val: &LxVal, span: Span) -> Result<DiagEdge, LxError> {
  let LxVal::Record(rec) = val else {
    return Err(LxError::type_err("edge must be Record", span));
  };
  let str_field = |k: &str| -> Result<String, LxError> {
    rec.get(k).and_then(|v| v.as_str()).map(String::from).ok_or_else(|| LxError::type_err(format!("edge missing '{k}'"), span))
  };
  let edge_type = rec.get("edge_type").and_then(|v| v.as_str()).map(String::from).unwrap_or_else(|| "exec".into());
  Ok(DiagEdge { from: str_field("from")?, to: str_field("to")?, label: str_field("label")?, style: str_field("style")?, edge_type })
}

fn node_shape(node: &DiagNode, indent: &str) -> String {
  match node.kind.as_str() {
    "agent" => format!("{indent}{}[\"{}\"]", node.id, node.label),
    "tool" => format!("{indent}{}([\"{}\"])", node.id, node.label),
    "decision" => format!("{indent}{}{{\"{}\"}}", node.id, node.label),
    "fork" | "join" => format!("{indent}{}[[\"{}\"]]", node.id, node.label),
    "loop" => format!("{indent}{}{{{{\"{}\"}}}}", node.id, node.label),
    "resource" => format!("{indent}{}[(\"{}\")]", node.id, node.label),
    "user" => format!("{indent}{}[/\"{}\"\\]", node.id, node.label),
    "io" => format!("{indent}{}>\"{}\"]", node.id, node.label),
    "type" => format!("{indent}{}((\"{}\"))", node.id, node.label),
    _ => format!("{indent}{}[\"{}\"]", node.id, node.label),
  }
}

fn to_mermaid(graph: &Graph) -> String {
  let mut out = String::from("flowchart TD\n");
  let sg_ids: std::collections::HashSet<&str> = graph.subgraphs.iter().flat_map(|sg| sg.node_ids.iter().map(|s| s.as_str())).collect();
  for sg in &graph.subgraphs {
    emit_subgraph(&mut out, sg, &graph.nodes);
  }
  for node in &graph.nodes {
    if !sg_ids.contains(node.id.as_str()) {
      out.push_str(&node_shape(node, "    "));
      out.push('\n');
    }
  }
  for edge in &graph.edges {
    let arrow = match edge.style.as_str() {
      "dashed" => "-.->",
      "double" => "==>",
      _ => "-->",
    };
    if edge.label.is_empty() {
      out.push_str(&format!("    {} {} {}\n", edge.from, arrow, edge.to));
    } else {
      out.push_str(&format!("    {} {}|\"{}\"| {}\n", edge.from, arrow, edge.label, edge.to));
    }
  }
  out.push_str("    classDef agent fill:#e1f5fe,stroke:#0288d1\n");
  out.push_str("    classDef tool fill:#f3e5f5,stroke:#7b1fa2\n");
  out.push_str("    classDef decision fill:#fff3e0,stroke:#ef6c00\n");
  out.push_str("    classDef loop fill:#e8f5e9,stroke:#388e3c\n");
  out.push_str("    classDef resource fill:#fce4ec,stroke:#c62828\n");
  out.push_str("    classDef user fill:#ede7f6,stroke:#4527a0\n");
  out.push_str("    classDef io fill:#e0f2f1,stroke:#00695c\n");
  out.push_str("    classDef type fill:#f5f5f5,stroke:#616161\n");
  for node in &graph.nodes {
    let class = match node.kind.as_str() {
      "agent" | "tool" | "decision" | "loop" | "resource" | "user" | "io" | "type" => Some(node.kind.as_str()),
      "fork" | "join" => Some("agent"),
      _ => None,
    };
    if let Some(c) = class {
      out.push_str(&format!("    class {} {c}\n", node.id));
    }
  }
  out
}

fn emit_subgraph(out: &mut String, sg: &Subgraph, nodes: &[DiagNode]) {
  out.push_str(&format!("    subgraph sg_{} [\"{}\"]\n", sg.label, sg.label));
  for nid in &sg.node_ids {
    if let Some(node) = nodes.iter().find(|n| n.id == *nid) {
      out.push_str(&node_shape(node, "        "));
      out.push('\n');
    }
  }
  out.push_str("    end\n");
}
