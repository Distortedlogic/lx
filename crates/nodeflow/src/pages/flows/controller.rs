use dioxus::prelude::*;

use anyhow::Result as AnyhowResult;

use crate::contexts::panel::{PanelContent, PanelState};
use lx_graph_editor::catalog::{GraphCredentialOption, GraphNodeTemplate};
use lx_graph_editor::commands::{GraphCommand, GraphCommandError, apply_graph_command};
use lx_graph_editor::history::{GraphEditorAction, GraphHistoryState};
use lx_graph_editor::model::{GraphDocument, GraphPoint, GraphSelection};
use lx_graph_editor::protocol::{GraphRunSnapshot, GraphWidgetDiagnostic};

use super::product::{FlowCompileState, FlowProductKind, evaluate_flow_document, resolve_flow_product};
use super::sample::sample_document;
use super::storage::FlowPersistence;

#[derive(Clone, Copy)]
pub struct FlowEditorState {
  pub flow_id: Signal<String>,
  pub document: Signal<GraphDocument>,
  pub product_kind: Signal<FlowProductKind>,
  pub templates: Signal<Vec<GraphNodeTemplate>>,
  pub credential_options: Signal<Vec<GraphCredentialOption>>,
  pub diagnostics: Signal<Vec<GraphWidgetDiagnostic>>,
  pub compile_state: Signal<Option<FlowCompileState>>,
  pub canvas_size: Signal<(f64, f64)>,
  pub selection: Signal<GraphSelection>,
  pub validation_count: Signal<usize>,
  pub history: Signal<GraphHistoryState>,
  pub active_run_agent_id: Signal<Option<String>>,
  pub run_snapshot: Signal<Option<GraphRunSnapshot>>,
  pub status_message: Signal<Option<String>>,
  pub panel: PanelState,
}

impl FlowEditorState {
  pub fn provide(flow_id: String, persistence: &FlowPersistence) -> Self {
    let (initial_document, initial_status) = match persistence.load_or_seed(&flow_id) {
      Ok(document) => (document, None),
      Err(error) => (sample_document(&flow_id), Some(format!("Opened bundled sample after load failed: {error}"))),
    };
    let initial_selection = initial_document.selection.clone();
    let initial_product = resolve_flow_product(&initial_document, &flow_id);
    let panel = use_context::<PanelState>();
    let state = Self {
      flow_id: use_signal(|| flow_id),
      document: use_signal(|| initial_document),
      product_kind: use_signal(|| initial_product.kind),
      templates: use_signal(|| initial_product.templates),
      credential_options: use_signal(|| initial_product.credential_options),
      diagnostics: use_signal(Vec::new),
      compile_state: use_signal(|| Option::<FlowCompileState>::None),
      canvas_size: use_signal(|| (1200.0, 760.0)),
      selection: use_signal(|| initial_selection),
      validation_count: use_signal(|| 0usize),
      history: use_signal(GraphHistoryState::default),
      active_run_agent_id: use_signal(|| Option::<String>::None),
      run_snapshot: use_signal(|| Option::<GraphRunSnapshot>::None),
      status_message: use_signal(|| initial_status),
      panel,
    };
    state.recompute_diagnostics();
    state.sync_shell_state();
    use_context_provider(|| state);
    state
  }

  pub fn dispatch(&mut self, command: GraphCommand) -> std::result::Result<(), GraphCommandError> {
    let before = self.document.read().clone();
    let templates = self.templates.read().clone();
    let history_command = command.clone();
    let status = describe_command(&command);
    {
      let mut document = self.document.write();
      apply_graph_command(&mut document, &templates, command)?;
    }
    let after = self.document.read().clone();
    if before != after {
      let mut history = self.history;
      history.write().record_command(&before, &history_command);
    }
    self.recompute_diagnostics();
    self.sync_shell_state();
    if let Some(status) = status {
      self.status_message.set(Some(status));
    }
    Ok(())
  }

  pub fn register_canvas_size(&self, width: f64, height: f64) {
    let mut canvas_size = self.canvas_size;
    canvas_size.set((width, height));
  }

  pub fn current_canvas_size(&self) -> (f64, f64) {
    *self.canvas_size.read()
  }

