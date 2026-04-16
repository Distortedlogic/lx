use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::contexts::panel::PanelContent;
use crate::graph_editor::catalog::{GraphFieldKind, GraphFieldSchema, GraphNodeTemplate, node_template};
use crate::graph_editor::commands::GraphCommand;
use crate::graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

use super::controller::try_flow_editor_state;

#[component]
pub fn FlowInspector(content: PanelContent) -> Element {
  match content {
    PanelContent::FlowNode { node_id } => rsx! {
      FlowNodeInspector { node_id }
    },
    PanelContent::FlowEdge { edge_id } => rsx! {
      FlowEdgeInspector { edge_id }
    },
  }
}

#[component]
fn FlowNodeInspector(node_id: String) -> Element {
  let Some(mut state) = try_flow_editor_state() else {
    return rsx! {
      MissingInspectorState { label: "The active flow editor is unavailable.".to_string() }
    };
  };
  let document = state.document.read().clone();
  let templates = state.templates.read().clone();
  let diagnostics = state.diagnostics.read().clone();
  let Some(node) = document.node(&node_id).cloned() else {
    return rsx! {
      MissingInspectorState { label: format!("Node `{node_id}` is no longer present.") }
    };
  };
  let template = node_template(&templates, &node.template_id).cloned();
  let node_diagnostics: Vec<_> = diagnostics
    .into_iter()
    .filter(|diagnostic| matches!(diagnostic.target, Some(crate::graph_editor::model::GraphEntityRef::Node(ref id)) if id == &node_id))
    .collect();

  rsx! {
    div { class: "flex flex-col gap-4",
      InspectorHeader {
        eyebrow: template
            .as_ref()
            .and_then(|entry| entry.category.clone())
            .unwrap_or_else(|| "node".to_string()),
        title: node.label.clone().unwrap_or_else(|| node.id.clone()),
        subtitle: format!(
            "{} • {}",
            node.id,
            template.as_ref().map_or(node.template_id.clone(), |entry| entry.label.clone()),
        ),
      }

      if !node_diagnostics.is_empty() {
        DiagnosticStack { diagnostics: node_diagnostics }
      }

      if let Some(template) = template {
        div { class: "space-y-4",
          for field in template.fields.iter() {
            FlowFieldEditor {
              node_id: node.id.clone(),
              template: template.clone(),
              field: field.clone(),
              value: node.properties.as_object().and_then(|props| props.get(&field.id)).cloned(),
            }
          }
        }
      } else {
        MissingInspectorState { label: format!("Template `{}` is missing for this node.", node.template_id) }
      }

      div { class: "flex items-center gap-2 pt-2",
        button {
          class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-medium text-[var(--on-surface)]",
          onclick: move |_| {
              let _ = state
                  .dispatch(GraphCommand::RemoveNode {
                      node_id: node.id.clone(),
                  });
          },
          "Delete Node"
        }
      }
    }
  }
}

#[component]
fn FlowEdgeInspector(edge_id: String) -> Element {
  let Some(mut state) = try_flow_editor_state() else {
    return rsx! {
      MissingInspectorState { label: "The active flow editor is unavailable.".to_string() }
    };
  };
  let document = state.document.read().clone();
  let diagnostics = state.diagnostics.read().clone();
  let Some(edge) = document.edge(&edge_id).cloned() else {
    return rsx! {
      MissingInspectorState { label: format!("Edge `{edge_id}` is no longer present.") }
    };
  };
  let source_label = format!("{}:{}", edge.from.node_id, edge.from.port_id);
  let target_label = format!("{}:{}", edge.to.node_id, edge.to.port_id);
  let edge_diagnostics: Vec<_> = diagnostics
    .into_iter()
    .filter(|diagnostic| matches!(diagnostic.target, Some(crate::graph_editor::model::GraphEntityRef::Edge(ref id)) if id == &edge_id))
    .collect();

  rsx! {
    div { class: "flex flex-col gap-4",
      InspectorHeader {
        eyebrow: "edge".to_string(),
        title: edge.label.clone().unwrap_or_else(|| edge.id.clone()),
        subtitle: format!("{} -> {}", edge.from.node_id, edge.to.node_id),
      }

      if !edge_diagnostics.is_empty() {
        DiagnosticStack { diagnostics: edge_diagnostics }
      }

      div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-high)] p-3 text-sm text-[var(--on-surface-variant)]",
        div { class: "flex items-center justify-between gap-3",
          span { "Source" }
          span { class: "font-mono text-xs text-[var(--on-surface)]", "{source_label}" }
        }
        div { class: "mt-2 flex items-center justify-between gap-3",
          span { "Target" }
          span { class: "font-mono text-xs text-[var(--on-surface)]", "{target_label}" }
        }
      }

      div { class: "flex items-center gap-2 pt-2",
        button {
          class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-medium text-[var(--on-surface)]",
          onclick: move |_| {
              let _ = state
                  .dispatch(GraphCommand::DisconnectEdge {
                      edge_id: edge.id.clone(),
                  });
          },
          "Delete Edge"
        }
      }
    }
  }
}

