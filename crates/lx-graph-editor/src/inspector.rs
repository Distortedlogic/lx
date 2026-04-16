use dioxus::prelude::*;
use serde_json::{Value, json};

use crate::catalog::{
  GraphBoundFieldValue, GraphCredentialOption, GraphFieldKind, GraphFieldSchema, GraphFieldValueMode, GraphNodeTemplate, bound_field_value, node_template,
  unwrap_literal_field_value,
};
use crate::commands::GraphCommand;
use crate::model::{GraphDocument, GraphEntityRef};
use crate::protocol::{GraphEdgeRunState, GraphRunSnapshot, GraphRunStatus, GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

#[derive(Clone, PartialEq)]
pub enum GraphInspectorContent {
  Node { node_id: String },
  Edge { edge_id: String },
}

#[component]
pub fn GraphInspector(
  content: GraphInspectorContent,
  document: GraphDocument,
  templates: Vec<GraphNodeTemplate>,
  diagnostics: Vec<GraphWidgetDiagnostic>,
  run_snapshot: Option<GraphRunSnapshot>,
  credential_options: Vec<GraphCredentialOption>,
  on_command: EventHandler<GraphCommand>,
) -> Element {
  match content {
    GraphInspectorContent::Node { node_id } => rsx! {
      GraphNodeInspector {
        node_id,
        document,
        templates,
        diagnostics,
        run_snapshot,
        credential_options,
        on_command,
      }
    },
    GraphInspectorContent::Edge { edge_id } => rsx! {
      GraphEdgeInspector {
        edge_id,
        document,
        diagnostics,
        run_snapshot,
        on_command,
      }
    },
  }
}

#[component]
fn GraphNodeInspector(
  node_id: String,
  document: GraphDocument,
  templates: Vec<GraphNodeTemplate>,
  diagnostics: Vec<GraphWidgetDiagnostic>,
  run_snapshot: Option<GraphRunSnapshot>,
  credential_options: Vec<GraphCredentialOption>,
  on_command: EventHandler<GraphCommand>,
) -> Element {
  let Some(node) = document.node(&node_id).cloned() else {
    return rsx! {
      MissingInspectorState { label: format!("Node `{node_id}` is no longer present.") }
    };
  };
  let template = node_template(&templates, &node.template_id).cloned();
  let node_diagnostics: Vec<_> =
    diagnostics.into_iter().filter(|diagnostic| matches!(diagnostic.target, Some(GraphEntityRef::Node(ref id)) if id == &node_id)).collect();
  let node_run_state = run_snapshot.as_ref().and_then(|snapshot| snapshot.node_states.iter().find(|state| state.node_id == node_id).cloned());

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

      if let Some(run_state) = node_run_state {
        RunStateCard {
          title: "Node execution".to_string(),
          subtitle: run_snapshot.as_ref().and_then(|snapshot| snapshot.label.clone()),
          status: run_state.status,
          label: run_state.label.clone(),
          detail: run_state.detail.clone(),
          output_summary: run_state.output_summary.clone(),
          started_at: run_state.started_at.clone(),
          finished_at: run_state.finished_at.clone(),
          duration_ms: run_state.duration_ms,
        }
      } else if let Some(run_snapshot) = run_snapshot.clone() {
        RunStateCard {
          title: "Latest run".to_string(),
          subtitle: run_snapshot.label.clone(),
          status: run_snapshot.status,
          label: None,
          detail: run_snapshot.summary.clone(),
          output_summary: None,
          started_at: run_snapshot.started_at.clone(),
          finished_at: run_snapshot.finished_at.clone(),
          duration_ms: run_snapshot.duration_ms,
        }
      }

      if !node_diagnostics.is_empty() {
        DiagnosticStack { diagnostics: node_diagnostics }
      }

      if let Some(template) = template {
        div { class: "space-y-4",
          for field in template.fields.iter() {
            GraphFieldEditor {
              node_id: node.id.clone(),
              template: template.clone(),
              field: field.clone(),
              value: node.properties.as_object().and_then(|props| props.get(&field.id)).cloned(),
              credential_options: credential_options.clone(),
              on_command,
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
              on_command
                  .call(GraphCommand::RemoveNode {
                      node_id: node.id.clone(),
                  })
          },
          "Delete Node"
        }
      }
    }
  }
}

