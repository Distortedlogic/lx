use dioxus::prelude::*;
use dioxus_widget_bridge::use_ts_widget;

use crate::contexts::breadcrumb::BreadcrumbEntry;
use crate::graph_editor::catalog::GraphNodeTemplate;
use crate::graph_editor::commands::GraphCommand;
use crate::graph_editor::model::{GraphEntityRef, GraphSelection};
use crate::graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity, GraphWidgetEvent};
use crate::routes::Route;

use super::controller::use_flow_editor_state;
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

  let flow_id = state.flow_id.read().clone();
  let document = state.document.read().clone();
  let templates = state.templates.read().clone();
  let diagnostics = state.diagnostics.read().clone();
  let selection = state.selection.read().clone();
  let validation_count = *state.validation_count.read();
  let status_message = state.status_message.read().clone();
  let title = document.title.clone();
  let breadcrumb_title = title.clone();
  let breadcrumb_flow_id = flow_id.clone();
  let node_count = document.nodes.len();
  let edge_count = document.edges.len();
  let selection_summary = selection_summary(&selection);
  let template_count = templates.len();
  let viewport = document.viewport;
  let query = palette_query.read().trim().to_lowercase();
  let filtered_templates: Vec<_> = templates.iter().filter(|template| palette_matches(template, &query)).cloned().collect();

  use_effect(move || {
    breadcrumb_state.set(vec![
      BreadcrumbEntry { label: "Flows".into(), href: Some("/flows".into()) },
      BreadcrumbEntry { label: breadcrumb_title.clone(), href: Some(format!("/flows/{breadcrumb_flow_id}")) },
    ]);
  });

  rsx! {
    div { class: "flex h-full min-h-0 flex-col gap-4",
      div { class: "flex items-start justify-between gap-4 rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container)] px-4 py-3",
        div { class: "min-w-0",
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
            "Workflow Editor"
          }
          h1 { class: "text-2xl font-semibold text-[var(--on-surface)] truncate",
            "{document.title}"
          }
          div { class: "mt-2 flex flex-wrap items-center gap-2 text-xs text-[var(--on-surface-variant)]",
            StatusPill { label: format!("flow id: {flow_id}") }
            StatusPill { label: format!("{node_count} nodes") }
            StatusPill { label: format!("{edge_count} edges") }
            StatusPill { label: format!("{template_count} templates") }
            StatusPill { label: format!("{validation_count} validation issues") }
          }
        }
        div { class: "flex shrink-0 flex-col items-end gap-3",
          div { class: "flex flex-wrap justify-end gap-2",
            button {
              class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-medium text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
              onclick: move |_| {
                  if let Err(error) = state.save(&save_persistence) {
                      report_action_error(&state, "save flow", error);
                  }
              },
              "Save"
            }
            button {
              class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-medium text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
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
              class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-medium text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
              onclick: move |_| {
                  if let Err(error) = state.reset_to_sample(&reset_persistence) {
                      report_action_error(&state, "reset flow", error);
                  }
              },
              "Reset To Sample"
            }
            button {
              class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-medium text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
              onclick: move |_| {
                  let _ = state
                      .dispatch(GraphCommand::Select {
                          selection: GraphSelection::empty(),
                      });
              },
              "Clear Selection"
            }
          }
          div { class: "text-right text-sm text-[var(--on-surface-variant)]",
            div { class: "font-medium text-[var(--on-surface)]", "{selection_summary}" }
            div { class: "mt-1",
              "Viewport: x={viewport.pan_x:.0}, y={viewport.pan_y:.0}, zoom={viewport.zoom:.2}"
            }
            if let Some(message) = status_message {
              div { class: "mt-2 text-xs text-[var(--outline)]", "{message}" }
            }
          }
        }
      }

      div { class: "flex min-h-0 flex-1 gap-4",
        aside { class: "flex w-72 shrink-0 flex-col rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container)] p-4",
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
            "Node Palette"
          }
          p { class: "mt-2 text-sm text-[var(--on-surface-variant)]",
            "Insert workflow steps into the current viewport. New nodes land in the live canvas center and become the active selection."
          }
          input {
            class: "mt-4 w-full rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2 text-sm text-[var(--on-surface)] outline-none",
            r#type: "text",
            value: "{palette_query}",
            placeholder: "Search nodes",
            oninput: move |evt| palette_query.set(evt.value()),
          }
          div { class: "mt-4 flex-1 overflow-y-auto pr-1",
            if filtered_templates.is_empty() {
              div { class: "rounded-xl border border-dashed border-[var(--outline-variant)] px-3 py-4 text-sm text-[var(--on-surface-variant)]",
                "No templates match this query."
              }
            } else {
              div { class: "flex flex-col gap-2",
                for template in filtered_templates {
                  PaletteTemplateCard {
                    key: "{template.id}",
                    template: template.clone(),
                    on_add: move |template_id: String| {
                        let canvas_host_id = state.current_canvas_host();
                        let mut flow_state = state;
                        spawn(async move {
                            let (width, height) = measure_canvas_host(canvas_host_id.as_deref())
                                .await
                                .unwrap_or((1200.0, 760.0));
                            if let Err(error) = flow_state
                                .insert_template_at_viewport_center(&template_id, width, height)
                            {
                                let mut status_message = flow_state.status_message;
                                status_message.set(Some(format!("Failed to add node: {error}")));
                            }
                        });
                    },
                  }
                }
              }
            }
          }
        }

        div { class: "flex min-h-0 flex-1 flex-col gap-4",
          div { class: "min-h-0 flex-1 rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-low)] overflow-hidden relative",
            FlowEditorCanvas {}
          }
          ValidationSurface { diagnostics }
        }
      }
    }
  }
}

