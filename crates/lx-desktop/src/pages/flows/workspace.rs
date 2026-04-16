use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use tokio::time::sleep;

use crate::contexts::breadcrumb::BreadcrumbEntry;
use crate::graph_editor::catalog::{GraphNodeTemplate, GraphPortTemplate, PortDirection, node_template};
use crate::graph_editor::commands::GraphCommand;
use crate::graph_editor::model::{GraphDocument, GraphEntityRef, GraphNode, GraphPoint, GraphPortRef, GraphSelection, GraphViewport};
use crate::graph_editor::protocol::{GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};
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
  let primary_action_button_class =
    "rounded-xl border border-sky-400/30 bg-sky-500/10 px-3 py-2 text-xs font-semibold text-sky-100 transition-colors hover:bg-sky-500/16";
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
            "Workflow"
          }
          div { class: "mt-1.5 flex flex-wrap items-center gap-3",
            h1 { class: "min-w-0 text-[1.85rem] font-semibold leading-none text-[var(--on-surface)] truncate",
              "{document.title}"
            }
            div { class: "flex flex-wrap items-center gap-2 text-xs text-[var(--on-surface-variant)]",
              StatusPill { label: format!("{node_count} nodes") }
              StatusPill { label: format!("{edge_count} edges") }
              StatusPill { label: validation_summary }
            }
          }
          if let Some(flow_notes) = flow_notes {
            p { class: "mt-2 max-w-3xl text-[13px] leading-5 text-[var(--on-surface-variant)]",
              "{flow_notes}"
            }
          }
          div { class: "mt-2 text-[11px] font-mono text-[var(--outline)]",
            "{flow_id}"
          }
        }
        div { class: "flex shrink-0 flex-col items-end gap-2",
          div { class: "flex flex-wrap justify-end gap-1.5",
            button {
              class: "{primary_action_button_class}",
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
                div { class: "font-medium text-[var(--on-surface)]", "{selection_summary}" }
              }
              if let Some(message) = status_message {
                div { class: if selection.is_empty() { "text-xs text-[var(--outline)]" } else { "mt-1 text-xs text-[var(--outline)]" }, "{message}" }
              }
            }
          }
        }
      }

      div { class: "flex min-h-0 flex-1 gap-3",
        aside { class: "flex w-64 shrink-0 flex-col rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container-low)] p-3.5",
          div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
            "Node Palette"
          }
          p { class: "mt-1.5 text-[13px] leading-5 text-[var(--on-surface-variant)]",
            "Add steps into the canvas. New nodes land at the viewport center."
          }
          input {
            class: "mt-3 w-full rounded-xl border border-[var(--outline-variant)] bg-[var(--surface-container)] px-3 py-2.5 text-sm text-[var(--on-surface)] outline-none transition-colors focus:border-sky-400/40 focus:bg-[var(--surface-container-high)]",
            r#type: "text",
            value: "{palette_query}",
            placeholder: "Search node types",
            oninput: move |evt| palette_query.set(evt.value()),
          }
          div { class: "mt-3 flex-1 overflow-y-auto pr-1",
            if filtered_templates.is_empty() {
              div { class: "rounded-xl border border-dashed border-[var(--outline-variant)] px-3 py-4 text-sm text-[var(--on-surface-variant)]",
                "No templates match this query."
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
                        }
                    },
                  }
                }
              }
            }
          }
        }

        div { class: "flex min-h-0 flex-1 flex-col gap-3",
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
      style: "box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.03);",
      "{label}"
    }
  }
}

#[component]
fn PaletteTemplateCard(template: GraphNodeTemplate, on_add: EventHandler<String>) -> Element {
  let port_count = template.ports.len();
  let field_count = template.fields.len();
  let category = template.category.as_deref().map(category_label);
  let description = template.description.clone();
  let template_id = template.id.clone();
  let meta_label = format!("{field_count} field{}", if field_count == 1 { "" } else { "s" });
  let port_label = format!("{port_count} port{}", if port_count == 1 { "" } else { "s" });

  rsx! {
    div { class: "rounded-xl border border-transparent bg-[var(--surface-container)] px-3 py-3 transition-colors hover:border-[var(--outline-variant)] hover:bg-[var(--surface-container-high)]",
      div { class: "flex items-start justify-between gap-3",
        div { class: "min-w-0",
          div { class: "font-medium text-[13px] text-[var(--on-surface)] truncate", "{template.label}" }
          if let Some(category) = category {
            div { class: "mt-1 inline-flex rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-2 py-0.5 text-[9px] font-semibold uppercase tracking-[0.18em] text-[var(--outline)]",
              "{category}"
            }
          }
        }
        span { class: "text-[10px] font-medium uppercase tracking-[0.14em] text-[var(--outline)]",
          "{port_label}"
        }
      }
      if let Some(description) = description {
        p { class: "mt-2 text-[11px] leading-5 text-[var(--on-surface-variant)]",
          style: "display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden;",
          "{description}"
        }
      }
      div { class: "mt-2.5 flex items-center justify-between gap-3",
        span { class: "text-[10px] uppercase tracking-[0.14em] text-[var(--outline)]", "{meta_label}" }
        button {
          class: "rounded-lg border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-2.5 py-1.5 text-[11px] font-semibold text-[var(--on-surface)] transition-colors hover:bg-[var(--surface-container-highest)]",
          onclick: move |_| on_add.call(template_id.clone()),
          "Add"
        }
      }
    }
  }
}

