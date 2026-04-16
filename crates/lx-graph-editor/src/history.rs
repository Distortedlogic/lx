use std::collections::{HashMap, HashSet};

use uuid::Uuid;

use crate::commands::GraphCommand;
use crate::model::{GraphDocument, GraphEdge, GraphEntityRef, GraphNode, GraphPoint, GraphSelection};

#[derive(Clone, Debug, PartialEq)]
pub enum GraphEditorAction {
  Undo,
  Redo,
  CopySelection,
  PasteClipboard,
  DuplicateSelection,
  SelectAll,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GraphHistoryState {
  undo_stack: Vec<GraphDocument>,
  redo_stack: Vec<GraphDocument>,
  clipboard: Option<GraphClipboard>,
  paste_serial: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct GraphClipboard {
  nodes: Vec<GraphNode>,
  edges: Vec<GraphEdge>,
}

impl GraphHistoryState {
  pub fn clear(&mut self) {
    self.undo_stack.clear();
    self.redo_stack.clear();
    self.clipboard = None;
    self.paste_serial = 0;
  }

  pub fn can_undo(&self) -> bool {
    !self.undo_stack.is_empty()
  }

  pub fn can_redo(&self) -> bool {
    !self.redo_stack.is_empty()
  }

  pub fn can_paste(&self) -> bool {
    self.clipboard.as_ref().is_some_and(|clipboard| !clipboard.nodes.is_empty())
  }

  pub fn record_command(&mut self, before: &GraphDocument, command: &GraphCommand) {
    if should_record_command(command) {
      self.record_snapshot_change(before);
    }
  }

  pub fn record_snapshot_change(&mut self, before: &GraphDocument) {
    self.undo_stack.push(before.clone());
    self.redo_stack.clear();
  }

  pub fn undo(&mut self, current: &GraphDocument) -> Option<GraphDocument> {
    let previous = self.undo_stack.pop()?;
    self.redo_stack.push(current.clone());
    Some(previous)
  }

  pub fn redo(&mut self, current: &GraphDocument) -> Option<GraphDocument> {
    let next = self.redo_stack.pop()?;
    self.undo_stack.push(current.clone());
    Some(next)
  }

  pub fn copy_selection(&mut self, document: &GraphDocument) -> bool {
    let selected_node_ids: HashSet<_> = document.selection.node_ids.iter().cloned().collect();
    if selected_node_ids.is_empty() {
      return false;
    }

    let nodes = document.nodes.iter().filter(|node| selected_node_ids.contains(&node.id)).cloned().collect::<Vec<_>>();
    let edges = document
      .edges
      .iter()
      .filter(|edge| selected_node_ids.contains(&edge.from.node_id) && selected_node_ids.contains(&edge.to.node_id))
      .cloned()
      .collect::<Vec<_>>();

    self.clipboard = Some(GraphClipboard { nodes, edges });
    self.paste_serial = 0;
    true
  }

  pub fn duplicate_selection(&mut self, document: &GraphDocument) -> Option<GraphDocument> {
    if !self.copy_selection(document) {
      return None;
    }
    self.paste_clipboard(document)
  }

  pub fn paste_clipboard(&mut self, document: &GraphDocument) -> Option<GraphDocument> {
    let clipboard = self.clipboard.clone()?;
    if clipboard.nodes.is_empty() {
      return None;
    }

    self.paste_serial += 1;
    let offset = GraphPoint { x: 32.0 * self.paste_serial as f64, y: 24.0 * self.paste_serial as f64 };
    let mut next = document.clone();
    let mut node_id_map = HashMap::<String, String>::new();
    let mut selected_node_ids = Vec::new();
    let mut selected_edge_ids = Vec::new();

    for node in clipboard.nodes {
      let next_id = next_node_id(&next, &node.id);
      node_id_map.insert(node.id.clone(), next_id.clone());
      selected_node_ids.push(next_id.clone());
      next.nodes.push(GraphNode {
        id: next_id,
        template_id: node.template_id,
        label: node.label,
        metadata: node.metadata,
        position: GraphPoint { x: node.position.x + offset.x, y: node.position.y + offset.y },
        properties: node.properties,
      });
    }

    for edge in clipboard.edges {
      let Some(from_node_id) = node_id_map.get(&edge.from.node_id).cloned() else {
        continue;
      };
      let Some(to_node_id) = node_id_map.get(&edge.to.node_id).cloned() else {
        continue;
      };
      let next_edge_id = format!("edge-{}", Uuid::new_v4());
      selected_edge_ids.push(next_edge_id.clone());
      next.edges.push(GraphEdge {
        id: next_edge_id,
        label: edge.label,
        metadata: edge.metadata,
        from: crate::model::GraphPortRef { node_id: from_node_id, port_id: edge.from.port_id },
        to: crate::model::GraphPortRef { node_id: to_node_id, port_id: edge.to.port_id },
      });
    }

    next.selection = selection_from_ids(selected_node_ids, selected_edge_ids);
    Some(next)
  }

  pub fn select_all(&self, document: &GraphDocument) -> GraphSelection {
    let node_ids = document.nodes.iter().map(|node| node.id.clone()).collect::<Vec<_>>();
    let edge_ids = document.edges.iter().map(|edge| edge.id.clone()).collect::<Vec<_>>();
    selection_from_ids(node_ids, edge_ids)
  }
}

fn should_record_command(command: &GraphCommand) -> bool {
  !matches!(command, GraphCommand::Select { .. } | GraphCommand::SetViewport { .. })
}

fn selection_from_ids(node_ids: Vec<String>, edge_ids: Vec<String>) -> GraphSelection {
  let anchor = node_ids.first().cloned().map(GraphEntityRef::Node).or_else(|| edge_ids.first().cloned().map(GraphEntityRef::Edge));
  GraphSelection { anchor, node_ids, edge_ids }
}

fn next_node_id(document: &GraphDocument, base_id: &str) -> String {
  let base = format!("{base_id}-copy");
  let mut candidate = base.clone();
  let mut index = 2usize;
  while document.node(&candidate).is_some() {
    candidate = format!("{base}-{index}");
    index += 1;
  }
  candidate
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::GraphHistoryState;
  use crate::commands::GraphCommand;
  use crate::model::{GraphDocument, GraphEdge, GraphNode, GraphPoint, GraphPortRef, GraphSelection};

  fn sample_document() -> GraphDocument {
    GraphDocument {
      id: "graph".to_string(),
      title: "Graph".to_string(),
      metadata: Default::default(),
      viewport: Default::default(),
      selection: GraphSelection::single_node("node-a"),
      nodes: vec![
        GraphNode {
          id: "node-a".to_string(),
          template_id: "source".to_string(),
          label: Some("A".to_string()),
          metadata: Default::default(),
          position: GraphPoint { x: 10.0, y: 20.0 },
          properties: json!({}),
        },
        GraphNode {
          id: "node-b".to_string(),
          template_id: "sink".to_string(),
          label: Some("B".to_string()),
          metadata: Default::default(),
          position: GraphPoint { x: 120.0, y: 24.0 },
          properties: json!({}),
        },
      ],
      edges: vec![GraphEdge {
        id: "edge-a-b".to_string(),
        label: None,
        metadata: Default::default(),
        from: GraphPortRef { node_id: "node-a".to_string(), port_id: "out".to_string() },
        to: GraphPortRef { node_id: "node-b".to_string(), port_id: "in".to_string() },
      }],
    }
  }

  #[test]
  fn duplicate_selection_creates_offset_copy() {
    let mut history = GraphHistoryState::default();
    let mut document = sample_document();
    document.selection = GraphSelection {
      anchor: Some(crate::model::GraphEntityRef::Node("node-a".to_string())),
      node_ids: vec!["node-a".to_string(), "node-b".to_string()],
      edge_ids: vec![],
    };

    let next = history.duplicate_selection(&document).expect("duplicate selection");

    assert_eq!(next.nodes.len(), 4);
    assert_eq!(next.edges.len(), 2);
    assert!(next.node("node-a-copy").is_some());
    let copied = next.node("node-a-copy").expect("copied node");
    assert_eq!(copied.position, GraphPoint { x: 42.0, y: 44.0 });
  }

  #[test]
  fn undo_and_redo_round_trip_document() {
    let mut history = GraphHistoryState::default();
    let before = sample_document();
    let mut after = before.clone();
    after.selection = GraphSelection::single_node("node-b");

    history.record_snapshot_change(&before);
    let undone = history.undo(&after).expect("undo snapshot");
    assert_eq!(undone, before);

    let redone = history.redo(&before).expect("redo snapshot");
    assert_eq!(redone, after);
  }

  #[test]
  fn select_all_includes_nodes_and_edges() {
    let history = GraphHistoryState::default();
    let document = sample_document();

    let selection = history.select_all(&document);
    assert_eq!(selection.node_ids.len(), 2);
    assert_eq!(selection.edge_ids.len(), 1);
  }

  #[test]
  fn select_and_viewport_commands_do_not_record_history() {
    let mut history = GraphHistoryState::default();
    let document = sample_document();

    history.record_command(&document, &GraphCommand::Select { selection: GraphSelection::empty() });
    history.record_command(&document, &GraphCommand::SetViewport { viewport: Default::default() });

    assert!(!history.can_undo());
  }
}