#[component]
fn StatusPill(label: String) -> Element {
  rsx! {
    span { class: "rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-2.5 py-1 text-[11px] font-medium text-[var(--on-surface-variant)]",
      "{label}"
    }
  }
}

#[component]
fn PaletteTemplateCard(template: GraphNodeTemplate, on_add: EventHandler<String>) -> Element {
  let port_count = template.ports.len();
  let field_count = template.fields.len();
  let category = template.category.clone();
  let description = template.description.clone();
  let template_id = template.id.clone();

  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-2.5",
      div { class: "flex items-center justify-between gap-3",
        div { class: "min-w-0",
          div { class: "font-medium text-[var(--on-surface)] truncate", "{template.label}" }
          if let Some(category) = category {
            div { class: "text-[11px] uppercase tracking-wide text-[var(--outline)]",
              "{category}"
            }
          }
        }
        span { class: "text-xs text-[var(--outline)]", "{port_count}p / {field_count}f" }
      }
      if let Some(description) = description {
        p { class: "mt-2 text-xs leading-5 text-[var(--on-surface-variant)]",
          "{description}"
        }
      }
      div { class: "mt-3 flex justify-end",
        button {
          class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-highest)] px-3 py-1.5 text-xs font-semibold text-[var(--on-surface)]",
          onclick: move |_| on_add.call(template_id.clone()),
          "Add Node"
        }
      }
    }
  }
}

#[component]
fn FlowEditorCanvas() -> Element {
  let mut state = use_flow_editor_state();
  let snapshot = state.widget_snapshot();
  let (element_id, widget) = use_ts_widget("dag-editor", snapshot.clone());
  let canvas_element_id = element_id.clone();

  use_effect(move || {
    state.register_canvas_host(canvas_element_id.clone());
    widget.send_update(snapshot.clone());
  });

  use_future(move || async move {
    loop {
      let Ok(event) = widget.recv::<GraphWidgetEvent>().await else { break };
      let snapshot = state.apply_widget_event(event);
      widget.send_update(snapshot);
    }
  });

  rsx! {
    div {
      id: "{element_id}",
      class: "h-full w-full bg-[var(--surface-container-lowest)]",
    }
  }
}

