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
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub source: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub detail: Option<String>,
  pub target: Option<super::model::GraphEntityRef>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphRunStatus {
  #[default]
  Idle,
  Pending,
  Running,
  Succeeded,
  Warning,
  Failed,
  Cancelled,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphNodeRunState {
  pub node_id: String,
  pub status: GraphRunStatus,
  pub label: Option<String>,
  pub detail: Option<String>,
  pub output_summary: Option<String>,
  pub started_at: Option<String>,
  pub finished_at: Option<String>,
  pub duration_ms: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphEdgeRunState {
  pub edge_id: String,
  pub status: GraphRunStatus,
  pub label: Option<String>,
  pub detail: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GraphRunSnapshot {
  pub id: String,
  pub status: GraphRunStatus,
  pub label: Option<String>,
  pub summary: Option<String>,
  pub started_at: Option<String>,
  pub finished_at: Option<String>,
  pub duration_ms: Option<u64>,
  #[serde(default)]
  pub node_states: Vec<GraphNodeRunState>,
  #[serde(default)]
  pub edge_states: Vec<GraphEdgeRunState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GraphWidgetSnapshot {
  pub document: GraphDocument,
  #[serde(default)]
  pub templates: Vec<GraphNodeTemplate>,
  #[serde(default)]
  pub diagnostics: Vec<GraphWidgetDiagnostic>,
  #[serde(default)]
  pub run_snapshot: Option<GraphRunSnapshot>,
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
