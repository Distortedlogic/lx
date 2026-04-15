use serde::{Deserialize, Serialize};

use super::catalog::GraphNodeTemplate;
use super::model::{GraphDocument, GraphPoint, GraphPortRef, GraphSelection, GraphViewport};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphWidgetDiagnosticSeverity {
  Info,
  Warning,
  Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphWidgetDiagnostic {
  pub id: String,
  pub severity: GraphWidgetDiagnosticSeverity,
  pub message: String,
  pub target: Option<super::model::GraphEntityRef>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphWidgetSnapshot {
  pub document: GraphDocument,
  #[serde(default)]
  pub templates: Vec<GraphNodeTemplate>,
  #[serde(default)]
  pub diagnostics: Vec<GraphWidgetDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GraphWidgetEvent {
  /// Replace the current editor selection with the ids reported by the widget.
  SelectionChanged { selection: GraphSelection },
  /// Commit the final node position after a drag gesture completes.
  NodeMoved { node_id: String, position: GraphPoint },
  /// Commit a completed connection from one port to another.
  EdgeCreated { edge_id: String, from: GraphPortRef, to: GraphPortRef, label: Option<String> },
  /// Commit the latest viewport pan/zoom after an interaction completes.
  ViewportChanged { viewport: GraphViewport },
  /// Delete whatever nodes and edges are currently selected.
  SelectionDeleted,
}