#[component]
fn GraphEdgeInspector(
  edge_id: String,
  document: GraphDocument,
  diagnostics: Vec<GraphWidgetDiagnostic>,
  run_snapshot: Option<GraphRunSnapshot>,
  on_command: EventHandler<GraphCommand>,
) -> Element {
  let Some(edge) = document.edge(&edge_id).cloned() else {
    return rsx! {
      MissingInspectorState { label: format!("Edge `{edge_id}` is no longer present.") }
    };
  };
  let source_label = format!("{}:{}", edge.from.node_id, edge.from.port_id);
  let target_label = format!("{}:{}", edge.to.node_id, edge.to.port_id);
  let edge_diagnostics: Vec<_> =
    diagnostics.into_iter().filter(|diagnostic| matches!(diagnostic.target, Some(GraphEntityRef::Edge(ref id)) if id == &edge_id)).collect();
  let edge_run_state = run_snapshot.as_ref().and_then(|snapshot| snapshot.edge_states.iter().find(|state| state.edge_id == edge_id).cloned());

  rsx! {
    div { class: "flex flex-col gap-4",
      InspectorHeader {
        eyebrow: "edge".to_string(),
        title: edge.label.clone().unwrap_or_else(|| edge.id.clone()),
        subtitle: format!("{} -> {}", edge.from.node_id, edge.to.node_id),
      }

      if let Some(run_state) = edge_run_state {
        EdgeRunStateCard {
          run_state,
          snapshot_label: run_snapshot.as_ref().and_then(|snapshot| snapshot.label.clone()),
        }
      } else if let Some(run_snapshot) = run_snapshot.clone() {
        RunStateCard {
          title: "Latest run".to_string(),
          subtitle: run_snapshot.label.clone(),
          status: run_snapshot.status,
          label: None,
          detail: run_snapshot.summary.clone(),
          output_summary: None,
          started_at: run_snapshot.started_at.clone(),
          finished_at: run_snapshot.finished_at.clone(),
          duration_ms: run_snapshot.duration_ms,
        }
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
              on_command
                  .call(GraphCommand::DisconnectEdge {
                      edge_id: edge.id.clone(),
                  })
          },
          "Delete Edge"
        }
      }
    }
  }
}

#[component]
fn EdgeRunStateCard(run_state: GraphEdgeRunState, snapshot_label: Option<String>) -> Element {
  rsx! {
    RunStateCard {
      title: "Connection execution".to_string(),
      subtitle: snapshot_label,
      status: run_state.status,
      label: run_state.label.clone(),
      detail: run_state.detail.clone(),
      output_summary: None,
      started_at: None,
      finished_at: None,
      duration_ms: None,
    }
  }
}

