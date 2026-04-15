use serde_json::Value;

use super::catalog::{GraphFieldKind, GraphNodeTemplate, PortDirection, field_schema, materialize_default_properties, node_template, port_template};
use super::model::{GraphDocument, GraphEdge, GraphEntityRef, GraphNode, GraphPoint, GraphPortRef, GraphSelection, GraphViewport};

#[derive(Clone, Debug, PartialEq)]
pub enum GraphCommand {
  AddNode { node_id: String, template_id: String, position: GraphPoint, label: Option<String> },
  RemoveNode { node_id: String },
  MoveNode { node_id: String, position: GraphPoint },
  Select { selection: GraphSelection },
  ConnectPorts { edge_id: String, from: GraphPortRef, to: GraphPortRef, label: Option<String> },
  DisconnectEdge { edge_id: String },
  SetViewport { viewport: GraphViewport },
  UpdateField { node_id: String, field_id: String, value: Value },
  DeleteSelection,
}

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum GraphCommandError {
  #[error("node `{0}` already exists")]
  DuplicateNodeId(String),
  #[error("edge `{0}` already exists")]
  DuplicateEdgeId(String),
  #[error("template `{0}` does not exist")]
  UnknownTemplate(String),
  #[error("node `{node_id}` does not exist")]
  UnknownNode { node_id: String },
  #[error("edge `{edge_id}` does not exist")]
  UnknownEdge { edge_id: String },
  #[error("port `{port_id}` does not exist on node `{node_id}`")]
  UnknownPort { node_id: String, port_id: String },
  #[error("selection references missing node `{0}`")]
  InvalidSelectionNode(String),
  #[error("selection references missing edge `{0}`")]
  InvalidSelectionEdge(String),
  #[error("selection anchor does not reference a selected entity")]
  InvalidSelectionAnchor,
  #[error("connection must flow from an output port to an input port")]
  InvalidConnectionDirection { from: PortDirection, to: PortDirection },
  #[error("ports `{from_port_id}` and `{to_port_id}` have incompatible types")]
  IncompatiblePortTypes { from_port_id: String, to_port_id: String },
  #[error("connection from `{from_node_id}:{from_port_id}` to `{to_node_id}:{to_port_id}` already exists")]
  DuplicateConnection { from_node_id: String, from_port_id: String, to_node_id: String, to_port_id: String },
  #[error("port `{node_id}:{port_id}` does not allow multiple connections")]
  PortConnectionLimitReached { node_id: String, port_id: String },
  #[error("field `{field_id}` does not exist on template `{template_id}`")]
  UnknownField { template_id: String, field_id: String },
  #[error("field `{field_id}` expects {expected}")]
  InvalidFieldValue { field_id: String, expected: &'static str },
  #[error("node `{node_id}` properties are not a JSON object")]
  InvalidPropertiesShape { node_id: String },
}

pub fn apply_graph_command(document: &mut GraphDocument, templates: &[GraphNodeTemplate], command: GraphCommand) -> Result<(), GraphCommandError> {
  match command {
    GraphCommand::AddNode { node_id, template_id, position, label } => add_node(document, templates, node_id, template_id, position, label),
    GraphCommand::RemoveNode { node_id } => remove_node(document, &node_id),
    GraphCommand::MoveNode { node_id, position } => move_node(document, &node_id, position),
    GraphCommand::Select { selection } => select(document, selection),
    GraphCommand::ConnectPorts { edge_id, from, to, label } => connect_ports(document, templates, edge_id, from, to, label),
    GraphCommand::DisconnectEdge { edge_id } => disconnect_edge(document, &edge_id),
    GraphCommand::SetViewport { viewport } => {
      document.viewport = viewport;
      Ok(())
    },
    GraphCommand::UpdateField { node_id, field_id, value } => update_field(document, templates, &node_id, &field_id, value),
    GraphCommand::DeleteSelection => {
      delete_selection(document);
      Ok(())
    },
  }
}

fn add_node(
  document: &mut GraphDocument,
  templates: &[GraphNodeTemplate],
  node_id: String,
  template_id: String,
  position: GraphPoint,
  label: Option<String>,
) -> Result<(), GraphCommandError> {
  if document.node(&node_id).is_some() {
    return Err(GraphCommandError::DuplicateNodeId(node_id));
  }

  let template = node_template(templates, &template_id).ok_or_else(|| GraphCommandError::UnknownTemplate(template_id.clone()))?;
  let node = GraphNode {
    id: node_id.clone(),
    template_id,
    label: label.or_else(|| template.default_label.clone()),
    metadata: Default::default(),
    position,
    properties: materialize_default_properties(template),
  };
  document.nodes.push(node);
  document.selection = GraphSelection::single_node(node_id);
  Ok(())
}

fn remove_node(document: &mut GraphDocument, node_id: &str) -> Result<(), GraphCommandError> {
  let original_len = document.nodes.len();
  document.nodes.retain(|node| node.id != node_id);
  if document.nodes.len() == original_len {
    return Err(GraphCommandError::UnknownNode { node_id: node_id.to_string() });
  }

  document.edges.retain(|edge| edge.from.node_id != node_id && edge.to.node_id != node_id);
  prune_selection(document);
  Ok(())
}

fn move_node(document: &mut GraphDocument, node_id: &str, position: GraphPoint) -> Result<(), GraphCommandError> {
  let node = document.node_mut(node_id).ok_or_else(|| GraphCommandError::UnknownNode { node_id: node_id.to_string() })?;
  node.position = position;
  Ok(())
}

fn select(document: &mut GraphDocument, selection: GraphSelection) -> Result<(), GraphCommandError> {
  validate_selection(document, &selection)?;
  document.selection = selection;
  Ok(())
}

fn connect_ports(
  document: &mut GraphDocument,
  templates: &[GraphNodeTemplate],
  edge_id: String,
  from: GraphPortRef,
  to: GraphPortRef,
  label: Option<String>,
) -> Result<(), GraphCommandError> {
  if document.edge(&edge_id).is_some() {
    return Err(GraphCommandError::DuplicateEdgeId(edge_id));
  }

  let from_node = document.node(&from.node_id).ok_or_else(|| GraphCommandError::UnknownNode { node_id: from.node_id.clone() })?;
  let to_node = document.node(&to.node_id).ok_or_else(|| GraphCommandError::UnknownNode { node_id: to.node_id.clone() })?;
  let from_port = port_template(templates, &from_node.template_id, &from.port_id)
    .ok_or_else(|| GraphCommandError::UnknownPort { node_id: from.node_id.clone(), port_id: from.port_id.clone() })?;
  let to_port = port_template(templates, &to_node.template_id, &to.port_id)
    .ok_or_else(|| GraphCommandError::UnknownPort { node_id: to.node_id.clone(), port_id: to.port_id.clone() })?;

  if from_port.direction != PortDirection::Output || to_port.direction != PortDirection::Input {
    return Err(GraphCommandError::InvalidConnectionDirection { from: from_port.direction, to: to_port.direction });
  }

  if let (Some(from_type), Some(to_type)) = (&from_port.data_type, &to_port.data_type)
    && from_type != to_type
  {
    return Err(GraphCommandError::IncompatiblePortTypes { from_port_id: from.port_id.clone(), to_port_id: to.port_id.clone() });
  }

  if document.edges.iter().any(|edge| edge.from == from && edge.to == to) {
    return Err(GraphCommandError::DuplicateConnection {
      from_node_id: from.node_id.clone(),
      from_port_id: from.port_id.clone(),
      to_node_id: to.node_id.clone(),
      to_port_id: to.port_id.clone(),
    });
  }

  if !from_port.allow_multiple && connection_count(document, &from) > 0 {
    return Err(GraphCommandError::PortConnectionLimitReached { node_id: from.node_id.clone(), port_id: from.port_id.clone() });
  }

  if !to_port.allow_multiple && connection_count(document, &to) > 0 {
    return Err(GraphCommandError::PortConnectionLimitReached { node_id: to.node_id.clone(), port_id: to.port_id.clone() });
  }

  document.edges.push(GraphEdge { id: edge_id.clone(), label, metadata: Default::default(), from, to });
  document.selection = GraphSelection::single_edge(edge_id);
  Ok(())
}

fn disconnect_edge(document: &mut GraphDocument, edge_id: &str) -> Result<(), GraphCommandError> {
  let original_len = document.edges.len();
  document.edges.retain(|edge| edge.id != edge_id);
  if document.edges.len() == original_len {
    return Err(GraphCommandError::UnknownEdge { edge_id: edge_id.to_string() });
  }
  prune_selection(document);
  Ok(())
}

fn update_field(document: &mut GraphDocument, templates: &[GraphNodeTemplate], node_id: &str, field_id: &str, value: Value) -> Result<(), GraphCommandError> {
  let template_id = {
    let node = document.node(node_id).ok_or_else(|| GraphCommandError::UnknownNode { node_id: node_id.to_string() })?;
    node.template_id.clone()
  };
  let field = field_schema(templates, &template_id, field_id)
    .ok_or_else(|| GraphCommandError::UnknownField { template_id: template_id.clone(), field_id: field_id.to_string() })?;
  validate_field_value(field, &value)?;

  let node = document.node_mut(node_id).ok_or_else(|| GraphCommandError::UnknownNode { node_id: node_id.to_string() })?;
  let properties = node.properties.as_object_mut().ok_or_else(|| GraphCommandError::InvalidPropertiesShape { node_id: node_id.to_string() })?;
  properties.insert(field_id.to_string(), value);
  Ok(())
}

fn delete_selection(document: &mut GraphDocument) {
  let node_ids = document.selection.node_ids.clone();
  let edge_ids = document.selection.edge_ids.clone();

  document.nodes.retain(|node| !node_ids.iter().any(|selected| selected == &node.id));
  document.edges.retain(|edge| {
    !edge_ids.iter().any(|selected| selected == &edge.id) && !node_ids.iter().any(|selected| selected == &edge.from.node_id || selected == &edge.to.node_id)
  });
  document.selection.clear();
}

fn validate_selection(document: &GraphDocument, selection: &GraphSelection) -> Result<(), GraphCommandError> {
  let mut node_ids = Vec::new();
  for node_id in &selection.node_ids {
    if document.node(node_id).is_none() {
      return Err(GraphCommandError::InvalidSelectionNode(node_id.clone()));
    }
    if !node_ids.iter().any(|seen| seen == node_id) {
      node_ids.push(node_id.clone());
    }
  }

  let mut edge_ids = Vec::new();
  for edge_id in &selection.edge_ids {
    if document.edge(edge_id).is_none() {
      return Err(GraphCommandError::InvalidSelectionEdge(edge_id.clone()));
    }
    if !edge_ids.iter().any(|seen| seen == edge_id) {
      edge_ids.push(edge_id.clone());
    }
  }

  if let Some(anchor) = &selection.anchor {
    let is_valid_anchor = match anchor {
      GraphEntityRef::Node(node_id) => node_ids.iter().any(|selected| selected == node_id),
      GraphEntityRef::Edge(edge_id) => edge_ids.iter().any(|selected| selected == edge_id),
    };
    if !is_valid_anchor {
      return Err(GraphCommandError::InvalidSelectionAnchor);
    }
  }

  Ok(())
}

fn validate_field_value(field: &super::catalog::GraphFieldSchema, value: &Value) -> Result<(), GraphCommandError> {
  match &field.kind {
    GraphFieldKind::Text | GraphFieldKind::TextArea => {
      if !value.is_string() {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "a string" });
      }
    },
    GraphFieldKind::Number => {
      if !value.is_number() {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "a number" });
      }
    },
    GraphFieldKind::Integer => {
      if !(value.as_i64().is_some() || value.as_u64().is_some()) {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "an integer" });
      }
    },
    GraphFieldKind::Boolean => {
      if !value.is_boolean() {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "a boolean" });
      }
    },
    GraphFieldKind::Select { options } => {
      let Some(selected) = value.as_str() else {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "a string option value" });
      };
      if !options.iter().any(|option| option.value == selected) {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "one of the declared select option values" });
      }
    },
    GraphFieldKind::StringList => {
      let Some(items) = value.as_array() else {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "an array of strings" });
      };
      if !items.iter().all(Value::is_string) {
        return Err(GraphCommandError::InvalidFieldValue { field_id: field.id.clone(), expected: "an array of strings" });
      }
    },
  }

  Ok(())
}

fn connection_count(document: &GraphDocument, port_ref: &GraphPortRef) -> usize {
  document.edges.iter().filter(|edge| edge.from == *port_ref || edge.to == *port_ref).count()
}

fn prune_selection(document: &mut GraphDocument) {
  let existing_node_ids: Vec<String> = document.nodes.iter().map(|node| node.id.clone()).collect();
  let existing_edge_ids: Vec<String> = document.edges.iter().map(|edge| edge.id.clone()).collect();

  document.selection.node_ids.retain(|node_id| existing_node_ids.iter().any(|existing| existing == node_id));
  document.selection.edge_ids.retain(|edge_id| existing_edge_ids.iter().any(|existing| existing == edge_id));
  if let Some(anchor) = &document.selection.anchor {
    let anchor_exists = match anchor {
      GraphEntityRef::Node(node_id) => document.selection.node_ids.iter().any(|selected| selected == node_id),
      GraphEntityRef::Edge(edge_id) => document.selection.edge_ids.iter().any(|selected| selected == edge_id),
    };
    if !anchor_exists {
      document.selection.anchor = None;
    }
  }
}
