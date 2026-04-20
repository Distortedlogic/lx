use std::collections::HashMap;

use anyhow::{Context, Result, bail};
use lx_graph_editor::model::{
  GraphDocument, GraphDocumentMetadata, GraphEdge, GraphEdgeMetadata, GraphNode, GraphNodeMetadata, GraphPoint, GraphPortRef, GraphSelection, GraphViewport,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct N8nWorkflow {
  pub name: Option<String>,
  #[serde(default)]
  pub nodes: Vec<N8nNode>,
  #[serde(default)]
  pub connections: HashMap<String, N8nConnectionsFromNode>,
}

#[derive(Debug, Deserialize)]
pub struct N8nNode {
  pub name: String,
  pub id: Option<String>,
  #[serde(rename = "type")]
  pub kind: String,
  #[serde(default)]
  pub position: Option<[f64; 2]>,
  #[serde(default)]
  pub parameters: Value,
}

#[derive(Debug, Deserialize)]
pub struct N8nConnectionsFromNode {
  #[serde(default)]
  pub main: Vec<Vec<N8nConnectionTarget>>,
}

#[derive(Debug, Deserialize)]
pub struct N8nConnectionTarget {
  pub node: String,
  #[serde(default)]
  pub index: Option<u32>,
}

pub fn import_n8n_json(payload: &str, target_flow_id: &str) -> Result<GraphDocument> {
  let workflow: N8nWorkflow = serde_json::from_str(payload).context("failed to parse n8n workflow json")?;
  if workflow.nodes.is_empty() {
    bail!("n8n workflow has no nodes");
  }

  let mut document = GraphDocument {
    id: target_flow_id.to_string(),
    title: workflow.name.unwrap_or_else(|| target_flow_id.to_string()),
    metadata: GraphDocumentMetadata::default(),
    viewport: GraphViewport::default(),
    selection: GraphSelection::default(),
    nodes: Vec::new(),
    edges: Vec::new(),
  };

  let mut name_to_id: HashMap<String, String> = HashMap::new();
  for n8n_node in &workflow.nodes {
    let id = n8n_node.id.clone().unwrap_or_else(|| slugify(&n8n_node.name));
    name_to_id.insert(n8n_node.name.clone(), id.clone());
    let template_id = map_n8n_kind(&n8n_node.kind);
    let position = n8n_node.position.map(|[x, y]| GraphPoint { x, y }).unwrap_or(GraphPoint { x: 0.0, y: 0.0 });
    document.nodes.push(GraphNode {
      id,
      template_id,
      label: Some(n8n_node.name.clone()),
      metadata: GraphNodeMetadata::default(),
      position,
      properties: n8n_node.parameters.clone(),
    });
  }

  let mut edge_index = 0usize;
  for (from_name, from_conns) in &workflow.connections {
    let Some(from_id) = name_to_id.get(from_name).cloned() else {
      continue;
    };
    for (output_index, targets) in from_conns.main.iter().enumerate() {
      let from_port = if output_index == 0 { default_output_port(&from_id, &document) } else { format!("out_{output_index}") };
      for target in targets {
        let Some(to_id) = name_to_id.get(&target.node).cloned() else {
          continue;
        };
        let to_port = match target.index.unwrap_or(0) {
          0 => default_input_port(&to_id, &document),
          other => format!("in_{other}"),
        };
        document.edges.push(GraphEdge {
          id: format!("edge-{edge_index}"),
          label: None,
          metadata: GraphEdgeMetadata::default(),
          from: GraphPortRef { node_id: from_id.clone(), port_id: from_port.clone() },
          to: GraphPortRef { node_id: to_id, port_id: to_port },
        });
        edge_index += 1;
      }
    }
  }

  Ok(document)
}

fn map_n8n_kind(kind: &str) -> String {
  let lower = kind.to_lowercase();
  if lower.contains("httprequest") {
    return "http_request".to_string();
  }
  if lower.contains("slack") {
    return "slack_post".to_string();
  }
  if lower.contains("if") {
    return "control_if".to_string();
  }
  if lower.contains("switch") {
    return "control_switch".to_string();
  }
  if lower.contains("merge") {
    return "control_merge".to_string();
  }
  if lower.contains("wait") {
    return "control_wait".to_string();
  }
  if lower.contains("set") {
    return "control_set".to_string();
  }
  if lower.contains("splitinbatches") {
    return "control_split_in_batches".to_string();
  }
  if lower.contains("code") || lower.contains("function") {
    return "control_code".to_string();
  }
  if lower.contains("cron") || lower.contains("schedule") {
    return "trigger_cron".to_string();
  }
  if lower.contains("webhook") {
    return "trigger_webhook".to_string();
  }
  if lower.contains("manualtrigger") || lower.contains("starttrigger") {
    return "trigger_manual".to_string();
  }
  if lower.contains("openai") {
    return "openai_chat".to_string();
  }
  if lower.contains("anthropic") || lower.contains("claude") {
    return "anthropic_messages".to_string();
  }
  if lower.contains("readbinaryfile") || lower.contains("readfile") {
    return "file_read".to_string();
  }
  if lower.contains("writebinaryfile") || lower.contains("writefile") {
    return "file_write".to_string();
  }
  if lower.contains("postgres") {
    return "postgres_query".to_string();
  }
  "http_request".to_string()
}

fn default_output_port(node_id: &str, document: &GraphDocument) -> String {
  for node in &document.nodes {
    if node.id != node_id {
      continue;
    }
    if node.template_id == "control_if" {
      return "true".to_string();
    }
    if node.template_id == "trigger_cron" || node.template_id == "trigger_manual" || node.template_id == "trigger_webhook" {
      return "out".to_string();
    }
    if node.template_id == "http_request" || node.template_id == "anthropic_messages" || node.template_id == "openai_chat" {
      return "response".to_string();
    }
    break;
  }
  "out".to_string()
}

fn default_input_port(node_id: &str, document: &GraphDocument) -> String {
  for node in &document.nodes {
    if node.id == node_id && node.template_id == "control_merge" {
      return "input_a".to_string();
    }
  }
  "input".to_string()
}

fn slugify(name: &str) -> String {
  let slug: String = name.chars().map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' }).collect();
  let trimmed = slug.trim_matches('-').to_string();
  if trimmed.is_empty() { "node".to_string() } else { trimmed }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn imports_trivial_workflow() {
    let payload = serde_json::json!({
      "name": "hello",
      "nodes": [
        { "name": "Manual", "type": "n8n-nodes-base.manualTrigger", "position": [0.0, 0.0] },
        { "name": "HTTP", "type": "n8n-nodes-base.httpRequest", "position": [200.0, 0.0], "parameters": { "url": "https://api.example.com" } }
      ],
      "connections": {
        "Manual": {
          "main": [[ { "node": "HTTP", "index": 0 } ]]
        }
      }
    })
    .to_string();

    let document = import_n8n_json(&payload, "imported").unwrap();
    assert_eq!(document.id, "imported");
    assert_eq!(document.nodes.len(), 2);
    assert!(document.nodes.iter().any(|node| node.template_id == "trigger_manual"));
    assert!(document.nodes.iter().any(|node| node.template_id == "http_request"));
    assert_eq!(document.edges.len(), 1);
  }
}
