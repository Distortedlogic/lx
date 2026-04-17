use dioxus::prelude::*;

use crate::contexts::breadcrumb::BreadcrumbEntry;
use crate::routes::Route;
use lx_graph_editor::catalog::GraphNodeTemplate;
use lx_graph_editor::commands::GraphCommand;
use lx_graph_editor::dioxus::{GraphCanvas, GraphCanvasSafeArea};
use lx_graph_editor::history::GraphEditorAction;
use lx_graph_editor::model::{GraphEntityRef, GraphSelection};
use lx_graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

use super::controller::use_flow_editor_state;
use super::product::{FlowCompileState, FlowCompileStatus, FlowProductKind};
use super::sample::{DEFAULT_FLOW_ID, DEFAULT_LX_FLOW_ID, DEFAULT_MERMAID_FLOW_ID};
use super::storage::use_flow_persistence;

#[component]
pub fn FlowWorkspace() -> Element {
  let mut state = use_flow_editor_state();
  let persistence = use_flow_persistence();
  let save_persistence = persistence.clone();
  let save_as_new_persistence = persistence.clone();
  let reset_persistence = persistence.clone();
  let breadcrumb_state = use_context::<crate::contexts::breadcrumb::BreadcrumbState>();
  let navigator = use_navigator();
  let mut palette_query = use_signal(String::new);
  let mut palette_open = use_signal(|| false);

  let flow_id = state.flow_id.read().clone();
  let document = state.document.read().clone();
  let product_kind = *state.product_kind.read();
  let templates = state.templates.read().clone();
  let diagnostics = state.diagnostics.read().clone();
  let compile_state = state.compile_state.read().clone();
  let run_snapshot = state.run_snapshot.read().clone();
  let selection = state.selection.read().clone();
  let status_message = state.status_message.read().clone();
  let title = document.title.clone();
  let breadcrumb_title = title.clone();
  let breadcrumb_flow_id = flow_id.clone();
  let flow_notes = document.metadata.notes.clone();
  let node_count = document.nodes.len();
  let edge_count = document.edges.len();
  let selection_summary = selection_summary(&selection);
  let error_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error).count();
  let warning_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Warning).count();
  let validation_summary = if diagnostics.is_empty() { "Healthy graph".to_string() } else { format!("{error_count} errors / {warning_count} warnings") };
  let query = palette_query.read().trim().to_lowercase();
  let filtered_templates: Vec<_> = templates.iter().filter(|template| palette_matches(template, &query)).cloned().collect();
  let palette_is_open = *palette_open.read();
  let primary_action_button_class = "rounded-xl border px-3 py-2 text-xs font-semibold transition-all hover:brightness-105";
  let primary_action_button_style = "border-color: color-mix(in srgb, var(--primary) 68%, transparent); background: var(--primary); color: var(--on-primary);";
  let secondary_action_button_class = "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-xs font-medium text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]";
  let tertiary_action_button_class = "rounded-xl border border-transparent bg-transparent px-3 py-2 text-xs font-medium text-[var(--on-surface-variant)] transition-colors hover:border-[var(--outline-variant)] hover:bg-[var(--surface-container-high)] hover:text-[var(--on-surface)]";

  use_effect(move || {
    breadcrumb_state.set(vec![
      BreadcrumbEntry { label: "Flows".into(), href: Some("/flows".into()) },
      BreadcrumbEntry { label: breadcrumb_title.clone(), href: Some(format!("/flows/{breadcrumb_flow_id}")) },
    ]);
  });

  rsx! {
    div { class: "flex h-full min-h-0 flex-col gap-3",
      div { class: "flex flex-wrap items-start justify-between gap-4 rounded-2xl border border-[var(--outline-variant)] bg-[var(--surface-container)] px-4 py-3.5",
        div { class: "min-w-0 flex-1",
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
            "{product_kind.label()}"
          }
          div { class: "mt-1.5 flex flex-wrap items-center gap-3",
            h1 { class: "min-w-0 text-[1.85rem] font-semibold leading-none text-[var(--on-surface)] truncate",
              "{document.title}"
            }
            div { class: "flex flex-wrap items-center gap-2 text-xs text-[var(--on-surface-variant)]",
              StatusPill { label: product_kind.badge_label().to_string() }
              StatusPill { label: format!("{node_count} nodes") }
              StatusPill { label: format!("{edge_count} edges") }
              StatusPill { label: validation_summary }
              if let Some(compile_state) = compile_state.clone() {
                StatusPill { label: compile_state.label }
              }
            }
          }
          if let Some(flow_notes) = flow_notes {
            p { class: "mt-2 max-w-3xl text-[13px] leading-5 text-[var(--on-surface-variant)]",
              "{flow_notes}"
            }
          }
          div { class: "mt-3 flex flex-wrap items-center gap-1.5",
            ProductSampleButton {
              label: "Workflow Sample".to_string(),
              flow_id: DEFAULT_FLOW_ID.to_string(),
              active: product_kind == FlowProductKind::Workflow,
            }
            ProductSampleButton {
              label: "LX Sample".to_string(),
              flow_id: DEFAULT_LX_FLOW_ID.to_string(),
              active: product_kind == FlowProductKind::Lx,
            }
            ProductSampleButton {
              label: "Mermaid Sample".to_string(),
              flow_id: DEFAULT_MERMAID_FLOW_ID.to_string(),
              active: product_kind == FlowProductKind::Mermaid,
            }
          }
        }
        div { class: "flex shrink-0 flex-col items-end gap-2",
          div { class: "flex flex-wrap justify-end gap-1.5",
            button {
              class: "{primary_action_button_class}",
              style: "{primary_action_button_style}",
              onclick: move |_| {
                  if let Err(error) = state.save(&save_persistence) {
                      report_action_error(&state, "save flow", error);
                  }
              },
              "Save"
            }
            button {
              class: "{secondary_action_button_class}",
              onclick: move |_| {
                  match state.save_as_new(&save_as_new_persistence) {
                      Ok(new_flow_id) => {
                          let _ = navigator
                              .push(Route::FlowDetail {
                                  flow_id: new_flow_id,
                              });
                      }
                      Err(error) => report_action_error(&state, "save flow copy", error),
                  }
              },
              "Save As New"
            }
            button {
              class: "{tertiary_action_button_class}",
              onclick: move |_| {
                  if let Err(error) = state.reset_to_sample(&reset_persistence) {
                      report_action_error(&state, "reset flow", error);
                  }
              },
              "Reset To Sample"
            }
            if !selection.is_empty() {
              button {
                class: "{secondary_action_button_class}",
                onclick: move |_| {
                    let _ = state
                        .dispatch(GraphCommand::Select {
                            selection: GraphSelection::empty(),
                        });
                },
                "Clear Selection"
              }
            }
          }
          if !selection.is_empty() || status_message.is_some() {
            div { class: "max-w-md text-right text-sm text-[var(--on-surface-variant)]",
              if !selection.is_empty() {
                div { class: "font-medium text-[var(--on-surface)]",
                  "{selection_summary}"
                }
              }
              if let Some(message) = status_message {
                div { class: if selection.is_empty() { "text-xs text-[var(--outline)]" } else { "mt-1 text-xs text-[var(--outline)]" },
                  "{message}"
                }
              }
            }
          }
        }
      }

      div { class: "flex min-h-0 flex-1",
        div { class: "relative flex min-h-0 flex-1 flex-col gap-3",
          div { class: "min-h-0 flex-1 rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-low)] overflow-hidden relative",
            GraphCanvas {
              document: document.clone(),
              templates: templates.clone(),
              diagnostics: diagnostics.clone(),
              run_snapshot: run_snapshot.clone(),
              canvas_size: state.current_canvas_size(),
              on_command: move |command: GraphCommand| dispatch_canvas_command(&mut state, command),
              on_editor_action: move |action: GraphEditorAction| state.apply_editor_action(&action),
              on_canvas_size: move |size: (f64, f64)| state.register_canvas_size(size.0, size.1),
              empty_title: product_kind.empty_title().to_string(),
              empty_message: product_kind.empty_message().to_string(),
              overlay_safe_area: graph_palette_safe_area(palette_is_open),
              button {
                class: "pointer-events-auto absolute left-4 top-4 z-30 rounded-full px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] transition-all hover:brightness-105",
                style: if palette_is_open { "border: 1px solid var(--graph-selection-border); background: var(--graph-selection-surface); color: var(--graph-selection-text); backdrop-filter: blur(14px);" } else { "border: 1px solid var(--graph-overlay-border); background: var(--graph-overlay-bg); color: var(--graph-overlay-text); backdrop-filter: blur(14px);" },
                onclick: move |_| {
                    let next_open = !*palette_open.peek();
                    palette_open.set(next_open);
                },
                if palette_is_open {
                  "Hide Nodes"
                } else {
                  "Nodes"
                }
              }
              if palette_is_open {
                button {
                  class: "pointer-events-auto absolute inset-0 z-20",
                  style: "background: var(--graph-overlay-scrim);",
                  onclick: move |_| palette_open.set(false),
                }
                aside {
                  class: "pointer-events-auto absolute left-4 top-16 bottom-4 z-30 flex w-[19rem] flex-col rounded-2xl p-4",
                  style: "border: 1px solid var(--graph-overlay-border); background: var(--graph-overlay-bg-strong); box-shadow: var(--graph-overlay-shadow); backdrop-filter: blur(18px);",
                  div { class: "flex items-start justify-between gap-3",
                    div {
                      div {
                        class: "text-[11px] font-mono uppercase tracking-[0.2em]",
                        style: "color: var(--graph-overlay-muted);",
                        "{product_kind.palette_title()}"
                      }
                      p {
                        class: "mt-1.5 max-w-xs text-[13px] leading-5",
                        style: "color: var(--graph-overlay-muted);",
                        "{product_kind.palette_description()}"
                      }
                    }
                    button {
                      class: "rounded-full px-2 py-1 text-[11px] font-semibold transition-colors hover:bg-[var(--surface-container-high)]",
                      style: "border: 1px solid var(--graph-overlay-border); color: var(--graph-overlay-text);",
                      onclick: move |_| palette_open.set(false),
                      "Close"
                    }
                  }
                  input {
                    class: "mt-3 w-full rounded-xl px-3 py-2.5 text-sm outline-none transition-colors focus:border-[var(--graph-selection-border)] focus:bg-[var(--graph-overlay-bg-strong)]",
                    style: "border: 1px solid var(--graph-overlay-border); background: var(--graph-overlay-bg); color: var(--graph-overlay-text);",
                    r#type: "text",
                    value: "{palette_query}",
                    placeholder: "Search node types",
                    oninput: move |evt| palette_query.set(evt.value()),
                  }
                  div { class: "mt-3 flex-1 overflow-y-auto pr-1",
                    if filtered_templates.is_empty() {
                      div {
                        class: "rounded-xl border border-dashed px-3 py-4 text-sm",
                        style: "border-color: var(--graph-overlay-border); color: var(--graph-overlay-muted);",
                        "No nodes match this query."
                      }
                    } else {
                      div { class: "flex flex-col gap-1.5",
                        for template in filtered_templates {
                          PaletteTemplateCard {
                            key: "{template.id}",
                            template: template.clone(),
                            on_add: move |template_id: String| {
                                let (width, height) = state.current_canvas_size();
                                if let Err(error) = state
                                    .insert_template_at_viewport_center(&template_id, width, height)
                                {
                                    let mut status_message = state.status_message;
                                    status_message.set(Some(format!("Failed to add node: {error}")));
                                } else {
                                    palette_open.set(false);
                                }
                            },
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
          if let Some(compile_state) = compile_state {
            CompileSurface { compile_state }
          }
          if !diagnostics.is_empty() {
            ValidationSurface {
              title: product_kind.diagnostics_title().to_string(),
              description: product_kind.diagnostics_description().to_string(),
              diagnostics,
            }
          }
        }
      }
    }
  }
}

#[component]
fn StatusPill(label: String) -> Element {
  rsx! {
    span {
      class: "rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-2.5 py-1 text-[11px] font-medium text-[var(--on-surface-variant)]",
      style: "box-shadow: inset 0 1px 0 color-mix(in srgb, var(--on-surface) 3%, transparent);",
      "{label}"
    }
  }
}

#[component]
fn ProductSampleButton(label: String, flow_id: String, active: bool) -> Element {
  let navigator = use_navigator();

  rsx! {
    button {
      class: "rounded-full border px-3 py-1.5 text-[11px] font-semibold transition-colors",
      style: if active { "border-color: color-mix(in srgb, var(--primary) 52%, transparent); background: color-mix(in srgb, var(--primary) 14%, transparent); color: var(--on-surface);" } else { "border-color: var(--outline-variant); background: var(--surface-container-high); color: var(--on-surface-variant);" },
      onclick: move |_| {
          let _ = navigator
              .push(Route::FlowDetail {
                  flow_id: flow_id.clone(),
              });
      },
      "{label}"
    }
  }
}

#[component]
fn PaletteTemplateCard(template: GraphNodeTemplate, on_add: EventHandler<String>) -> Element {
  let category = template.category.as_deref().map(category_label);
  let description = template.description.clone();
  let template_id = template.id.clone();

  rsx! {
    div { class: "rounded-xl border border-transparent bg-[var(--surface-container)] px-3 py-3 transition-colors hover:border-[var(--outline-variant)] hover:bg-[var(--surface-container-high)]",
      div { class: "flex items-start justify-between gap-3",
        div { class: "min-w-0",
          if let Some(category) = category {
            div { class: "mt-1 inline-flex rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-2 py-0.5 text-[9px] font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
              "{category}"
            }
          }
          div { class: "mt-2 text-[13px] font-semibold text-[var(--on-surface)] truncate",
            "{template.label}"
          }
        }
        button {
          class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-2.5 py-1.5 text-[11px] font-semibold text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
          onclick: move |_| on_add.call(template_id.clone()),
          "Add"
        }
      }
      if let Some(description) = description {
        p {
          class: "mt-2 text-[11px] leading-5 text-[var(--on-surface-variant)]",
          style: "display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;",
          "{description}"
        }
      }
    }
  }
}

#[component]
fn CompileSurface(compile_state: FlowCompileState) -> Element {
  let (surface_style, badge_style) = match compile_state.status {
    FlowCompileStatus::Ready => (
      "border: 1px solid color-mix(in srgb, var(--primary) 36%, transparent); background: color-mix(in srgb, var(--primary) 10%, var(--surface-container)); color: var(--on-surface);",
      "border: 1px solid color-mix(in srgb, var(--primary) 44%, transparent); background: color-mix(in srgb, var(--primary) 18%, transparent); color: var(--on-surface);",
    ),
    FlowCompileStatus::Blocked => (
      "border: 1px solid var(--graph-error-border); background: var(--graph-error-surface); color: var(--graph-error-text);",
      "border: 1px solid var(--graph-error-border); background: color-mix(in srgb, var(--graph-error-surface) 70%, transparent); color: var(--graph-error-text);",
    ),
  };

  rsx! {
    div { class: "rounded-xl p-4", style: "{surface_style}",
      div { class: "flex items-start justify-between gap-3",
        div {
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] opacity-80",
            "Compile State"
          }
          p { class: "mt-1 text-sm leading-6 opacity-90", "{compile_state.detail}" }
        }
        span {
          class: "rounded-full px-2.5 py-1 text-[11px] font-semibold",
          style: "{badge_style}",
          "{compile_state.label}"
        }
      }
    }
  }
}

#[component]
fn ValidationSurface(title: String, description: String, diagnostics: Vec<GraphWidgetDiagnostic>) -> Element {
  let error_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error).count();
  let warning_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Warning).count();

  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container)] p-4",
      div { class: "flex items-center justify-between gap-4",
        div {
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
            "{title}"
          }
          p { class: "mt-1 text-sm text-[var(--on-surface-variant)]", "{description}" }
        }
        div { class: "flex flex-wrap gap-2",
          StatusPill { label: format!("{error_count} errors") }
          StatusPill { label: format!("{warning_count} warnings") }
        }
      }
      div { class: "mt-4 flex max-h-48 flex-col gap-2 overflow-y-auto pr-1",
        for diagnostic in diagnostics {
          ValidationDiagnosticRow { key: "{diagnostic.id}", diagnostic }
        }
      }
    }
  }
}

