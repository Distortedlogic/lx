use std::collections::HashSet;

use super::diag_walk::{DiagNode, EdgeStyle, Graph, NodeKind, Subgraph};

fn node_shape(node: &DiagNode, indent: &str) -> String {
  match node.kind {
    NodeKind::Agent => format!("{indent}{}[\"{}\"]", node.id, node.label),
    NodeKind::Tool => format!("{indent}{}([\"{}\"])", node.id, node.label),
    NodeKind::Decision => format!("{indent}{}{{\"{}\"}}", node.id, node.label),
    NodeKind::Fork | NodeKind::Join => format!("{indent}{}[[\"{}\"]]", node.id, node.label),
    NodeKind::Loop => format!("{indent}{}{{{{\"{}\"}}}}", node.id, node.label),
    NodeKind::Resource => format!("{indent}{}[(\"{}\")]", node.id, node.label),
    NodeKind::User => format!("{indent}{}[/\"{}\"\\]", node.id, node.label),
    NodeKind::Io => format!("{indent}{}>\"{}\"]", node.id, node.label),
    NodeKind::Type => format!("{indent}{}((\"{}\"))", node.id, node.label),
  }
}

pub(crate) fn to_mermaid(graph: &Graph) -> String {
  let mut out = String::from("flowchart TD\n");
  let sg_ids: HashSet<&str> = graph.subgraphs.iter().flat_map(|sg| sg.node_ids.iter().map(|s| s.as_str())).collect();
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
    let arrow = match edge.style {
      EdgeStyle::Dashed => "-.->",
      EdgeStyle::Double => "==>",
      EdgeStyle::Solid => "-->",
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
    let class = match node.kind {
      NodeKind::Agent | NodeKind::Tool | NodeKind::Decision | NodeKind::Loop | NodeKind::Resource | NodeKind::User | NodeKind::Io | NodeKind::Type => {
        Some(node.kind.as_str())
      },
      NodeKind::Fork | NodeKind::Join => Some("agent"),
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
