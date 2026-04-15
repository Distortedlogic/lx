use dioxus::prelude::*;

use crate::graph_editor::catalog::GraphNodeTemplate;
use crate::graph_editor::commands::{GraphCommand, GraphCommandError, apply_graph_command};
use crate::graph_editor::model::{GraphDocument, GraphSelection};

use super::sample::{sample_document, sample_templates};

#[derive(Clone, Copy)]
pub struct FlowEditorState {
  pub flow_id: Signal<String>,
  pub document: Signal<GraphDocument>,
  pub templates: Signal<Vec<GraphNodeTemplate>>,
  pub selection: Signal<GraphSelection>,
  pub validation_count: Signal<usize>,
  pub status_message: Signal<Option<String>>,
}

impl FlowEditorState {
  pub fn provide(flow_id: String) -> Self {
    let initial_document = sample_document(&flow_id);
    let initial_selection = initial_document.selection.clone();
    let initial_templates = sample_templates();
    let state = Self {
      flow_id: use_signal(|| flow_id),
      document: use_signal(|| initial_document),
      templates: use_signal(|| initial_templates),
      selection: use_signal(|| initial_selection),
      validation_count: use_signal(|| 0usize),
      status_message: use_signal(|| Some("Route host ready".to_string())),
    };
    use_context_provider(|| state);
    state
  }

  pub fn dispatch(&mut self, command: GraphCommand) -> Result<(), GraphCommandError> {
    let templates = self.templates.read().clone();
    let status = describe_command(&command);
    let selection = {
      let mut document = self.document.write();
      apply_graph_command(&mut document, &templates, command)?;
      document.selection.clone()
    };
    self.selection.set(selection);
    self.status_message.set(Some(status));
    Ok(())
  }
}

pub fn use_flow_editor_state() -> FlowEditorState {
  use_context()
}

fn describe_command(command: &GraphCommand) -> String {
  match command {
    GraphCommand::AddNode { template_id, .. } => format!("Added {template_id} node"),
    GraphCommand::RemoveNode { node_id } => format!("Removed node {node_id}"),
    GraphCommand::MoveNode { node_id, .. } => format!("Moved node {node_id}"),
    GraphCommand::Select { .. } => "Updated selection".to_string(),
    GraphCommand::ConnectPorts { edge_id, .. } => format!("Connected edge {edge_id}"),
    GraphCommand::DisconnectEdge { edge_id } => format!("Removed edge {edge_id}"),
    GraphCommand::SetViewport { .. } => "Updated viewport".to_string(),
    GraphCommand::UpdateField { field_id, .. } => format!("Updated field {field_id}"),
    GraphCommand::DeleteSelection => "Deleted selection".to_string(),
  }
}