#[component]
fn ValidationDiagnosticRow(diagnostic: GraphWidgetDiagnostic) -> Element {
  let mut state = use_flow_editor_state();
  let target = diagnostic.target.clone();
  let target_chip = target.as_ref().map(target_label);
  let source_chip = diagnostic.source.clone();
  let detail = diagnostic.detail.clone();
  let row_style = match diagnostic.severity {
    GraphWidgetDiagnosticSeverity::Error => {
      "border: 1px solid var(--graph-error-border); background: var(--graph-error-surface); color: var(--graph-error-text);"
    },
    GraphWidgetDiagnosticSeverity::Warning => {
      "border: 1px solid var(--graph-warning-border); background: var(--graph-warning-surface); color: var(--graph-warning-text);"
    },
    GraphWidgetDiagnosticSeverity::Info => "border: 1px solid var(--graph-info-border); background: var(--graph-info-surface); color: var(--graph-info-text);",
  };

  rsx! {
    button {
      class: "rounded-xl px-3 py-2 text-left text-sm transition-all hover:brightness-105",
      style: "{row_style}",
      onclick: move |_| {
          if let Some(selection) = selection_for_target(target.clone()) {
              let _ = state.dispatch(GraphCommand::Select { selection });
          }
      },
      div { class: "flex items-start justify-between gap-3",
        div { class: "min-w-0 flex-1",
          span { class: "font-medium", "{diagnostic.message}" }
          if let Some(detail) = detail {
            p { class: "mt-1 text-xs leading-5 opacity-80", "{detail}" }
          }
        }
        div { class: "flex shrink-0 flex-wrap items-center justify-end gap-1.5",
          if let Some(source_chip) = source_chip {
            span {
              class: "rounded-full border px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.14em]",
              style: "border-color: var(--graph-overlay-border); color: var(--graph-overlay-text);",
              "{source_chip}"
            }
          }
          if let Some(target_chip) = target_chip {
            span {
              class: "rounded-full border px-2 py-1 text-[11px] font-mono",
              style: "border-color: var(--graph-overlay-border); color: var(--graph-overlay-text);",
              "{target_chip}"
            }
          }
        }
      }
    }
  }
}