  pub fn insert_template_at_viewport_center(&mut self, template_id: &str, scene_width: f64, scene_height: f64) -> std::result::Result<(), GraphCommandError> {
    let document = self.document.read().clone();
    let count_for_template = document.nodes.iter().filter(|node| node.template_id == template_id).count();
    let mut position = viewport_center(&document, scene_width, scene_height);
    position.x += count_for_template as f64 * 28.0;
    position.y += count_for_template as f64 * 20.0;
    let node_id = next_node_id(&document, template_id);
    self.dispatch(GraphCommand::AddNode { node_id, template_id: template_id.to_string(), position, label: None })
  }

  pub fn save(&self, persistence: &FlowPersistence) -> AnyhowResult<()> {
    let document = self.document.read().clone();
    persistence.save(&document)?;
    self.set_status_message(format!("Saved flow {}", document.id));
    Ok(())
  }

  pub fn save_as_new(&mut self, persistence: &FlowPersistence) -> AnyhowResult<String> {
    let current_document = self.document.read().clone();
    let next_document = persistence.save_as_new(&current_document)?;
    let new_flow_id = next_document.id.clone();
    self.replace_document(new_flow_id.clone(), next_document);
    self.set_status_message(format!("Saved new flow {new_flow_id}"));
    Ok(new_flow_id)
  }

  pub fn reset_to_sample(&mut self, persistence: &FlowPersistence) -> AnyhowResult<()> {
    let flow_id = self.flow_id.read().clone();
    let document = persistence.reset_to_sample(&flow_id)?;
    self.replace_document(flow_id.clone(), document);
    self.set_status_message(format!("Reset {flow_id} to the bundled sample"));
    Ok(())
  }

  pub fn apply_editor_action(&mut self, action: &GraphEditorAction) {
    match action {
      GraphEditorAction::Undo => {
        let current = self.document.read().clone();
        let next = {
          let mut history = self.history;
          history.write().undo(&current)
        };
        if let Some(document) = next {
          self.apply_document(document);
          self.set_status_message("Undid graph edit".to_string());
        }
      },
      GraphEditorAction::Redo => {
        let current = self.document.read().clone();
        let next = {
          let mut history = self.history;
          history.write().redo(&current)
        };
        if let Some(document) = next {
          self.apply_document(document);
          self.set_status_message("Redid graph edit".to_string());
        }
      },
      GraphEditorAction::CopySelection => {
        let current = self.document.read().clone();
        let copied = {
          let mut history = self.history;
          history.write().copy_selection(&current)
        };
        if copied {
          self.set_status_message("Copied selection".to_string());
        }
      },
      GraphEditorAction::PasteClipboard => {
        let current = self.document.read().clone();
        let next = {
          let mut history = self.history;
          let mut history_state = history.write();
          let next = history_state.paste_clipboard(&current);
          if next.is_some() {
            history_state.record_snapshot_change(&current);
          }
          next
        };
        if let Some(document) = next {
          self.apply_document(document);
          self.set_status_message("Pasted selection".to_string());
        }
      },
      GraphEditorAction::DuplicateSelection => {
        let current = self.document.read().clone();
        let next = {
          let mut history = self.history;
          let mut history_state = history.write();
          let next = history_state.duplicate_selection(&current);
          if next.is_some() {
            history_state.record_snapshot_change(&current);
          }
          next
        };
        if let Some(document) = next {
          self.apply_document(document);
          self.set_status_message("Duplicated selection".to_string());
        }
      },
      GraphEditorAction::SelectAll => {
        let current = self.document.read().clone();
        let selection = {
          let history = self.history;
          history.read().select_all(&current)
        };
        let _ = self.dispatch(GraphCommand::Select { selection });
      },
    }
  }

  pub fn set_active_run_surface(&self, agent_id: Option<String>, snapshot: Option<GraphRunSnapshot>) {
    if self.active_run_agent_id.read().as_ref() != agent_id.as_ref() {
      let mut active_run_agent_id = self.active_run_agent_id;
      active_run_agent_id.set(agent_id);
    }
    if *self.run_snapshot.read() != snapshot {
      let mut run_snapshot_signal = self.run_snapshot;
      run_snapshot_signal.set(snapshot);
    }
  }

  pub fn clear_run_surface(&self) {
    self.set_active_run_surface(None, None);
  }

  pub fn supports_runtime(&self) -> bool {
    self.product_kind.read().supports_runtime()
  }