#[component]
fn GraphFieldEditor(
  node_id: String,
  template: GraphNodeTemplate,
  field: GraphFieldSchema,
  value: Option<Value>,
  credential_options: Vec<GraphCredentialOption>,
  on_command: EventHandler<GraphCommand>,
) -> Element {
  let field_id = field.id.clone();
  let current_value = value.or(field.default_value.clone()).unwrap_or(Value::Null);
  let binding_mode = field_binding_mode(&current_value);
  let literal_value = unwrap_literal_field_value(&current_value);
  let expression_value = match bound_field_value(&current_value) {
    Some(GraphBoundFieldValue::Expression { expression }) => expression,
    _ => String::new(),
  };
  let (credential_id, credential_key) = match bound_field_value(&current_value) {
    Some(GraphBoundFieldValue::Credential { credential_id, key }) => (credential_id, key.unwrap_or_default()),
    _ => (String::new(), String::new()),
  };
  let label = field.label.clone();
  let description = field.description.clone();
  let required = field.required;
  let supports_expression = field.capabilities.supports_expressions();
  let credential_requirement = field.capabilities.credential.clone();
  let filtered_credentials = credential_requirement
    .as_ref()
    .map(|requirement| {
      credential_options.into_iter().filter(|option| option.namespace == requirement.namespace && option.kind == requirement.kind).collect::<Vec<_>>()
    })
    .unwrap_or_default();
  let available_modes = available_field_modes(&field);

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
      if available_modes.len() > 1 {
        select {
          class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-xs font-medium uppercase tracking-[0.12em] text-[var(--on-surface-variant)] outline-none",
          value: "{field_mode_value(binding_mode)}",
          onchange: {
              let node_id = node_id.clone();
              let field_id = field_id.clone();
              let field = field.clone();
              let literal_value = literal_value.clone();
              let expression_value = expression_value.clone();
              let credential_id = credential_id.clone();
              let credential_key = credential_key.clone();
              move |evt| {
                  let next_mode = parse_field_mode(&evt.value());
                  commit_field_update(
                      on_command,
                      &node_id,
                      &field_id,
                      value_for_mode(
                          &field,
                          next_mode,
                          &literal_value,
                          &expression_value,
                          &credential_id,
                          &credential_key,
                      ),
                  );
              }
          },
          for mode in available_modes {
            option { value: "{field_mode_value(mode)}", "{field_mode_label(mode)}" }
          }
        }
      }
      match binding_mode {
          GraphFieldValueMode::Literal => rsx! {
            LiteralFieldEditor {
              node_id: node_id.clone(),
              field_id: field_id.clone(),
              field: field.clone(),
              value: literal_value.clone(),
              on_command,
            }
          },
          GraphFieldValueMode::Expression => rsx! {
            textarea {
              class: "min-h-24 w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 font-mono text-sm text-[var(--on-surface)] outline-none",
              value: "{expression_value}",
              placeholder: field
                  .capabilities
                  .expression
                  .as_ref()
                  .and_then(|entry| entry.placeholder.as_deref())
                  .unwrap_or("{{ steps.fetch.output }}"),
              oninput: {
                  let node_id = node_id.clone();
                  let field_id = field_id.clone();
                  move |evt| commit_field_update(
                      on_command,
                      &node_id,
                      &field_id,
                      json!({ "mode" : "expression", "expression" : evt.value() }),
                  )
              },
            }
            if supports_expression {
              p { class: "text-[11px] text-[var(--outline)]", "Expression-backed field" }
            }
          },
          GraphFieldValueMode::Credential => rsx! {
            if let Some(requirement) = credential_requirement {
              div { class: "space-y-2",
                select {
                  class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
                  value: "{credential_id}",
                  onchange: {
                      let node_id = node_id.clone();
                      let field_id = field_id.clone();
                      let credential_key = credential_key.clone();
                      move |evt| commit_field_update(
                          on_command,
                          &node_id,
                          &field_id,
                          credential_binding_value(evt.value(), &credential_key),
                      )
                  },
                  option { value: "", "Select {requirement.label}" }
                  for option in filtered_credentials.iter() {
                    option { value: "{option.id}", "{option.label}" }
                  }
                }
                if requirement.allow_key_selection {
                  input {
                    class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 font-mono text-sm text-[var(--on-surface)] outline-none",
                    r#type: "text",
                    value: "{credential_key}",
                    placeholder: "Optional secret key",
                    oninput: {
                        let node_id = node_id.clone();
                        let field_id = field_id.clone();
                        let credential_id = credential_id.clone();
                        move |evt| commit_field_update(
                            on_command,
                            &node_id,
                            &field_id,
                            credential_binding_value(credential_id.clone(), &evt.value()),
                        )
                    },
                  }
                }
                if filtered_credentials.is_empty() {
                  p { class: "text-[11px] text-[var(--outline)]",
                    "No matching credentials are currently available."
                  }
                } else {
                  p { class: "text-[11px] text-[var(--outline)]",
                    "The editor stores only a credential reference, not the secret itself."
                  }
                }
              }
            }
          },
      }
      p { class: "text-[11px] text-[var(--outline)]", "Template: {template.label}" }
    }
  }
}

