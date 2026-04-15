use dioxus::prelude::*;

use crate::contexts::breadcrumb::BreadcrumbEntry;
use crate::graph_editor::catalog::{GraphNodeTemplate, node_template};
use crate::graph_editor::commands::GraphCommand;
use crate::graph_editor::model::{GraphNode, GraphSelection};

use super::controller::use_flow_editor_state;

#[component]
pub fn FlowWorkspace() -> Element {
  let mut state = use_flow_editor_state();
  let breadcrumb_state = use_context::<crate::contexts::breadcrumb::BreadcrumbState>();

  let flow_id = state.flow_id.read().clone();
  let document = state.document.read().clone();
  let templates = state.templates.read().clone();
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
                  let _ = state
                      .dispatch(GraphCommand::Select {
                          selection: GraphSelection::single_node("fetch"),
                      });
              },
              "Focus Fetch"
            }
            button {
              class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1.5 text-xs font-medium text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
              onclick: move |_| {
                  let _ = state
                      .dispatch(GraphCommand::Select {
                          selection: GraphSelection::single_node("feed"),
                      });
              },
              "Focus Feed"
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
        aside { class: "w-72 shrink-0 rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container)] p-4",
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
            "Node Palette"
          }
          p { class: "mt-2 text-sm text-[var(--on-surface-variant)]",
            "This route-first shell reserves the insertion rail now. The interactive insert behavior lands after the widget bridge and inspector units."
          }
          div { class: "mt-4 flex flex-col gap-2",
            for template in templates.iter() {
              PaletteTemplateCard { template: template.clone() }
            }
          }
        }

        div { class: "min-h-0 flex-1 rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-low)] overflow-auto relative",
          div {
            class: "absolute inset-0 opacity-40",
            style: "background-image: radial-gradient(circle at 1px 1px, color-mix(in srgb, var(--outline) 55%, transparent) 1px, transparent 0); background-size: 28px 28px;",
          }
          div { class: "relative min-w-[1800px] min-h-[820px] p-8",
            for node in document.nodes.iter() {
              FlowPreviewNode {
                node: node.clone(),
                template: node_template(&templates, &node.template_id).cloned(),
                is_selected: selection.node_ids.iter().any(|selected| selected == &node.id),
              }
            }

            div { class: "absolute bottom-6 right-6 w-64 rounded-xl border border-dashed border-[var(--outline)]/70 bg-[var(--surface-container)]/95 p-3 text-sm text-[var(--on-surface-variant)] shadow-sm",
              div { class: "font-medium text-[var(--on-surface)]", "Canvas Host" }
              p { class: "mt-1",
                "This surface is the reserved mount point for the `dag-editor` widget. Unit 02 keeps it route-first and stateful without going through `CanvasView`."
              }
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
    span { class: "rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-2.5 py-1 text-[11px] font-medium text-[var(--on-surface-variant)]",
      "{label}"
    }
  }
}

#[component]
fn PaletteTemplateCard(template: GraphNodeTemplate) -> Element {
  let port_count = template.ports.len();
  let field_count = template.fields.len();
  let category = template.category.clone();
  let description = template.description.clone();

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
    }
  }
}

#[component]
fn FlowPreviewNode(node: GraphNode, template: Option<GraphNodeTemplate>, is_selected: bool) -> Element {
  let card_class = if is_selected {
    "absolute w-60 rounded-2xl border border-[var(--primary)] bg-[var(--surface-container)] shadow-lg ring-2 ring-[var(--primary)]/25"
  } else {
    "absolute w-60 rounded-2xl border border-[var(--outline-variant)] bg-[var(--surface-container)] shadow-sm"
  };
  let template_label = template.as_ref().map_or("Node", |entry| entry.label.as_str());
  let property_count = node.properties.as_object().map_or(0, |props| props.len());
  let style = format!("left: {}px; top: {}px;", node.position.x, node.position.y);

  rsx! {
    div { class: "{card_class}", style: "{style}",
      div { class: "border-b border-[var(--outline-variant)] px-4 py-3",
        div { class: "flex items-start justify-between gap-3",
          div { class: "min-w-0",
            div { class: "text-[11px] font-mono uppercase tracking-[0.18em] text-[var(--outline)]",
              "{template_label}"
            }
            div { class: "mt-1 text-base font-semibold text-[var(--on-surface)] truncate",
              "{node.label.clone().unwrap_or_else(|| node.id.clone())}"
            }
          }
          span { class: "rounded-full bg-[var(--surface-container-high)] px-2 py-1 text-[11px] text-[var(--on-surface-variant)]",
            "{property_count} fields"
          }
        }
      }
      div { class: "px-4 py-3 text-sm text-[var(--on-surface-variant)]",
        div { class: "flex items-center justify-between gap-3",
          span { "id" }
          span { class: "font-mono text-xs text-[var(--outline)]", "{node.id}" }
        }
        div { class: "mt-2 flex items-center justify-between gap-3",
          span { "template" }
          span { class: "font-mono text-xs text-[var(--outline)]", "{node.template_id}" }
        }
        if let Some(template) = template {
          div { class: "mt-3 flex flex-wrap gap-1.5",
            for port in template.ports.iter() {
              span { class: "rounded-full border border-[var(--outline-variant)] px-2 py-1 text-[11px]",
                "{port.label}"
              }
            }
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