#[component]
fn FlowFieldEditor(node_id: String, template: GraphNodeTemplate, field: GraphFieldSchema, value: Option<Value>) -> Element {
  let Some(state) = try_flow_editor_state() else {
    return rsx! {
      MissingInspectorState { label: "The active flow editor is unavailable.".to_string() }
    };
  };
  let field_id = field.id.clone();
  let current_value = value.or(field.default_value.clone()).unwrap_or(Value::Null);
  let label = field.label.clone();
  let description = field.description.clone();
  let required = field.required;

  rsx! {
    div { class: "space-y-1.5",
      label { class: "text-sm font-medium text-[var(--on-surface)]",
        "{label}"
        if required {
          span { class: "ml-1 text-[var(--error)]", "*" }
        }
      }
      if let Some(description) = description {
        p { class: "text-xs leading-5 text-[var(--outline)]", "{description}" }
      }
      match field.kind.clone() {
          GraphFieldKind::Text => rsx! {
            input {
              class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
              r#type: "text",
              value: "{current_value.as_str().unwrap_or_default()}",
              oninput: {
                  let mut state = state;
                  let node_id = node_id.clone();
                  let field_id = field_id.clone();
                  move |evt| commit_field_update(
                      &mut state,
                      &node_id,
                      &field_id,
                      json!(evt.value()),
                  )
              },
            }
          },
          GraphFieldKind::TextArea => rsx! {
            textarea {
              class: "min-h-28 w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
              value: "{current_value.as_str().unwrap_or_default()}",
              oninput: {
                  let mut state = state;
                  let node_id = node_id.clone();
                  let field_id = field_id.clone();
                  move |evt| commit_field_update(
                      &mut state,
                      &node_id,
                      &field_id,
                      json!(evt.value()),
                  )
              },
            }
          },
          GraphFieldKind::Number => rsx! {
            input {
              class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
              r#type: "number",
              step: "any",
              value: "{format_numeric_value(&current_value)}",
              onchange: {
                  let mut state = state;
                  let node_id = node_id.clone();
                  let field_id = field_id.clone();
                  move |evt| {
                      if let Ok(value) = evt.value().parse::<f64>() {
                          commit_field_update(&mut state, &node_id, &field_id, json!(value));
                      }
                  }
              },
            }
          },
          GraphFieldKind::Integer => rsx! {
            input {
              class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
              r#type: "number",
              step: "1",
              value: "{format_numeric_value(&current_value)}",
              onchange: {
                  let mut state = state;
                  let node_id = node_id.clone();
                  let field_id = field_id.clone();
                  move |evt| {
                      if let Ok(value) = evt.value().parse::<i64>() {
                          commit_field_update(&mut state, &node_id, &field_id, json!(value));
                      }
                  }
              },
            }
          },
          GraphFieldKind::Boolean => {
              let enabled = current_value.as_bool().unwrap_or(false);
              rsx! {
                button {
                  class: if enabled { "inline-flex items-center rounded-full border border-emerald-500/30 bg-emerald-500/15 px-3 py-1.5 text-xs font-semibold text-emerald-300" } else { "inline-flex items-center rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-semibold text-[var(--on-surface-variant)]" },
                  onclick: {
                      let mut state = state;
                      let node_id = node_id.clone();
                      let field_id = field_id.clone();
                      move |_| commit_field_update(&mut state, &node_id, &field_id, json!(! enabled))
                  },
                  if enabled {
                    "Enabled"
                  } else {
                    "Disabled"
                  }
                }
              }
          }
          GraphFieldKind::Select { options } => {
              let selected = current_value.as_str().unwrap_or_default().to_string();
              rsx! {
                select {
                  class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
                  value: "{selected}",
                  onchange: {
                      let mut state = state;
                      let node_id = node_id.clone();
                      let field_id = field_id.clone();
                      move |evt| commit_field_update(
                          &mut state,
                          &node_id,
                          &field_id,
                          json!(evt.value()),
                      )
                  },
                  for option in options {
                    option { value: "{option.value}", "{option.label}" }
                  }
                }
              }
          }
          GraphFieldKind::StringList => rsx! {
            textarea {
              class: "min-h-28 w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
              value: "{string_list_value(&current_value)}",
              oninput: {
                  let mut state = state;
                  let node_id = node_id.clone();
                  let field_id = field_id.clone();
                  move |evt| {
                      let items = evt
                          .value()
                          .lines()
                          .map(str::trim)
                          .filter(|line| !line.is_empty())
                          .map(str::to_string)
                          .collect::<Vec<_>>();
                      commit_field_update(&mut state, &node_id, &field_id, json!(items));
                  }
              },
            }
            p { class: "text-[11px] text-[var(--outline)]", "One item per line." }
          },
      }
      p { class: "text-[11px] text-[var(--outline)]", "Template: {template.label}" }
    }
  }
}

