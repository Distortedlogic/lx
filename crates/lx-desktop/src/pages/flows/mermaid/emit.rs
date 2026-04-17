use std::collections::BTreeMap;

use super::types::{MermaidChart, MermaidNodeMetadata, MermaidSemanticKind};

pub fn emit_chart(chart: &MermaidChart) -> String {
  let mut lines = vec![format!("flowchart {}", chart.direction.keyword())];
  lines.push(format!("%% lx-flow: {}", serde_json::to_string(&serde_json::json!({ "title": chart.title, "notes": chart.notes })).expect("flow metadata should serialize")));
  emit_subgraphs(chart, None, &mut lines);
  emit_root_nodes(chart, &mut lines);
  for edge in &chart.edges {
    let left = emit_node_ref(chart, &edge.from);
    let right = emit_node_ref(chart, &edge.to);
    match edge.label.as_deref().filter(|label| !label.trim().is_empty()) {
      Some(label) => lines.push(format!("    {left} -->|{}| {right}", escape_label(label))),
      None => lines.push(format!("    {left} --> {right}")),
    }
  }
  for class_name in [
    MermaidSemanticKind::Step,
    MermaidSemanticKind::Agent,
    MermaidSemanticKind::Decision,
    MermaidSemanticKind::Tool,
    MermaidSemanticKind::Io,
  ] {
    lines.push(format!("    classDef {} {}", class_name.class_name(), class_style(class_name)));
  }
  for (class_name, node_ids) in class_applications(chart) {
    lines.push(format!("    class {} {};", node_ids.join(","), class_name));
  }
  for node in &chart.nodes {
    if has_node_metadata(&node.metadata) {
      lines.push(format!("    %% lx-node: {} {}", node.id, serde_json::to_string(&node.metadata).expect("node metadata should serialize")));
    }
  }
  lines.join("\n") + "\n"
}

fn emit_subgraphs(chart: &MermaidChart, parent_id: Option<&str>, lines: &mut Vec<String>) {
  let mut subgraphs: Vec<_> = chart.subgraphs.iter().filter(|subgraph| subgraph.parent_id.as_deref() == parent_id).collect();
  subgraphs.sort_by(|left, right| left.id.cmp(&right.id));
  for subgraph in subgraphs {
    lines.push(format!("    subgraph {}[\"{}\"]", subgraph.id, escape_label(&subgraph.title)));
    emit_subgraphs(chart, Some(&subgraph.id), lines);
    let mut nodes: Vec<_> = chart.nodes.iter().filter(|node| node.subgraph_id.as_deref() == Some(subgraph.id.as_str())).collect();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    for node in nodes {
      lines.push(format!("        {}", emit_node_ref(chart, &node.id)));
    }
    lines.push("    end".to_string());
  }
}

fn emit_root_nodes(chart: &MermaidChart, lines: &mut Vec<String>) {
  let mut nodes: Vec<_> = chart.nodes.iter().filter(|node| node.subgraph_id.is_none()).collect();
  nodes.sort_by(|left, right| left.id.cmp(&right.id));
  for node in nodes {
    lines.push(format!("    {}", emit_node_ref(chart, &node.id)));
  }
}

fn emit_node_ref(chart: &MermaidChart, node_id: &str) -> String {
  let node = chart.node(node_id).expect("chart node should exist");
  let label = escape_label(&node.display_label);
  match node.semantic_kind {
    MermaidSemanticKind::Step => format!("{}[\"{}\"]", node.id, label),
    MermaidSemanticKind::Agent => format!("{}([\"{}\"])", node.id, label),
    MermaidSemanticKind::Decision => format!("{}{{\"{}\"}}", node.id, label),
    MermaidSemanticKind::Tool => format!("{}[[\"{}\"]]", node.id, label),
    MermaidSemanticKind::Io => format!("{}[/\"{}\"/]", node.id, label),
  }
}

fn class_applications(chart: &MermaidChart) -> BTreeMap<&'static str, Vec<String>> {
  let mut applications = BTreeMap::<&'static str, Vec<String>>::new();
  for node in &chart.nodes {
    applications.entry(node.semantic_kind.class_name()).or_default().push(node.id.clone());
  }
  applications
}

fn class_style(kind: MermaidSemanticKind) -> &'static str {
  match kind {
    MermaidSemanticKind::Step => "fill:#dbeafe,stroke:#2563eb,color:#0f172a",
    MermaidSemanticKind::Agent => "fill:#e0f2fe,stroke:#0284c7,color:#0f172a",
    MermaidSemanticKind::Decision => "fill:#ffedd5,stroke:#ea580c,color:#0f172a",
    MermaidSemanticKind::Tool => "fill:#f3e8ff,stroke:#7c3aed,color:#0f172a",
    MermaidSemanticKind::Io => "fill:#dcfce7,stroke:#16a34a,color:#0f172a",
  }
}

fn has_node_metadata(metadata: &MermaidNodeMetadata) -> bool {
  metadata.task_summary.as_ref().is_some_and(|value| !value.trim().is_empty()) || metadata.prompt.as_ref().is_some_and(|value| !value.trim().is_empty())
}

fn escape_label(label: &str) -> String {
  label.replace('"', "\\\"")
}