#[component]
fn ValidationSurface(diagnostics: Vec<GraphWidgetDiagnostic>) -> Element {
  let error_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error).count();
  let warning_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Warning).count();

  rsx! {
    div { class: "rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container)] p-4",
      div { class: "flex items-center justify-between gap-4",
        div {
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
            "Validation"
          }
          p { class: "mt-1 text-sm text-[var(--on-surface-variant)]",
            "Workflow-specific graph checks run after each mutation."
          }
        }
        div { class: "flex flex-wrap gap-2",
          StatusPill { label: format!("{error_count} errors") }
          StatusPill { label: format!("{warning_count} warnings") }
        }
      }
      if diagnostics.is_empty() {
        div { class: "mt-4 rounded-xl border border-emerald-500/20 bg-emerald-500/8 px-3 py-3 text-sm text-emerald-200",
          "No validation issues."
        }
      } else {
        div { class: "mt-4 flex max-h-48 flex-col gap-2 overflow-y-auto pr-1",
          for diagnostic in diagnostics {
            ValidationDiagnosticRow { key: "{diagnostic.id}", diagnostic }
          }
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
  let row_class = match diagnostic.severity {
    GraphWidgetDiagnosticSeverity::Error => "rounded-xl border border-red-500/30 bg-red-500/8 px-3 py-2 text-sm text-red-100",
    GraphWidgetDiagnosticSeverity::Warning => "rounded-xl border border-amber-500/30 bg-amber-500/8 px-3 py-2 text-sm text-amber-100",
    GraphWidgetDiagnosticSeverity::Info => "rounded-xl border border-sky-500/30 bg-sky-500/8 px-3 py-2 text-sm text-sky-100",
  };

  rsx! {
    button {
      class: "{row_class} text-left transition-colors hover:bg-white/8",
      onclick: move |_| {
          if let Some(selection) = selection_for_target(target.clone()) {
              let _ = state.dispatch(GraphCommand::Select { selection });
          }
      },
      div { class: "flex items-center justify-between gap-3",
        span { class: "font-medium", "{diagnostic.message}" }
        if let Some(target_chip) = target_chip {
          span { class: "rounded-full border border-white/10 px-2 py-1 text-[11px] font-mono text-white/80",
            "{target_chip}"
          }
        }
      }
    }
  }
}

fn selection_summary(selection: &GraphSelection) -> String {
  if selection.is_empty() {
    return "No selection yet".to_string();
  }
  if selection.node_ids.len() == 1 && selection.edge_ids.is_empty() {
    return format!("Selected node {}", selection.node_ids[0]);
  }
  if selection.edge_ids.len() == 1 && selection.node_ids.is_empty() {
    return format!("Selected edge {}", selection.edge_ids[0]);
  }
  format!("Selected {} nodes and {} edges", selection.node_ids.len(), selection.edge_ids.len())
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

async fn measure_canvas_host(element_id: Option<&str>) -> Option<(f64, f64)> {
  let element_id = element_id?;
  let js = format!(
    "(function() {{ var el = document.getElementById('{element_id}'); if (!el) return JSON.stringify({{width: 0, height: 0}}); var rect = el.getBoundingClientRect(); return JSON.stringify({{width: rect.width, height: rect.height}}); }})()"
  );
  let result = document::eval(&js).await.ok()?;
  let payload = result.to_string();
  let payload = payload.trim_matches('"');
  let value = serde_json::from_str::<serde_json::Value>(payload).ok()?;
  Some((value["width"].as_f64().unwrap_or(0.0), value["height"].as_f64().unwrap_or(0.0)))
}

fn report_action_error(state: &super::controller::FlowEditorState, action: &str, error: impl std::fmt::Display) {
  let mut status_message = state.status_message;
  status_message.set(Some(format!("Failed to {action}: {error}")));
}
