use dioxus::prelude::*;

use crate::contexts::panel::PanelContent;
use lx_graph_editor::commands::GraphCommand;
use lx_graph_editor::inspector::{GraphInspector, GraphInspectorContent};

use super::controller::try_flow_editor_state;

#[component]
pub fn FlowInspector(content: PanelContent) -> Element {
  let Some(mut state) = try_flow_editor_state() else {
    return rsx! {
      MissingFlowInspectorState { label: "The active flow editor is unavailable.".to_string() }
    };
  };

  let document = state.document.read().clone();
  let templates = state.templates.read().clone();
  let diagnostics = state.diagnostics.read().clone();
  let run_snapshot = state.run_snapshot.read().clone();
  let credential_options = state.credential_options.read().clone();
  let inspector_content = match content {
    PanelContent::FlowNode { node_id } => GraphInspectorContent::Node { node_id },
    PanelContent::FlowEdge { edge_id } => GraphInspectorContent::Edge { edge_id },
  };

  rsx! {
    GraphInspector {
      content: inspector_content,
      document,
      templates,
      diagnostics,
      run_snapshot,
      credential_options,
      on_command: move |command: GraphCommand| dispatch_flow_inspector_command(&mut state, command),
    }
  }
}

#[component]
fn MissingFlowInspectorState(label: String) -> Element {
  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface-variant)]",
      "{label}"
    }
  }
}

fn dispatch_flow_inspector_command(state: &mut super::controller::FlowEditorState, command: GraphCommand) {
  let _ = state.dispatch(command);
}