const GRAPH_PALETTE_CHROME_TOP_SAFE_AREA: f64 = 56.0;
const GRAPH_PALETTE_BUTTON_SAFE_AREA_LEFT: f64 = 112.0;
const GRAPH_PALETTE_DRAWER_WIDTH: f64 = 19.0 * 16.0;
const GRAPH_PALETTE_EDGE_INSET: f64 = 16.0;

fn graph_palette_safe_area(palette_open: bool) -> GraphCanvasSafeArea {
  GraphCanvasSafeArea {
    top: GRAPH_PALETTE_CHROME_TOP_SAFE_AREA,
    right: 0.0,
    bottom: 0.0,
    left: if palette_open { GRAPH_PALETTE_EDGE_INSET + GRAPH_PALETTE_DRAWER_WIDTH + GRAPH_PALETTE_EDGE_INSET } else { GRAPH_PALETTE_BUTTON_SAFE_AREA_LEFT },
  }
}

fn selection_summary(selection: &GraphSelection) -> String {
  if selection.is_empty() {
    return "No selection".to_string();
  }
  if selection.node_ids.len() == 1 && selection.edge_ids.is_empty() {
    return "1 node selected".to_string();
  }
  if selection.edge_ids.len() == 1 && selection.node_ids.is_empty() {
    return "1 edge selected".to_string();
  }
  format!("{} nodes and {} edges selected", selection.node_ids.len(), selection.edge_ids.len())
}