#[component]
fn InspectorHeader(eyebrow: String, title: String, subtitle: String) -> Element {
  rsx! {
    div { class: "space-y-1",
      div { class: "text-[11px] font-mono uppercase tracking-[0.18em] text-[var(--outline)]",
        "{eyebrow}"
      }
      h2 { class: "text-lg font-semibold text-[var(--on-surface)]", "{title}" }
      p { class: "text-xs text-[var(--on-surface-variant)]", "{subtitle}" }
    }
  }
}

#[component]
fn DiagnosticStack(diagnostics: Vec<GraphWidgetDiagnostic>) -> Element {
  rsx! {
    div { class: "flex flex-col gap-2",
      for diagnostic in diagnostics {
        div {
          key: "{diagnostic.id}",
          class: match diagnostic.severity {
              GraphWidgetDiagnosticSeverity::Error => {
                  "rounded-xl border border-red-500/30 bg-red-500/8 px-3 py-2 text-sm text-red-200"
              }
              GraphWidgetDiagnosticSeverity::Warning => {
                  "rounded-xl border border-amber-500/30 bg-amber-500/8 px-3 py-2 text-sm text-amber-100"
              }
              GraphWidgetDiagnosticSeverity::Info => {
                  "rounded-xl border border-sky-500/30 bg-sky-500/8 px-3 py-2 text-sm text-sky-100"
              }
          },
          "{diagnostic.message}"
        }
      }
    }
  }
}

#[component]
fn MissingInspectorState(label: String) -> Element {
  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface-variant)]",
      "{label}"
    }
  }
}

fn string_list_value(value: &Value) -> String {
  value.as_array().map(|items| items.iter().filter_map(Value::as_str).collect::<Vec<_>>().join("\n")).unwrap_or_default()
}

fn format_numeric_value(value: &Value) -> String {
  value
    .as_i64()
    .map(|number| number.to_string())
    .or_else(|| value.as_u64().map(|number| number.to_string()))
    .or_else(|| value.as_f64().map(|number| number.to_string()))
    .unwrap_or_default()
}

fn commit_field_update(state: &mut super::controller::FlowEditorState, node_id: &str, field_id: &str, value: Value) {
  let _ = state.dispatch(GraphCommand::UpdateField { node_id: node_id.to_string(), field_id: field_id.to_string(), value });
}
