use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphDocumentMetadata {
  pub notes: Option<String>,
  #[serde(default)]
  pub tags: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphNodeMetadata {
  pub notes: Option<String>,
  #[serde(default)]
  pub tags: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphEdgeMetadata {
  pub notes: Option<String>,
  #[serde(default)]
  pub tags: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct GraphPoint {
  pub x: f64,
  pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphViewport {
  pub pan_x: f64,
  pub pan_y: f64,
  pub zoom: f64,
}

impl Default for GraphViewport {
  fn default() -> Self {
    Self { pan_x: 0.0, pan_y: 0.0, zoom: 1.0 }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum GraphEntityRef {
  Node(String),
  Edge(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphSelection {
  pub anchor: Option<GraphEntityRef>,
  #[serde(default)]
  pub node_ids: Vec<String>,
  #[serde(default)]
  pub edge_ids: Vec<String>,
}

impl GraphSelection {
  pub fn empty() -> Self {
    Self::default()
  }

  pub fn single_node(node_id: impl Into<String>) -> Self {
    let node_id = node_id.into();
    Self { anchor: Some(GraphEntityRef::Node(node_id.clone())), node_ids: vec![node_id], edge_ids: Vec::new() }
  }

  pub fn single_edge(edge_id: impl Into<String>) -> Self {
    let edge_id = edge_id.into();
    Self { anchor: Some(GraphEntityRef::Edge(edge_id.clone())), node_ids: Vec::new(), edge_ids: vec![edge_id] }
  }

  pub fn is_empty(&self) -> bool {
    self.node_ids.is_empty() && self.edge_ids.is_empty()
  }

  pub fn clear(&mut self) {
    self.anchor = None;
    self.node_ids.clear();
    self.edge_ids.clear();
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphPortRef {
  pub node_id: String,
  pub port_id: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphNode {
  pub id: String,
  pub template_id: String,
  pub label: Option<String>,
  #[serde(default)]
  pub metadata: GraphNodeMetadata,
  pub position: GraphPoint,
  #[serde(default = "default_properties")]
  pub properties: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphEdge {
  pub id: String,
  pub label: Option<String>,
  #[serde(default)]
  pub metadata: GraphEdgeMetadata,
  pub from: GraphPortRef,
  pub to: GraphPortRef,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphDocument {
  pub id: String,
  pub title: String,
  #[serde(default)]
  pub metadata: GraphDocumentMetadata,
  #[serde(default)]
  pub viewport: GraphViewport,
  #[serde(default)]
  pub selection: GraphSelection,
  #[serde(default)]
  pub nodes: Vec<GraphNode>,
  #[serde(default)]
  pub edges: Vec<GraphEdge>,
}

impl GraphDocument {
  pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
    Self {
      id: id.into(),
      title: title.into(),
      metadata: GraphDocumentMetadata::default(),
      viewport: GraphViewport::default(),
      selection: GraphSelection::default(),
      nodes: Vec::new(),
      edges: Vec::new(),
    }
  }

  pub fn node(&self, node_id: &str) -> Option<&GraphNode> {
    self.nodes.iter().find(|node| node.id == node_id)
  }

  pub fn node_mut(&mut self, node_id: &str) -> Option<&mut GraphNode> {
    self.nodes.iter_mut().find(|node| node.id == node_id)
  }

  pub fn edge(&self, edge_id: &str) -> Option<&GraphEdge> {
    self.edges.iter().find(|edge| edge.id == edge_id)
  }

  pub fn edge_mut(&mut self, edge_id: &str) -> Option<&mut GraphEdge> {
    self.edges.iter_mut().find(|edge| edge.id == edge_id)
  }
}

fn default_properties() -> Value {
  Value::Object(Default::default())
}