fn category_label(category: &str) -> String {
  category
    .replace('_', " ")
    .split_whitespace()
    .map(|word| {
      let mut chars = word.chars();
      match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
      }
    })
    .collect::<Vec<_>>()
    .join(" ")
}

fn palette_matches(template: &GraphNodeTemplate, query: &str) -> bool {
  if query.is_empty() {
    return true;
  }
  template.label.to_lowercase().contains(query)
    || template.id.to_lowercase().contains(query)
    || template.category.as_ref().is_some_and(|category| category.to_lowercase().contains(query))
    || template.description.as_ref().is_some_and(|description| description.to_lowercase().contains(query))
}

fn target_label(target: &GraphEntityRef) -> String {
  match target {
    GraphEntityRef::Node(node_id) => format!("node:{node_id}"),
    GraphEntityRef::Edge(edge_id) => format!("edge:{edge_id}"),
  }
}

fn selection_for_target(target: Option<GraphEntityRef>) -> Option<GraphSelection> {
  match target? {
    GraphEntityRef::Node(node_id) => Some(GraphSelection::single_node(node_id)),
    GraphEntityRef::Edge(edge_id) => Some(GraphSelection::single_edge(edge_id)),
  }
}

fn dispatch_canvas_command(state: &mut super::controller::FlowEditorState, command: GraphCommand) {
  let action = match &command {
    GraphCommand::AddNode { .. } => "add node",
    GraphCommand::RemoveNode { .. } => "remove node",
    GraphCommand::MoveNode { .. } => "move node",
    GraphCommand::Select { .. } => "update selection",
    GraphCommand::ConnectPorts { .. } => "connect ports",
    GraphCommand::DisconnectEdge { .. } => "remove edge",
    GraphCommand::SetViewport { .. } => "update viewport",
    GraphCommand::UpdateField { .. } => "update field",
    GraphCommand::DeleteSelection => "delete selection",
  };
  if let Err(error) = state.dispatch(command) {
    report_action_error(state, action, error);
  }
}

fn report_action_error(state: &super::controller::FlowEditorState, action: &str, error: impl std::fmt::Display) {
  let mut status_message = state.status_message;
  status_message.set(Some(format!("Failed to {action}: {error}")));
}