#[component]
fn LiteralFieldEditor(node_id: String, field_id: String, field: GraphFieldSchema, value: Value, on_command: EventHandler<GraphCommand>) -> Element {
  match field.kind.clone() {
    GraphFieldKind::Text => rsx! {
      input {
        class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
        r#type: "text",
        value: "{value.as_str().unwrap_or_default()}",
        oninput: move |evt| commit_field_update(on_command, &node_id, &field_id, json!(evt.value())),
      }
    },
    GraphFieldKind::TextArea => rsx! {
      textarea {
        class: "min-h-28 w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
        value: "{value.as_str().unwrap_or_default()}",
        oninput: move |evt| commit_field_update(on_command, &node_id, &field_id, json!(evt.value())),
      }
    },
    GraphFieldKind::Number => rsx! {
      input {
        class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
        r#type: "number",
        step: "any",
        value: "{format_numeric_value(&value)}",
        onchange: move |evt| {
            if let Ok(value) = evt.value().parse::<f64>() {
                commit_field_update(on_command, &node_id, &field_id, json!(value));
            }
        },
      }
    },
    GraphFieldKind::Integer => rsx! {
      input {
        class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
        r#type: "number",
        step: "1",
        value: "{format_numeric_value(&value)}",
        onchange: move |evt| {
            if let Ok(value) = evt.value().parse::<i64>() {
                commit_field_update(on_command, &node_id, &field_id, json!(value));
            }
        },
      }
    },
    GraphFieldKind::Boolean => {
      let enabled = value.as_bool().unwrap_or(false);
      rsx! {
        button {
          class: if enabled { "inline-flex items-center rounded-full border border-emerald-500/30 bg-emerald-500/15 px-3 py-1.5 text-xs font-semibold text-emerald-300" } else { "inline-flex items-center rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-semibold text-[var(--on-surface-variant)]" },
          onclick: move |_| commit_field_update(on_command, &node_id, &field_id, json!(! enabled)),
          if enabled {
            "Enabled"
          } else {
            "Disabled"
          }
        }
      }
    },
    GraphFieldKind::Select { options } => {
      let selected = value.as_str().unwrap_or_default().to_string();
      rsx! {
        select {
          class: "w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
          value: "{selected}",
          onchange: move |evt| commit_field_update(on_command, &node_id, &field_id, json!(evt.value())),
          for option in options {
            option { value: "{option.value}", "{option.label}" }
          }
        }
      }
    },
    GraphFieldKind::StringList => rsx! {
      textarea {
        class: "min-h-28 w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
        value: "{string_list_value(&value)}",
        oninput: move |evt| {
            let items = evt
                .value()
                .lines()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(str::to_string)
                .collect::<Vec<_>>();
            commit_field_update(on_command, &node_id, &field_id, json!(items));
        },
      }
      p { class: "text-[11px] text-[var(--outline)]", "One item per line." }
    },
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

#[component]
fn RunStateCard(
  title: String,
  subtitle: Option<String>,
  status: GraphRunStatus,
  label: Option<String>,
  detail: Option<String>,
  output_summary: Option<String>,
  started_at: Option<String>,
  finished_at: Option<String>,
  duration_ms: Option<u64>,
) -> Element {
  let badge_label = label.unwrap_or_else(|| run_status_label(status).to_string());
  let badge_style = run_status_badge_style(status);
  let card_style = run_status_card_style(status);
  let duration = duration_ms.map(format_duration);

  rsx! {
    div { class: "rounded-2xl border px-4 py-3", style: "{card_style}",
      div { class: "flex items-start justify-between gap-3",
        div { class: "min-w-0",
          div { class: "text-[11px] font-mono uppercase tracking-[0.18em] text-[var(--outline)]",
            "{title}"
          }
          if let Some(subtitle) = subtitle {
            p { class: "mt-1 text-xs text-[var(--on-surface-variant)] truncate",
              "{subtitle}"
            }
          }
        }
        span {
          class: "rounded-full border px-2.5 py-1 text-[11px] font-semibold",
          style: "{badge_style}",
          "{badge_label}"
        }
      }
      div { class: "mt-3 flex flex-col gap-2 text-sm text-[var(--on-surface-variant)]",
        if let Some(detail) = detail {
          p { class: "text-[var(--on-surface)]", "{detail}" }
        }
        if let Some(output_summary) = output_summary {
          div {
            class: "rounded-xl border px-3 py-2 text-[13px] leading-5",
            style: "border-color: color-mix(in srgb, var(--outline-variant) 62%, transparent); background: color-mix(in srgb, var(--surface-container-high) 72%, transparent); color: var(--on-surface);",
            "{output_summary}"
          }
        }
        if started_at.is_some() || finished_at.is_some() || duration.is_some() {
          div { class: "grid gap-2 text-xs text-[var(--outline)]",
            if let Some(started_at) = started_at {
              div { class: "flex items-center justify-between gap-3",
                span { "Started" }
                span { class: "font-mono text-[var(--on-surface)]", "{started_at}" }
              }
            }
            if let Some(finished_at) = finished_at {
              div { class: "flex items-center justify-between gap-3",
                span { "Finished" }
                span { class: "font-mono text-[var(--on-surface)]", "{finished_at}" }
              }
            }
            if let Some(duration) = duration {
              div { class: "flex items-center justify-between gap-3",
                span { "Duration" }
                span { class: "font-mono text-[var(--on-surface)]", "{duration}" }
              }
            }
          }
        }
      }
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

fn commit_field_update(on_command: EventHandler<GraphCommand>, node_id: &str, field_id: &str, value: Value) {
  on_command.call(GraphCommand::UpdateField { node_id: node_id.to_string(), field_id: field_id.to_string(), value });
}

fn available_field_modes(field: &GraphFieldSchema) -> Vec<GraphFieldValueMode> {
  let mut modes = vec![GraphFieldValueMode::Literal];
  if field.capabilities.supports_expressions() {
    modes.push(GraphFieldValueMode::Expression);
  }
  if field.capabilities.supports_credentials() {
    modes.push(GraphFieldValueMode::Credential);
  }
  modes
}

fn field_binding_mode(value: &Value) -> GraphFieldValueMode {
  bound_field_value(value).map(|binding| binding.mode()).unwrap_or(GraphFieldValueMode::Literal)
}

fn field_mode_value(mode: GraphFieldValueMode) -> &'static str {
  match mode {
    GraphFieldValueMode::Literal => "literal",
    GraphFieldValueMode::Expression => "expression",
    GraphFieldValueMode::Credential => "credential",
  }
}

fn field_mode_label(mode: GraphFieldValueMode) -> &'static str {
  match mode {
    GraphFieldValueMode::Literal => "Literal",
    GraphFieldValueMode::Expression => "Expression",
    GraphFieldValueMode::Credential => "Credential",
  }
}

fn parse_field_mode(value: &str) -> GraphFieldValueMode {
  match value {
    "expression" => GraphFieldValueMode::Expression,
    "credential" => GraphFieldValueMode::Credential,
    _ => GraphFieldValueMode::Literal,
  }
}

fn value_for_mode(
  field: &GraphFieldSchema,
  mode: GraphFieldValueMode,
  literal_value: &Value,
  expression_value: &str,
  credential_id: &str,
  credential_key: &str,
) -> Value {
  match mode {
    GraphFieldValueMode::Literal => {
      if literal_value.is_null() {
        default_literal_value(&field.kind)
      } else {
        literal_value.clone()
      }
    },
    GraphFieldValueMode::Expression => json!({
      "mode": "expression",
      "expression": if expression_value.trim().is_empty() {
        field.capabilities.expression.as_ref().and_then(|entry| entry.placeholder.clone()).unwrap_or_default()
      } else {
        expression_value.to_string()
      }
    }),
    GraphFieldValueMode::Credential => credential_binding_value(credential_id.to_string(), credential_key),
  }
}

fn credential_binding_value(credential_id: impl Into<String>, key: &str) -> Value {
  let credential_id = credential_id.into();
  let trimmed_key = key.trim();
  if trimmed_key.is_empty() {
    json!({ "mode": "credential", "credential_id": credential_id })
  } else {
    json!({ "mode": "credential", "credential_id": credential_id, "key": trimmed_key })
  }
}

fn default_literal_value(kind: &GraphFieldKind) -> Value {
  match kind {
    GraphFieldKind::Text | GraphFieldKind::TextArea | GraphFieldKind::Select { .. } => json!(""),
    GraphFieldKind::Number | GraphFieldKind::Integer => json!(0),
    GraphFieldKind::Boolean => json!(false),
    GraphFieldKind::StringList => json!([]),
  }
}

fn run_status_label(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => "idle",
    GraphRunStatus::Pending => "pending",
    GraphRunStatus::Running => "running",
    GraphRunStatus::Succeeded => "succeeded",
    GraphRunStatus::Warning => "warning",
    GraphRunStatus::Failed => "failed",
    GraphRunStatus::Cancelled => "cancelled",
  }
}

fn run_status_badge_style(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => {
      "border-color: color-mix(in srgb, var(--outline-variant) 68%, transparent); background: color-mix(in srgb, var(--surface-container-high) 72%, transparent); color: var(--on-surface-variant);"
    },
    GraphRunStatus::Pending => {
      "border-color: color-mix(in srgb, var(--warning) 28%, transparent); background: color-mix(in srgb, var(--warning) 14%, transparent); color: color-mix(in srgb, var(--on-surface) 78%, var(--warning) 22%);"
    },
    GraphRunStatus::Running => {
      "border-color: color-mix(in srgb, var(--primary) 34%, transparent); background: color-mix(in srgb, var(--primary) 18%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--primary) 18%);"
    },
    GraphRunStatus::Succeeded => {
      "border-color: color-mix(in srgb, var(--success) 34%, transparent); background: color-mix(in srgb, var(--success) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--success) 18%);"
    },
    GraphRunStatus::Warning => {
      "border-color: color-mix(in srgb, var(--warning) 34%, transparent); background: color-mix(in srgb, var(--warning) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--warning) 18%);"
    },
    GraphRunStatus::Failed => {
      "border-color: color-mix(in srgb, var(--error) 36%, transparent); background: color-mix(in srgb, var(--error) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 80%, var(--error) 20%);"
    },
    GraphRunStatus::Cancelled => {
      "border-color: color-mix(in srgb, var(--outline) 36%, transparent); background: color-mix(in srgb, var(--surface-container-high) 76%, transparent); color: var(--on-surface-variant);"
    },
  }
}

fn run_status_card_style(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => {
      "border-color: color-mix(in srgb, var(--outline-variant) 62%, transparent); background: color-mix(in srgb, var(--surface-container-high) 42%, transparent);"
    },
    GraphRunStatus::Pending => {
      "border-color: color-mix(in srgb, var(--warning) 22%, transparent); background: color-mix(in srgb, var(--warning) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Running => {
      "border-color: color-mix(in srgb, var(--primary) 24%, transparent); background: color-mix(in srgb, var(--primary) 9%, var(--surface-container-low));"
    },
    GraphRunStatus::Succeeded => {
      "border-color: color-mix(in srgb, var(--success) 24%, transparent); background: color-mix(in srgb, var(--success) 9%, var(--surface-container-low));"
    },
    GraphRunStatus::Warning => {
      "border-color: color-mix(in srgb, var(--warning) 24%, transparent); background: color-mix(in srgb, var(--warning) 10%, var(--surface-container-low));"
    },
    GraphRunStatus::Failed => {
      "border-color: color-mix(in srgb, var(--error) 24%, transparent); background: color-mix(in srgb, var(--error) 10%, var(--surface-container-low));"
    },
    GraphRunStatus::Cancelled => {
      "border-color: color-mix(in srgb, var(--outline) 22%, transparent); background: color-mix(in srgb, var(--surface-container-high) 48%, transparent);"
    },
  }
}

fn format_duration(duration_ms: u64) -> String {
  if duration_ms < 1_000 {
    return format!("{duration_ms} ms");
  }
  if duration_ms < 60_000 {
    return format!("{:.1} s", duration_ms as f64 / 1_000.0);
  }
  let minutes = duration_ms / 60_000;
  let seconds = (duration_ms % 60_000) / 1_000;
  format!("{minutes}m {seconds}s")
}