  fn sync_shell_state(&self) {
    let selection = self.document.read().selection.clone();
    let validation_count = self.diagnostics.read().len();
    let mut selection_signal = self.selection;
    selection_signal.set(selection.clone());
    let mut validation_count_signal = self.validation_count;
    validation_count_signal.set(validation_count);
    sync_panel(&self.panel, &selection);
  }

  fn recompute_diagnostics(&self) {
    self.refresh_product_state();
    let document = self.document.read().clone();
    let kind = *self.product_kind.read();
    let templates = self.templates.read().clone();
    let evaluation = evaluate_flow_document(kind, &document, &templates);
    let mut diagnostics_signal = self.diagnostics;
    diagnostics_signal.set(evaluation.diagnostics);
    let mut compile_state_signal = self.compile_state;
    compile_state_signal.set(evaluation.compile_state);
  }

  fn replace_document(&self, flow_id: String, document: GraphDocument) {
    let mut flow_id_signal = self.flow_id;
    flow_id_signal.set(flow_id);
    self.apply_document(document);
    let mut history = self.history;
    history.write().clear();
    self.clear_run_surface();
  }

  pub fn set_status_message(&self, message: String) {
    let mut status_message = self.status_message;
    status_message.set(Some(message));
  }

  fn apply_document(&self, document: GraphDocument) {
    let selection = document.selection.clone();
    let mut document_signal = self.document;
    document_signal.set(document);
    let mut selection_signal = self.selection;
    selection_signal.set(selection);
    self.recompute_diagnostics();
    self.sync_shell_state();
  }

  fn refresh_product_state(&self) {
    let flow_id = self.flow_id.read().clone();
    let document = self.document.read().clone();
    let config = resolve_flow_product(&document, &flow_id);

    if *self.product_kind.read() != config.kind {
      let mut product_kind_signal = self.product_kind;
      product_kind_signal.set(config.kind);
    }
    if *self.templates.read() != config.templates {
      let mut templates_signal = self.templates;
      templates_signal.set(config.templates);
    }
    if *self.credential_options.read() != config.credential_options {
      let mut credential_options_signal = self.credential_options;
      credential_options_signal.set(config.credential_options);
    }
    if !config.kind.supports_runtime() {
      self.clear_run_surface();
    }
  }
}

pub fn use_flow_editor_state() -> FlowEditorState {
  use_context()
}

pub fn try_flow_editor_state() -> Option<FlowEditorState> {
  try_consume_context()
}

fn sync_panel(panel: &PanelState, selection: &GraphSelection) {
  if let Some(node_id) = selection.node_ids.first() {
    panel.open(PanelContent::FlowNode { node_id: node_id.clone() });
    return;
  }
  if let Some(edge_id) = selection.edge_ids.first() {
    panel.open(PanelContent::FlowEdge { edge_id: edge_id.clone() });
    return;
  }
  panel.close();
}

fn viewport_center(document: &GraphDocument, scene_width: f64, scene_height: f64) -> GraphPoint {
  let viewport = document.viewport;
  let width = scene_width.max(1.0);
  let height = scene_height.max(1.0);
  GraphPoint { x: (width * 0.5 - viewport.pan_x) / viewport.zoom, y: (height * 0.5 - viewport.pan_y) / viewport.zoom }
}

fn next_node_id(document: &GraphDocument, template_id: &str) -> String {
  let base = template_id.replace('_', "-");
  let mut candidate = base.clone();
  let mut index = 2usize;
  while document.node(&candidate).is_some() {
    candidate = format!("{base}-{index}");
    index += 1;
  }
  candidate
}

fn describe_command(command: &GraphCommand) -> Option<String> {
  match command {
    GraphCommand::AddNode { template_id, .. } => Some(format!("Added {template_id} node")),
    GraphCommand::RemoveNode { node_id } => Some(format!("Removed node {node_id}")),
    GraphCommand::MoveNode { node_id, .. } => Some(format!("Moved node {node_id}")),
    GraphCommand::Select { .. } => None,
    GraphCommand::ConnectPorts { edge_id, .. } => Some(format!("Connected edge {edge_id}")),
    GraphCommand::DisconnectEdge { edge_id } => Some(format!("Removed edge {edge_id}")),
    GraphCommand::SetViewport { .. } => None,
    GraphCommand::UpdateField { field_id, .. } => Some(format!("Updated field {field_id}")),
    GraphCommand::DeleteSelection => Some("Deleted selection".to_string()),
  }
}