#[component]
fn FlowEditorCanvas() -> Element {
  let mut state = use_flow_editor_state();
  let flow_id = state.flow_id.read().clone();
  let document = state.document.read().clone();
  let templates = state.templates.read().clone();
  let diagnostics = state.diagnostics.read().clone();
  let selection = state.selection.read().clone();
  let mut scene_element = use_signal(|| Option::<Rc<MountedData>>::None);
  let scene_rect = use_signal(|| Option::<SceneRect>::None);
  let mut viewport_preview = use_signal(|| Option::<GraphViewport>::None);
  let mut node_preview_positions = use_signal(HashMap::<String, GraphPoint>::new);
  let mut drag_state = use_signal(|| Option::<DragState>::None);
  let mut pan_state = use_signal(|| Option::<PanState>::None);
  let mut connection_state = use_signal(|| Option::<ConnectionState>::None);
  let wheel_revision = use_signal(|| 0u64);
  let auto_framed_flow = use_signal(|| Option::<String>::None);

  let nodes = document.nodes.clone();
  let edges = document.edges.clone();
  let displayed_viewport = viewport_preview.read().unwrap_or(document.viewport);
  let preview_positions = node_preview_positions.read().clone();
  let is_dragging_node = drag_state.read().is_some();
  let is_panning = pan_state.read().is_some();
  let connection_preview = connection_state.read().clone();
  let (scene_width, scene_height) = state.current_canvas_size();
  let (world_width, world_height) = canvas_world_size(&document, &templates, &preview_positions);
  let grid_size = (24.0 * displayed_viewport.zoom).max(18.0);
  let major_grid_size = grid_size * 6.0;
  let error_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error).count();
  let warning_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Warning).count();
  let issue_summary = format!("{error_count} errors / {warning_count} warnings");
  let issue_badge_style = if error_count > 0 {
    "border: 1px solid rgba(248, 113, 113, 0.26); background: rgba(127, 29, 29, 0.42); color: #fecaca; backdrop-filter: blur(16px);"
  } else {
    "border: 1px solid rgba(251, 191, 36, 0.24); background: rgba(120, 53, 15, 0.38); color: #fde68a; backdrop-filter: blur(16px);"
  };
  let selection_badge = selection_summary(&selection);
  let zoom_label = format!("{:.0}% zoom", displayed_viewport.zoom * 100.0);
  let scene_class = if connection_preview.is_some() {
    "relative flex-1 min-h-0 overflow-hidden outline-none cursor-crosshair"
  } else if is_panning || is_dragging_node {
    "relative flex-1 min-h-0 overflow-hidden outline-none cursor-grabbing"
  } else {
    "relative flex-1 min-h-0 overflow-hidden outline-none cursor-grab"
  };
  let interaction_hint = if connection_preview.is_some() {
    "Release on an input port to connect."
  } else if !selection.is_empty() {
    "Drag to arrange. Delete removes the current selection."
  } else {
    "Pan, zoom, and drag from outputs to connect."
  };
  let show_interaction_hint = selection.is_empty() && !is_panning && !is_dragging_node;
  let canvas_badge_class = "rounded-full px-3 py-1.5 text-[11px] font-medium";
  let canvas_badge_style = "border: 1px solid rgba(255, 255, 255, 0.10); background: rgba(7, 12, 18, 0.74); color: #9fb0c2; backdrop-filter: blur(16px);";
  let grid_overlay_style = format!(
    "background-image: linear-gradient(rgba(255, 255, 255, 0.018) 1px, transparent 1px), linear-gradient(90deg, rgba(255, 255, 255, 0.018) 1px, transparent 1px), linear-gradient(rgba(119, 136, 153, 0.055) 1px, transparent 1px), linear-gradient(90deg, rgba(119, 136, 153, 0.055) 1px, transparent 1px); background-size: {grid_size}px {grid_size}px, {grid_size}px {grid_size}px, {major_grid_size}px {major_grid_size}px, {major_grid_size}px {major_grid_size}px; background-position: {}px {}px, {}px {}px, {}px {}px, {}px {}px;",
    displayed_viewport.pan_x,
    displayed_viewport.pan_y,
    displayed_viewport.pan_x,
    displayed_viewport.pan_y,
    displayed_viewport.pan_x,
    displayed_viewport.pan_y,
    displayed_viewport.pan_x,
    displayed_viewport.pan_y,
  );
  let renderable_edges: Vec<_> = edges
    .iter()
    .filter_map(|edge| {
      let from_node = nodes.iter().find(|node| node.id == edge.from.node_id)?;
      let to_node = nodes.iter().find(|node| node.id == edge.to.node_id)?;
      let from_point = port_world_point_for_displayed_node(&templates, from_node, &preview_positions, &edge.from.port_id)?;
      let to_point = port_world_point_for_displayed_node(&templates, to_node, &preview_positions, &edge.to.port_id)?;
      Some(RenderableEdge {
        id: edge.id.clone(),
        label: edge.label.clone(),
        from: from_point,
        to: to_point,
        is_selected: selection.edge_ids.iter().any(|selected| selected == &edge.id),
        has_error: diagnostics.iter().any(|diagnostic| {
          diagnostic.severity == GraphWidgetDiagnosticSeverity::Error && matches!(diagnostic.target, Some(GraphEntityRef::Edge(ref id)) if id == &edge.id)
        }),
      })
    })
    .collect();
  let can_delete_selection = !selection.node_ids.is_empty() || !selection.edge_ids.is_empty();

  let mut finish_interaction = {
    let mut drag_state = drag_state;
    let mut pan_state = pan_state;
    let mut connection_state = connection_state;
    let mut viewport_preview = viewport_preview;
    let mut node_preview_positions = node_preview_positions;
    move || {
      connection_state.set(None);

      let current_drag = drag_state.peek().clone();
      if let Some(drag) = current_drag {
        let final_position =
          node_preview_positions.peek().get(&drag.node_id).copied().or_else(|| state.document.read().node(&drag.node_id).map(|node| node.position));
        if let Some(position) = final_position
          && drag.moved
          && let Err(error) = state.dispatch(GraphCommand::MoveNode { node_id: drag.node_id.clone(), position })
        {
          report_action_error(&state, "move node", error);
        }
        node_preview_positions.write().remove(&drag.node_id);
        drag_state.set(None);
        return;
      }

      let current_pan = *pan_state.peek();
      if let Some(pan) = current_pan {
        if pan.moved {
          let current_viewport = *viewport_preview.peek();
          if let Some(viewport) = current_viewport
            && let Err(error) = state.dispatch(GraphCommand::SetViewport { viewport })
          {
            report_action_error(&state, "update viewport", error);
          }
        } else if !state.selection.read().is_empty()
          && let Err(error) = state.dispatch(GraphCommand::Select { selection: GraphSelection::empty() })
        {
          report_action_error(&state, "clear selection", error);
        }
        pan_state.set(None);
        viewport_preview.set(None);
      }
    }
  };

  {
    let mut auto_fit_state = state;
    let mut auto_framed_flow = auto_framed_flow;
    let flow_id = flow_id.clone();
    let document = document.clone();
    let templates = templates.clone();
    use_effect(move || {
      if document.nodes.is_empty() || scene_width < 10.0 || scene_height < 10.0 {
        return;
      }
      if auto_framed_flow.read().as_ref() == Some(&flow_id) {
        return;
      }
      auto_framed_flow.set(Some(flow_id.clone()));
      if viewport_needs_fit(&document, &templates, scene_width, scene_height)
        && let Some(viewport) = fit_viewport(&document, &templates, scene_width, scene_height)
        && let Err(error) = auto_fit_state.dispatch(GraphCommand::SetViewport { viewport })
      {
        report_action_error(&auto_fit_state, "frame canvas", error);
      }
    });
  }

  rsx! {
    div {
      class: "relative flex h-full min-h-0 flex-col",
      style: "background: linear-gradient(180deg, #0d1621 0%, #091018 100%); color: #e8edf2;",
      div {
        class: "pointer-events-none absolute inset-0",
        style: "background: radial-gradient(circle at top left, rgba(56, 189, 248, 0.07) 0%, transparent 34%), radial-gradient(circle at bottom right, rgba(34, 197, 94, 0.05) 0%, transparent 28%);"
      }
      div {
        class: "pointer-events-none absolute right-4 top-4 z-10 flex max-w-[55%] flex-wrap items-center justify-end gap-2",
        if !diagnostics.is_empty() {
          span { class: "{canvas_badge_class}", style: "{issue_badge_style}", "{issue_summary}" }
        }
        if !selection.is_empty() {
          span { class: "{canvas_badge_class}", style: "{canvas_badge_style}", "{selection_badge}" }
        }
        span { class: "{canvas_badge_class}", style: "{canvas_badge_style}", "{zoom_label}" }
        if !nodes.is_empty() {
          button {
            class: "pointer-events-auto rounded-full border border-white/10 px-3 py-1.5 text-[11px] font-semibold text-white transition-colors",
            style: "background: rgba(7, 12, 18, 0.8); backdrop-filter: blur(16px);",
            onclick: move |_| {
                let (width, height) = state.current_canvas_size();
                let document = state.document.read().clone();
                let templates = state.templates.read().clone();
                if let Some(viewport) = fit_viewport(&document, &templates, width, height)
                    && let Err(error) = state.dispatch(GraphCommand::SetViewport { viewport })
                {
                    report_action_error(&state, "fit canvas", error);
                }
            },
            "Fit View"
          }
        }
      }

      div {
        class: "{scene_class}",
        tabindex: "0",
        onmounted: move |evt: MountedEvent| {
            let element = evt.data();
            scene_element.set(Some(element));
            refresh_scene_metrics(scene_element, scene_rect, state);
        },
        onmouseenter: move |_| refresh_scene_metrics(scene_element, scene_rect, state),
        onpointerdown: move |evt| {
            evt.prevent_default();
            focus_scene(scene_element);
            refresh_scene_metrics(scene_element, scene_rect, state);
            let _ = bump_revision(wheel_revision);
            let coords = evt.client_coordinates();
            pan_state
                .set(
                    Some(PanState {
                        start_client_x: coords.x,
                        start_client_y: coords.y,
                        origin: displayed_viewport,
                        moved: false,
                    }),
                );
        },
        onpointermove: move |evt| {
            let coords = evt.client_coordinates();
            let scene_point = scene_point_from_client(
                *scene_rect.read(),
                coords.x,
                coords.y,

            );
            if connection_state.peek().is_some() {
                if let Some(connection) = connection_state.write().as_mut() {
                    connection.pointer = scene_point;
                }
                return;
            }
            let current_drag = drag_state.peek().clone();
            if let Some(drag) = current_drag {
                let dx = (coords.x - drag.start_client_x) / displayed_viewport.zoom;
                let dy = (coords.y - drag.start_client_y) / displayed_viewport.zoom;
                drag_state
                    .set(
                        Some(DragState {
                            moved: drag.moved || dx.abs() > 1.0 || dy.abs() > 1.0,
                            ..drag.clone()
                        }),
                    );
                node_preview_positions
                    .write()
                    .insert(
                        drag.node_id.clone(),
                        GraphPoint {
                            x: drag.origin.x + dx,
                            y: drag.origin.y + dy,
                        },
                    );
                return;
            }
            let current_pan = *pan_state.peek();
            if let Some(pan) = current_pan {
                let dx = coords.x - pan.start_client_x;
                let dy = coords.y - pan.start_client_y;
                pan_state
                    .set(
                        Some(PanState {
                            moved: pan.moved || dx.abs() > 1.0 || dy.abs() > 1.0,
                            ..pan
                        }),
                    );
                viewport_preview
                    .set(
                        Some(GraphViewport {
                            pan_x: pan.origin.pan_x + dx,
                            pan_y: pan.origin.pan_y + dy,
                            zoom: pan.origin.zoom,
                        }),
                    );
            }
        },
        onpointerup: move |_| finish_interaction(),
        onpointercancel: move |_| finish_interaction(),
        onmouseleave: move |_| finish_interaction(),
        onwheel: move |evt| {
            evt.prevent_default();
            focus_scene(scene_element);
            refresh_scene_metrics(scene_element, scene_rect, state);

            let delta_y = match evt.delta() {
                WheelDelta::Pixels(delta) => delta.y,
                WheelDelta::Lines(delta) => delta.y * 40.0,
                WheelDelta::Pages(delta) => delta.y * 400.0,
            };
            let factor = if delta_y < 0.0 { 1.1 } else { 1.0 / 1.1 };
            let coords = evt.client_coordinates();
            let scene_point = scene_point_from_client(
                *scene_rect.read(),
                coords.x,
                coords.y,
            );
            let world_point = world_point_from_scene(displayed_viewport, scene_point);
            let next_zoom = (displayed_viewport.zoom * factor).clamp(0.3, 2.4);
            viewport_preview
                .set(
                    Some(GraphViewport {
                        pan_x: scene_point.x - world_point.x * next_zoom,
                        pan_y: scene_point.y - world_point.y * next_zoom,
                        zoom: next_zoom,
                    }),
                );
            let revision = bump_revision(wheel_revision);
            let mut flow_state = state;
            let mut viewport_preview = viewport_preview;
            spawn(async move {
                sleep(Duration::from_millis(140)).await;
                let next_viewport = *viewport_preview.peek();
                if *wheel_revision.peek() == revision && let Some(viewport) = next_viewport {
                    if let Err(error) = flow_state
                        .dispatch(GraphCommand::SetViewport {
                            viewport,
                        })
                    {
                        report_action_error(&flow_state, "update viewport", error);
                    }
                    viewport_preview.set(None);
                }
            });
        },
        onkeydown: move |evt: KeyboardEvent| {
            if (evt.key() == Key::Delete || evt.key() == Key::Backspace)
                && can_delete_selection
            {
                evt.prevent_default();
                if let Err(error) = state.dispatch(GraphCommand::DeleteSelection) {
                    report_action_error(&state, "delete selection", error);
                }
            }
        },
        div {
          class: "pointer-events-none absolute inset-0",
          style: "{grid_overlay_style}"
        }
        div {
          class: "pointer-events-none absolute inset-x-0 top-0 h-24",
          style: "background: linear-gradient(180deg, rgba(5, 9, 15, 0.16) 0%, transparent 100%);"
        }
        div {
          class: "absolute left-0 top-0",
          style: "width: {world_width}px; height: {world_height}px; transform: translate({displayed_viewport.pan_x}px, {displayed_viewport.pan_y}px) scale({displayed_viewport.zoom}); transform-origin: 0 0;",
          svg {
            class: "absolute left-0 top-0 overflow-visible",
            width: "{world_width}",
            height: "{world_height}",
            view_box: "0 0 {world_width} {world_height}",
            for edge in renderable_edges {
              {
                  let from_point = edge.from;
                  let to_point = edge.to;
                  let is_selected = edge.is_selected;
                  let edge_id = edge.id.clone();
                  let edge_selection = GraphSelection::single_edge(edge_id.clone());
                  let should_select_edge = selection != edge_selection;
                  let edge_label = edge.label.clone();
                  let edge_color = if edge.has_error {
                      "#f87171"
                  } else if is_selected {
                      "#7dd3fc"
                  } else {
                      "#8ea4ba"
                  };
                  let path_data = edge_path(from_point, to_point);
                  rsx! {
                    g {
                      key: "{edge.id}",
                      path {
                        d: "{path_data}",
                        fill: "none",
                        stroke: "rgba(4, 8, 14, 0.74)",
                        stroke_width: if is_selected { "7" } else { "5" },
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        pointer_events: "none",
                      }
                      if is_selected {
                        path {
                          d: "{path_data}",
                          fill: "none",
                          stroke: "{edge_color}",
                          stroke_width: "6",
                          stroke_linecap: "round",
                          stroke_linejoin: "round",
                          stroke_opacity: "0.14",
                          pointer_events: "none",
                        }
                      }
                      path {
                        d: "{path_data}",
                        fill: "none",
                        stroke: "{edge_color}",
                        stroke_width: if is_selected { "3" } else { "2" },
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        style: "cursor: pointer;",
                        onpointerdown: move |evt| {
                            evt.stop_propagation();
                            evt.prevent_default();
                            focus_scene(scene_element);
                            let _ = bump_revision(wheel_revision);
                            if should_select_edge
                                && let Err(error) = state
                                    .dispatch(GraphCommand::Select {
                                        selection: edge_selection.clone(),
                                    })
                            {
                                report_action_error(&state, "select edge", error);
                            }
                        },
                      }
                      if let Some(edge_label) = edge_label {
                        {
                            let label_x = (from_point.x + to_point.x) / 2.0;
                            let label_y = (from_point.y + to_point.y) / 2.0 - 10.0;
                            let label_width = edge_label.chars().count() as f64 * 6.6 + 18.0;
                            rsx! {
                              g {
                                transform: "translate({label_x} {label_y})",
                                pointer_events: "none",
                                rect {
                                  x: "{-label_width / 2.0}",
                                  y: "-11",
                                  width: "{label_width}",
                                  height: "22",
                                  rx: "11",
                                  fill: "rgba(7, 12, 18, 0.88)",
                                  stroke: "rgba(156, 176, 197, 0.18)",
                                }
                                text {
                                  x: "0",
                                  y: "4",
                                  fill: "#9cb0c5",
                                  font_size: "11",
                                  font_weight: "600",
                                  text_anchor: "middle",
                                  "{edge_label}"
                                }
                              }
                            }
                        }
                      }
                    }
                  }
              }
            }
          }

          div { class: "absolute left-0 top-0",
            if nodes.is_empty() {
              div {
                class: "grid place-items-center",
                style: "width: {world_width}px; height: {world_height}px;",
                div { class: "rounded-2xl border border-white/10 px-6 py-5 text-center",
                  style: "background: rgba(7, 12, 18, 0.72); backdrop-filter: blur(16px);",
                  div { class: "text-[10px] font-semibold uppercase tracking-[0.22em] text-[#7f95ab]",
                    "Canvas Empty"
                  }
                  div { class: "mt-2 text-base font-semibold text-white",
                    "Add a node to begin shaping this workflow."
                  }
                  p { class: "mt-2 max-w-sm text-sm leading-6 text-[#90a3b7]",
                    "Use the palette on the left to drop the first step into the graph. New nodes land in the current viewport center."
                  }
                }
              }
            } else {
              for node in nodes {
                {
                    let template = node_template(&templates, &node.template_id).cloned();
                    let position = displayed_node_position(&node, &preview_positions);
                    let node_id = node.id.clone();
                    let node_key = node_id.clone();
                    let node_template_id = node.template_id.clone();
                    let node_label = node.label.clone().unwrap_or_else(|| node_id.clone());
                    let node_template_label =
                        template.as_ref().map_or(node_template_id.clone(), |template| template.label.clone());
                    let node_category = template.as_ref().and_then(|template| template.category.as_deref().map(category_label));
                    let node_selection = GraphSelection::single_node(node_id.clone());
                    let should_select_node = selection != node_selection;
                    let node_diagnostics: Vec<_> = diagnostics
                        .iter()
                        .filter(|diagnostic| {
                            matches!(
                                diagnostic.target,
                                Some(GraphEntityRef::Node(ref id))
                                if id == &node.id
                            )
                        })
                        .cloned()
                        .collect();
                    let is_selected = selection.node_ids.iter().any(|selected| selected == &node.id);
                    let node_style = node_style(
                        position,
                        node_height(template.as_ref()),
                        is_selected,
                    );
                    let input_ports = ports_by_direction(template.as_ref(), PortDirection::Input);
                    let output_ports = ports_by_direction(template.as_ref(), PortDirection::Output);
                    rsx! {
                      div {
                        key: "{node_key}",
                        style: "{node_style}",
                        onpointerdown: move |evt| {
                            evt.stop_propagation();
                            evt.prevent_default();
                            focus_scene(scene_element);
                            refresh_scene_metrics(scene_element, scene_rect, state);
                            let _ = bump_revision(wheel_revision);
                            let coords = evt.client_coordinates();
                            if should_select_node
                                && let Err(error) = state
                                    .dispatch(GraphCommand::Select {
                                        selection: node_selection.clone(),
                                    })
                            {
                                report_action_error(&state, "select node", error);
                            }
                            drag_state
                                .set(
                                    Some(DragState {
                                        node_id: node_id.clone(),
                                        start_client_x: coords.x,
                                        start_client_y: coords.y,
                                        origin: position,
                                        moved: false,
                                    }),
                                );
                        },
                        div {
                          class: "absolute inset-x-5 top-0 h-px",
                          style: if is_selected {
                              "background: linear-gradient(90deg, transparent, rgba(125, 211, 252, 0.95), transparent);"
                          } else {
                              "background: linear-gradient(90deg, transparent, rgba(132, 149, 169, 0.45), transparent);"
                          },
                        }
                        div { style: "padding: 15px 16px 13px; border-bottom: 1px solid rgba(255, 255, 255, 0.05); background: linear-gradient(180deg, rgba(20, 29, 42, 0.98) 0%, rgba(11, 17, 26, 0.84) 100%);",
                          div { class: "flex items-start justify-between gap-3",
                            div { class: "min-w-0",
                              if let Some(node_category) = node_category {
                                div { class: "inline-flex rounded-full border border-white/10 px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.2em] text-[#8ea3b9]",
                                  style: "background: rgba(255, 255, 255, 0.04);",
                                  "{node_category}"
                                }
                              }
                              div { class: "mt-3 truncate text-[17px] font-semibold text-white",
                                "{node_label}"
                              }
                              div { class: "mt-1 truncate text-[12px] text-[#a3b3c4]",
                                "{node_template_label}"
                              }
                            }
                            if !node_diagnostics.is_empty() {
                              span {
                                class: "inline-flex min-w-6 items-center justify-center rounded-full px-2.5 py-1 text-[11px] font-bold",
                                style: if node_diagnostics
                          .iter()
                          .any(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error) { "background: rgba(248, 113, 113, 0.16); color: #fca5a5; border: 1px solid rgba(248, 113, 113, 0.26);" } else { "background: rgba(251, 191, 36, 0.16); color: #fcd34d; border: 1px solid rgba(251, 191, 36, 0.26);" },
                                "{node_diagnostics.len()}"
                              }
                            }
                          }
                        }
                        div {
                          class: "grid gap-4",
                          style: "grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); padding: 16px;",
                          div { class: "flex flex-col",
                            for port in input_ports {
                              {
                                  let badge_key = format!("{node_key}-{}-input", port.id);
                                  let target_node_id = node_id.clone();
                                  let target_port_id = port.id.clone();
                                  rsx! {
                                    PortBadge {
                                      key: "{badge_key}",
                                      port: port.clone(),
                                      direction: PortDirection::Input,
                                      on_pointer_down: move |evt: PointerEvent| {
                                          evt.stop_propagation();
                                          evt.prevent_default();
                                          focus_scene(scene_element);
                                          let _ = bump_revision(wheel_revision);
                                      },
                                      on_pointer_up: move |evt: PointerEvent| {
                                          evt.stop_propagation();
                                          evt.prevent_default();
                                          let current_connection = connection_state.peek().clone();
                                          if let Some(connection) = current_connection {
                                              let edge_id = uuid::Uuid::new_v4().to_string();
                                          let command = GraphCommand::ConnectPorts {
                                              edge_id,
                                              from: connection.from,
                                              to: GraphPortRef {
                                                  node_id: target_node_id.clone(),
                                                  port_id: target_port_id.clone(),
                                              },
                                              label: None,
                                          };
                                          if let Err(error) = state.dispatch(command) {
                                              report_action_error(&state, "connect ports", error);
                                          }
                                              connection_state.set(None);
                                          }
                                      },
                                    }
                                  }
                              }
                            }
                          }
                          div { class: "flex flex-col items-end",
                            for port in output_ports {
                              {
                                  let badge_key = format!("{node_key}-{}-output", port.id);
                                  let source_node_id = node_id.clone();
                                  let source_port_id = port.id.clone();
                                  rsx! {
                                    PortBadge {
                                      key: "{badge_key}",
                                      port: port.clone(),
                                      direction: PortDirection::Output,
                                      on_pointer_down: move |evt: PointerEvent| {
                                          evt.stop_propagation();
                                          evt.prevent_default();
                                          focus_scene(scene_element);
                                          refresh_scene_metrics(scene_element, scene_rect, state);
                                          let _ = bump_revision(wheel_revision);
                                          let coords = evt.client_coordinates();
                                          connection_state
                                              .set(
                                                  Some(ConnectionState {
                                                      from: GraphPortRef {
                                                          node_id: source_node_id.clone(),
                                                          port_id: source_port_id.clone(),
                                                      },
                                                      pointer: scene_point_from_client(
                                                          *scene_rect.read(),
                                                          coords.x,
                                                          coords.y,
                                                      ),
                                                  }),
                                              );
                                      },
                                      on_pointer_up: move |evt: PointerEvent| {
                                          evt.stop_propagation();
                                          evt.prevent_default();
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
              }
            }
          }
        }

        svg {
          class: "pointer-events-none absolute inset-0",
          width: "{scene_width.max(1.0)}",
          height: "{scene_height.max(1.0)}",
          view_box: "0 0 {scene_width.max(1.0)} {scene_height.max(1.0)}",
          if let Some(connection) = connection_preview {
            if let Some(from_node) = document.node(&connection.from.node_id) {
              if let Some(start_point) = port_screen_point(
                  &templates,
                  from_node,
                  &preview_positions,
                  &connection.from.port_id,
                  displayed_viewport,
              )
              {
                path {
                  d: "{edge_path(start_point, connection.pointer)}",
                  fill: "none",
                  stroke: "#34d399",
                  stroke_width: "2.5",
                  stroke_dasharray: "8 6",
                }
              }
            }
          }
        }
      }
      if show_interaction_hint {
        div {
          class: "pointer-events-none absolute bottom-4 left-4 z-10 max-w-[22rem] rounded-full px-3 py-1.5 text-[10px] uppercase tracking-[0.14em] text-[#93a6ba]",
          style: "border: 1px solid rgba(255, 255, 255, 0.08); background: rgba(7, 12, 18, 0.58); backdrop-filter: blur(14px);",
          "{interaction_hint}"
        }
      }
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SceneRect {
  left: f64,
  top: f64,
  width: f64,
  height: f64,
}

#[derive(Clone, Debug, PartialEq)]
struct DragState {
  node_id: String,
  start_client_x: f64,
  start_client_y: f64,
  origin: GraphPoint,
  moved: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PanState {
  start_client_x: f64,
  start_client_y: f64,
  origin: GraphViewport,
  moved: bool,
}

#[derive(Clone, Debug, PartialEq)]
struct ConnectionState {
  from: GraphPortRef,
  pointer: GraphPoint,
}

#[derive(Clone, Debug, PartialEq)]
struct RenderableEdge {
  id: String,
  label: Option<String>,
  from: GraphPoint,
  to: GraphPoint,
  is_selected: bool,
  has_error: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GraphBounds {
  min_x: f64,
  min_y: f64,
  max_x: f64,
  max_y: f64,
}

#[component]
fn PortBadge(
  port: GraphPortTemplate,
  direction: PortDirection,
  on_pointer_down: EventHandler<PointerEvent>,
  on_pointer_up: EventHandler<PointerEvent>,
) -> Element {
  let dot = if direction == PortDirection::Input { "#7dd3fc" } else { "#4ade80" };
  let background = if direction == PortDirection::Input { "rgba(56, 189, 248, 0.10)" } else { "rgba(34, 197, 94, 0.10)" };
  let border = if direction == PortDirection::Input { "1px solid rgba(56, 189, 248, 0.26)" } else { "1px solid rgba(34, 197, 94, 0.24)" };
  let text_color = if direction == PortDirection::Input { "#d3ecff" } else { "#d6fbe1" };
  let cursor = if direction == PortDirection::Output { "crosshair" } else { "pointer" };

  rsx! {
    div {
      class: "inline-flex min-h-8 max-w-full items-center gap-2.5 rounded-full px-3 py-1.5",
      style: "background: {background}; border: {border}; margin-bottom: 10px; cursor: {cursor}; box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.04);",
      onpointerdown: move |evt| on_pointer_down.call(evt),
      onpointerup: move |evt| on_pointer_up.call(evt),
      if direction == PortDirection::Input {
        span {
          class: "size-2 rounded-full shrink-0",
          style: "background: {dot};",
        }
        span {
          class: "truncate text-[11px] font-semibold",
          style: "color: {text_color};",
          "{port.label}"
        }
      } else {
        span {
          class: "truncate text-[11px] font-semibold",
          style: "color: {text_color};",
          "{port.label}"
        }
        span {
          class: "size-2 rounded-full shrink-0",
          style: "background: {dot};",
        }
      }
    }
  }
}

fn bump_revision(mut wheel_revision: Signal<u64>) -> u64 {
  let next = *wheel_revision.peek() + 1;
  wheel_revision.set(next);
  next
}

fn focus_scene(scene_element: Signal<Option<Rc<MountedData>>>) {
  let Some(scene) = scene_element.peek().as_ref().cloned() else {
    return;
  };
  spawn(async move {
    let _ = scene.set_focus(true).await;
  });
}

fn refresh_scene_metrics(scene_element: Signal<Option<Rc<MountedData>>>, mut scene_rect: Signal<Option<SceneRect>>, state: super::controller::FlowEditorState) {
  let Some(scene) = scene_element.peek().as_ref().cloned() else {
    return;
  };
  spawn(async move {
    let Ok(rect) = scene.get_client_rect().await else {
      return;
    };
    let next_rect = SceneRect { left: rect.origin.x, top: rect.origin.y, width: rect.width(), height: rect.height() };
    scene_rect.set(Some(next_rect));
    state.register_canvas_size(next_rect.width.max(1.0), next_rect.height.max(1.0));
  });
}

fn scene_point_from_client(scene_rect: Option<SceneRect>, client_x: f64, client_y: f64) -> GraphPoint {
  match scene_rect {
    Some(rect) => GraphPoint { x: client_x - rect.left, y: client_y - rect.top },
    None => GraphPoint { x: client_x, y: client_y },
  }
}

fn world_point_from_scene(viewport: GraphViewport, point: GraphPoint) -> GraphPoint {
  GraphPoint { x: (point.x - viewport.pan_x) / viewport.zoom, y: (point.y - viewport.pan_y) / viewport.zoom }
}

fn canvas_world_size(document: &GraphDocument, templates: &[GraphNodeTemplate], preview_positions: &HashMap<String, GraphPoint>) -> (f64, f64) {
  let width = document.nodes.iter().map(|node| displayed_node_position(node, preview_positions).x + NODE_WIDTH + 180.0).fold(1280.0, f64::max);
  let height = document
    .nodes
    .iter()
    .map(|node| displayed_node_position(node, preview_positions).y + node_height(node_template(templates, &node.template_id)) + 180.0)
    .fold(720.0, f64::max);
  (width, height)
}

fn fit_viewport(document: &GraphDocument, templates: &[GraphNodeTemplate], scene_width: f64, scene_height: f64) -> Option<GraphViewport> {
  let bounds = document_bounds(document, templates, &HashMap::new())?;
  let width = (bounds.max_x - bounds.min_x).max(1.0);
  let height = (bounds.max_y - bounds.min_y).max(1.0);
  let available_width = (scene_width - FIT_VIEW_PADDING_X * 2.0).max(280.0);
  let available_height = (scene_height - FIT_VIEW_PADDING_Y * 2.0).max(220.0);
  let zoom = (available_width / width).min(available_height / height).clamp(0.42, 1.0);
  let center_x = bounds.min_x + width * 0.5;
  let center_y = bounds.min_y + height * 0.5;
  Some(GraphViewport { pan_x: scene_width * 0.5 - center_x * zoom, pan_y: scene_height * 0.5 - center_y * zoom, zoom })
}

fn viewport_needs_fit(document: &GraphDocument, templates: &[GraphNodeTemplate], scene_width: f64, scene_height: f64) -> bool {
  let Some(bounds) = document_bounds(document, templates, &HashMap::new()) else {
    return false;
  };
  let screen_bounds = world_bounds_to_screen(bounds, document.viewport);
  let inset = 36.0;
  let visible_width = (screen_bounds.max_x - screen_bounds.min_x).max(1.0);
  let visible_height = (screen_bounds.max_y - screen_bounds.min_y).max(1.0);
  let width_fill = visible_width / scene_width.max(1.0);
  let height_fill = visible_height / scene_height.max(1.0);
  screen_bounds.min_x < inset
    || screen_bounds.max_x > scene_width - inset
    || screen_bounds.min_y < inset
    || screen_bounds.max_y > scene_height - inset
    || width_fill < 0.56
    || height_fill < 0.34
}

fn document_bounds(document: &GraphDocument, templates: &[GraphNodeTemplate], preview_positions: &HashMap<String, GraphPoint>) -> Option<GraphBounds> {
  let mut bounds = None::<GraphBounds>;
  for node in &document.nodes {
    let position = displayed_node_position(node, preview_positions);
    let node_bounds = GraphBounds {
      min_x: position.x,
      min_y: position.y,
      max_x: position.x + NODE_WIDTH,
      max_y: position.y + node_height(node_template(templates, &node.template_id)),
    };
    bounds = Some(match bounds {
      Some(current) => GraphBounds {
        min_x: current.min_x.min(node_bounds.min_x),
        min_y: current.min_y.min(node_bounds.min_y),
        max_x: current.max_x.max(node_bounds.max_x),
        max_y: current.max_y.max(node_bounds.max_y),
      },
      None => node_bounds,
    });
  }
  bounds
}

fn world_bounds_to_screen(bounds: GraphBounds, viewport: GraphViewport) -> GraphBounds {
  GraphBounds {
    min_x: bounds.min_x * viewport.zoom + viewport.pan_x,
    min_y: bounds.min_y * viewport.zoom + viewport.pan_y,
    max_x: bounds.max_x * viewport.zoom + viewport.pan_x,
    max_y: bounds.max_y * viewport.zoom + viewport.pan_y,
  }
}

const FIT_VIEW_PADDING_X: f64 = 56.0;
const FIT_VIEW_PADDING_Y: f64 = 72.0;
const NODE_WIDTH: f64 = 288.0;
const NODE_HEADER_HEIGHT: f64 = 72.0;
const NODE_PORT_ROW_HEIGHT: f64 = 30.0;
const NODE_BODY_PADDING: f64 = 16.0;

fn node_style(position: GraphPoint, height: f64, is_selected: bool) -> String {
  let border = if is_selected { "1px solid rgba(125, 211, 252, 0.92)" } else { "1px solid rgba(255, 255, 255, 0.07)" };
  let box_shadow = if is_selected { "0 0 0 2px rgba(56, 189, 248, 0.18), 0 18px 42px rgba(0, 0, 0, 0.34)" } else { "0 14px 34px rgba(0, 0, 0, 0.30)" };
  format!(
    "position: absolute; left: {}px; top: {}px; width: {}px; min-height: {}px; border-radius: 22px; border: {}; box-shadow: {}; background: linear-gradient(180deg, rgba(17, 25, 36, 0.99) 0%, rgba(8, 13, 21, 0.99) 100%); overflow: hidden; user-select: none;",
    position.x, position.y, NODE_WIDTH, height, border, box_shadow
  )
}

fn ports_by_direction(template: Option<&GraphNodeTemplate>, direction: PortDirection) -> Vec<GraphPortTemplate> {
  template.map(|template| template.ports.iter().filter(|port| port.direction == direction).cloned().collect()).unwrap_or_default()
}

fn node_height(template: Option<&GraphNodeTemplate>) -> f64 {
  let input_rows = ports_by_direction(template, PortDirection::Input).len();
  let output_rows = ports_by_direction(template, PortDirection::Output).len();
  NODE_HEADER_HEIGHT + NODE_BODY_PADDING * 2.0 + NODE_PORT_ROW_HEIGHT * input_rows.max(output_rows).max(1) as f64
}

fn displayed_node_position(node: &GraphNode, preview_positions: &HashMap<String, GraphPoint>) -> GraphPoint {
  preview_positions.get(&node.id).copied().unwrap_or(node.position)
}

fn port_world_point_for_displayed_node(
  templates: &[GraphNodeTemplate],
  node: &GraphNode,
  preview_positions: &HashMap<String, GraphPoint>,
  port_id: &str,
) -> Option<GraphPoint> {
  port_world_point(templates, node, displayed_node_position(node, preview_positions), port_id)
}

fn port_screen_point(
  templates: &[GraphNodeTemplate],
  node: &GraphNode,
  preview_positions: &HashMap<String, GraphPoint>,
  port_id: &str,
  viewport: GraphViewport,
) -> Option<GraphPoint> {
  let world_point = port_world_point_for_displayed_node(templates, node, preview_positions, port_id)?;
  Some(GraphPoint { x: world_point.x * viewport.zoom + viewport.pan_x, y: world_point.y * viewport.zoom + viewport.pan_y })
}

fn port_world_point(templates: &[GraphNodeTemplate], node: &GraphNode, position: GraphPoint, port_id: &str) -> Option<GraphPoint> {
  let template = node_template(templates, &node.template_id);
  let inputs = ports_by_direction(template, PortDirection::Input);
  let outputs = ports_by_direction(template, PortDirection::Output);

  let input_index = inputs.iter().position(|port| port.id == port_id);
  if let Some(index) = input_index {
    return Some(GraphPoint {
      x: position.x,
      y: position.y + NODE_HEADER_HEIGHT + NODE_BODY_PADDING + index as f64 * NODE_PORT_ROW_HEIGHT + NODE_PORT_ROW_HEIGHT / 2.0,
    });
  }

  let output_index = outputs.iter().position(|port| port.id == port_id);
  output_index.map(|index| GraphPoint {
    x: position.x + NODE_WIDTH,
    y: position.y + NODE_HEADER_HEIGHT + NODE_BODY_PADDING + index as f64 * NODE_PORT_ROW_HEIGHT + NODE_PORT_ROW_HEIGHT / 2.0,
  })
}

fn edge_path(from: GraphPoint, to: GraphPoint) -> String {
  let dx = (to.x - from.x).abs().mul_add(0.42, 0.0).max(84.0);
  format!("M {} {} C {} {} {} {} {} {}", from.x, from.y, from.x + dx, from.y, to.x - dx, to.y, to.x, to.y)
}

#[component]
fn ValidationSurface(diagnostics: Vec<GraphWidgetDiagnostic>) -> Element {
  let error_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error).count();
  let warning_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Warning).count();

  rsx! {
    if diagnostics.is_empty() {
      div { class: "rounded-xl border border-emerald-500/20 bg-[var(--surface-container)] px-4 py-3",
        div { class: "flex flex-wrap items-center justify-between gap-3",
          div {
            div { class: "text-[11px] font-mono uppercase tracking-[0.2em] text-[var(--outline)]",
              "Validation"
            }
            div { class: "mt-1 text-sm font-medium text-emerald-200",
              "Healthy graph"
            }
          }
          div { class: "flex items-center gap-2",
            span { class: "rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.14em] text-emerald-200",
              "0 errors"
            }
            span { class: "rounded-full border border-[var(--outline-variant)] bg-[var(--surface-container-high)] px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.14em] text-[var(--on-surface-variant)]",
              "0 warnings"
            }
          }
        }
      }
    } else {
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

fn report_action_error(state: &super::controller::FlowEditorState, action: &str, error: impl std::fmt::Display) {
  let mut status_message = state.status_message;
  status_message.set(Some(format!("Failed to {action}: {error}")));
}
