use std::collections::BTreeMap;

use serde::Deserialize;

use super::super::types::{MermaidNodeMetadata, MermaidSemanticKind};

#[derive(Clone, Debug, Default, Deserialize)]
struct FlowMetadata {
  title: Option<String>,
  notes: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct ScanResult {
  pub diagnostics: Vec<lx_graph_editor::protocol::GraphWidgetDiagnostic>,
  pub flow_title: Option<String>,
  pub flow_notes: Option<String>,
  pub semantic_classes: BTreeMap<String, MermaidSemanticKind>,
  pub node_metadata: BTreeMap<String, MermaidNodeMetadata>,
}

pub fn scan_source(source: &str) -> ScanResult {
  let mut result = ScanResult::default();
  let mut header_seen = false;
  let mut subgraph_depth = 0usize;

  for (index, line) in source.lines().enumerate() {
    let trimmed = line.trim();
    if trimmed.is_empty() {
      continue;
    }
    if let Some(metadata) = trimmed.strip_prefix("%% lx-flow:") {
      match serde_json::from_str::<FlowMetadata>(metadata.trim()) {
        Ok(flow_metadata) => {
          result.flow_title = flow_metadata.title;
          result.flow_notes = flow_metadata.notes;
        },
        Err(error) => result.diagnostics.push(error_diagnostic(format!("mermaid-flow-meta-{index}"), format!("Invalid flow metadata: {error}"), None)),
      }
      continue;
    }
    if let Some(metadata) = trimmed.strip_prefix("%% lx-node:") {
      let mut parts = metadata.trim().splitn(2, char::is_whitespace);
      let Some(node_id) = parts.next().filter(|value| !value.is_empty()) else {
        result.diagnostics.push(error_diagnostic(format!("mermaid-node-meta-{index}"), "Node metadata must start with a node id.", None));
        continue;
      };
      let Some(json) = parts.next() else {
        result.diagnostics.push(error_diagnostic(
          format!("mermaid-node-meta-json-{index}"),
          format!("Node metadata for `{node_id}` is missing a JSON payload."),
          None,
        ));
        continue;
      };
      match serde_json::from_str::<MermaidNodeMetadata>(json.trim()) {
        Ok(node_metadata) => {
          result.node_metadata.insert(node_id.to_string(), node_metadata);
        },
        Err(error) => {
          result.diagnostics.push(error_diagnostic(format!("mermaid-node-meta-json-{index}"), format!("Invalid node metadata for `{node_id}`: {error}"), None))
        },
      }
      continue;
    }
    if trimmed.starts_with("%%") {
      continue;
    }
    if !header_seen {
      header_seen = true;
      if !matches!(trimmed.to_ascii_lowercase().as_str(), "flowchart td" | "flowchart lr") {
        result.diagnostics.push(error_diagnostic("mermaid-header", "Only `flowchart TD` and `flowchart LR` headers are supported.", None));
      }
      continue;
    }
    if let Some(class_names) = trimmed.strip_prefix("classDef ") {
      for class_name in class_names.split_whitespace().next().unwrap_or_default().split(',').map(str::trim).filter(|value| !value.is_empty()) {
        if MermaidSemanticKind::from_class_name(class_name).is_none() {
          result.diagnostics.push(error_diagnostic(
            format!("mermaid-classdef-{class_name}-{index}"),
            format!("Unsupported Mermaid semantic class `{class_name}`."),
            None,
          ));
        }
      }
      continue;
    }
    if let Some(rest) = trimmed.strip_prefix("class ") {
      let mut parts = rest.splitn(2, char::is_whitespace);
      let node_ids = parts.next().unwrap_or_default();
      let class_name = parts.next().unwrap_or_default().trim();
      let Some(kind) = MermaidSemanticKind::from_class_name(class_name) else {
        result.diagnostics.push(error_diagnostic(format!("mermaid-class-{index}"), format!("Unsupported Mermaid semantic class `{class_name}`."), None));
        continue;
      };
      for node_id in node_ids.split(',').map(str::trim).filter(|value| !value.is_empty()) {
        if let Some(existing) = result.semantic_classes.insert(node_id.to_string(), kind)
          && existing != kind
        {
          result.diagnostics.push(error_diagnostic(
            format!("mermaid-class-duplicate-{node_id}"),
            format!("Node `{node_id}` was assigned multiple semantic classes."),
            None,
          ));
        }
      }
      continue;
    }
    if trimmed.eq("end") {
      if subgraph_depth == 0 {
        result.diagnostics.push(error_diagnostic(format!("mermaid-end-{index}"), "Encountered `end` without an open subgraph.", None));
      } else {
        subgraph_depth -= 1;
      }
      continue;
    }
    if trimmed.starts_with("subgraph ") {
      subgraph_depth += 1;
      continue;
    }
    if is_supported_edge_statement(trimmed) || is_supported_node_statement(trimmed) {
      continue;
    }
    result.diagnostics.push(error_diagnostic(format!("mermaid-statement-{index}"), format!("Unsupported Mermaid statement: `{trimmed}`"), None));
  }

  if !header_seen {
    result.diagnostics.push(error_diagnostic("mermaid-header-missing", "The Mermaid file is missing a `flowchart TD` or `flowchart LR` header.", None));
  }
  if subgraph_depth != 0 {
    result.diagnostics.push(error_diagnostic("mermaid-subgraph-balance", "Subgraph blocks must be closed with `end`.", None));
  }
  result
}

pub fn error_diagnostic(
  id: impl Into<String>,
  message: impl Into<String>,
  target: Option<lx_graph_editor::model::GraphEntityRef>,
) -> lx_graph_editor::protocol::GraphWidgetDiagnostic {
  lx_graph_editor::protocol::GraphWidgetDiagnostic {
    id: id.into(),
    severity: lx_graph_editor::protocol::GraphWidgetDiagnosticSeverity::Error,
    message: message.into(),
    source: Some("mermaid".to_string()),
    detail: None,
    target,
  }
}

pub fn humanize_flow_id(flow_id: &str) -> String {
  flow_id
    .split(['-', '_'])
    .filter(|segment| !segment.is_empty())
    .map(|segment| {
      let mut chars = segment.chars();
      match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}

fn is_supported_edge_statement(line: &str) -> bool {
  let Some((left, right)) = line.split_once("-->") else {
    return false;
  };
  if left.contains('&') || right.contains('&') || left.contains('<') || right.contains('<') || line.matches("-->").count() != 1 {
    return false;
  }
  let right = if let Some(after_arrow) = right.trim().strip_prefix('|') {
    let Some((_, target)) = after_arrow.split_once('|') else {
      return false;
    };
    target.trim()
  } else {
    right.trim()
  };
  parse_endpoint_id(left.trim()).is_some() && parse_endpoint_id(right).is_some()
}

fn is_supported_node_statement(line: &str) -> bool {
  parse_endpoint_id(line).is_some()
}

fn parse_endpoint_id(value: &str) -> Option<&str> {
  let trimmed = value.trim();
  let end = trimmed.find(['[', '{', '(', '/']).unwrap_or(trimmed.len());
  let node_id = trimmed[..end].trim();
  if node_id.is_empty() || !node_id.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.')) {
    return None;
  }
  Some(node_id)
}
