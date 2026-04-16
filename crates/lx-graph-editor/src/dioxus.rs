use std::collections::HashMap;
use std::rc::Rc;
use std::time::Duration;

use dioxus::html::geometry::WheelDelta;
use dioxus::prelude::*;
use tokio::time::sleep;

use crate::catalog::{GraphNodeTemplate, GraphPortTemplate, PortDirection, node_template};
use crate::commands::GraphCommand;
use crate::history::GraphEditorAction;
use crate::model::{GraphDocument, GraphEntityRef, GraphNode, GraphPoint, GraphPortRef, GraphSelection, GraphViewport};
use crate::protocol::{GraphEdgeRunState, GraphRunSnapshot, GraphRunStatus, GraphWidgetDiagnostic, GraphWidgetDiagnosticSeverity};

#[component]
pub fn GraphCanvas(
  document: GraphDocument,
  templates: Vec<GraphNodeTemplate>,
  diagnostics: Vec<GraphWidgetDiagnostic>,
  run_snapshot: Option<GraphRunSnapshot>,
  canvas_size: (f64, f64),
  on_command: EventHandler<GraphCommand>,
  on_editor_action: EventHandler<GraphEditorAction>,
  on_canvas_size: EventHandler<(f64, f64)>,
  empty_title: String,
  empty_message: String,
) -> Element {
  let flow_id = document.id.clone();
  let selection = document.selection.clone();
  let mut scene_element = use_signal(|| Option::<Rc<MountedData>>::None);
  let scene_rect = use_signal(|| Option::<SceneRect>::None);
  let mut viewport_preview = use_signal(|| Option::<GraphViewport>::None);
  let mut node_preview_positions = use_signal(HashMap::<String, GraphPoint>::new);
  let mut drag_state = use_signal(|| Option::<DragState>::None);
  let mut pan_state = use_signal(|| Option::<PanState>::None);
  let mut marquee_state = use_signal(|| Option::<MarqueeState>::None);
  let mut connection_state = use_signal(|| Option::<ConnectionState>::None);
  let wheel_revision = use_signal(|| 0u64);
  let auto_framed_flow = use_signal(|| Option::<String>::None);

  let nodes = document.nodes.clone();
  let has_nodes = !nodes.is_empty();
  let edges = document.edges.clone();
  let displayed_viewport = viewport_preview.read().unwrap_or(document.viewport);
  let preview_positions = node_preview_positions.read().clone();
  let node_run_states = run_snapshot
    .as_ref()
    .map(|snapshot| snapshot.node_states.iter().cloned().map(|state| (state.node_id.clone(), state)).collect::<HashMap<_, _>>())
    .unwrap_or_default();
  let edge_run_states = run_snapshot
    .as_ref()
    .map(|snapshot| snapshot.edge_states.iter().cloned().map(|state| (state.edge_id.clone(), state)).collect::<HashMap<_, _>>())
    .unwrap_or_default();
  let is_dragging_node = drag_state.read().is_some();
  let is_panning = pan_state.read().is_some();
  let is_marquee_selecting = marquee_state.read().is_some();
  let connection_preview = connection_state.read().clone();
  let (scene_width, scene_height) = canvas_size;
  let (world_width, world_height) = canvas_world_size(&document, &templates, &preview_positions);
  let grid_size = (24.0 * displayed_viewport.zoom).max(18.0);
  let major_grid_size = grid_size * 6.0;
  let error_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error).count();
  let warning_count = diagnostics.iter().filter(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Warning).count();
  let issue_summary = format!("{error_count} errors / {warning_count} warnings");
  let issue_badge_style = if error_count > 0 {
    "border: 1px solid var(--graph-error-border); background: var(--graph-error-surface); color: var(--graph-error-text); backdrop-filter: blur(16px);"
  } else {
    "border: 1px solid var(--graph-warning-border); background: var(--graph-warning-surface); color: var(--graph-warning-text); backdrop-filter: blur(16px);"
  };
  let run_badge = run_snapshot.as_ref().map(|snapshot| (run_snapshot_label(snapshot), run_status_badge_style(snapshot.status)));
  let selection_badge = selection_summary(&selection);
  let zoom_label = format!("{:.0}% zoom", displayed_viewport.zoom * 100.0);
  let scene_class = if connection_preview.is_some() || is_marquee_selecting {
    "relative flex-1 min-h-0 overflow-hidden outline-none cursor-crosshair"
  } else if is_panning || is_dragging_node {
    "relative flex-1 min-h-0 overflow-hidden outline-none cursor-grabbing"
  } else {
    "relative flex-1 min-h-0 overflow-hidden outline-none cursor-grab"
  };
  let canvas_badge_class = "rounded-full px-3 py-1.5 text-[11px] font-medium";
  let canvas_badge_style =
    "border: 1px solid var(--graph-overlay-border); background: var(--graph-overlay-bg); color: var(--graph-overlay-muted); backdrop-filter: blur(16px);";
  let canvas_control_class = "rounded-full px-3 py-1.5 text-[11px] font-semibold transition-all hover:brightness-105";
  let canvas_control_style =
    "border: 1px solid var(--graph-overlay-border); background: var(--graph-overlay-bg-strong); color: var(--graph-overlay-text); backdrop-filter: blur(16px);";
  let grid_overlay_style = format!(
    "background-image: linear-gradient(var(--graph-grid-minor) 1px, transparent 1px), linear-gradient(90deg, var(--graph-grid-minor) 1px, transparent 1px), linear-gradient(var(--graph-grid-major) 1px, transparent 1px), linear-gradient(90deg, var(--graph-grid-major) 1px, transparent 1px); background-size: {grid_size}px {grid_size}px, {grid_size}px {grid_size}px, {major_grid_size}px {major_grid_size}px, {major_grid_size}px {major_grid_size}px; background-position: {}px {}px, {}px {}px, {}px {}px, {}px {}px;",
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
        run_state: edge_run_states.get(&edge.id).cloned(),
      })
    })
    .collect();
  let can_delete_selection = !selection.node_ids.is_empty() || !selection.edge_ids.is_empty();
  let interaction_document = document.clone();
  let interaction_selection = selection.clone();
  let interaction_templates = templates.clone();
  let interaction_preview_positions = preview_positions.clone();
  let marquee_overlay = marquee_state.read().as_ref().map(marquee_bounds);
  let marquee_base_selection = selection.clone();

  let finish_interaction = EventHandler::new({
    let on_command = on_command;
    let mut drag_state = drag_state;
    let mut pan_state = pan_state;
    let mut marquee_state = marquee_state;
    let mut connection_state = connection_state;
    let mut viewport_preview = viewport_preview;
    let mut node_preview_positions = node_preview_positions;
    move |_| {
      connection_state.set(None);

      let current_drag = drag_state.peek().clone();
      if let Some(drag) = current_drag {
        let final_position =
          node_preview_positions.peek().get(&drag.node_id).copied().or_else(|| interaction_document.node(&drag.node_id).map(|node| node.position));
        if let Some(position) = final_position
          && drag.moved
        {
          on_command.call(GraphCommand::MoveNode { node_id: drag.node_id.clone(), position });
        }
        node_preview_positions.write().remove(&drag.node_id);
        drag_state.set(None);
        return;
      }

      let current_marquee = marquee_state.peek().clone();
      if let Some(marquee) = current_marquee {
        if marquee.moved {
          let scene_bounds = marquee_bounds(&marquee);
          let world_bounds = world_bounds_from_scene(scene_bounds, displayed_viewport);
          let next_selection =
            selection_for_marquee(&interaction_document, &interaction_templates, &interaction_preview_positions, world_bounds, &marquee.base_selection);
          on_command.call(GraphCommand::Select { selection: next_selection });
        }
        marquee_state.set(None);
        return;
      }

      let current_pan = *pan_state.peek();
      if let Some(pan) = current_pan {
        if pan.moved {
          let current_viewport = *viewport_preview.peek();
          if let Some(viewport) = current_viewport {
            on_command.call(GraphCommand::SetViewport { viewport });
          }
        } else if !interaction_selection.is_empty() {
          on_command.call(GraphCommand::Select { selection: GraphSelection::empty() });
        }
        pan_state.set(None);
        viewport_preview.set(None);
      }
    }
  });

  {
    let on_command = on_command;
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
      {
        on_command.call(GraphCommand::SetViewport { viewport });
      }
    });
  }

  rsx! {
    div {
      class: "relative flex h-full min-h-0 flex-col",
      style: "background: linear-gradient(180deg, var(--graph-canvas-bg-start) 0%, var(--graph-canvas-bg-end) 100%); color: var(--graph-overlay-text);",
      div {
        class: "pointer-events-none absolute inset-0",
        style: "background: radial-gradient(circle at top left, var(--graph-canvas-tint-primary) 0%, transparent 34%), radial-gradient(circle at bottom right, var(--graph-canvas-tint-secondary) 0%, transparent 28%);",
      }
      div { class: "pointer-events-none absolute right-4 top-4 z-10 flex max-w-[45%] flex-wrap items-center justify-end gap-2",
        if let Some((run_badge_label, run_badge_style)) = run_badge.clone() {
          span {
            class: "{canvas_badge_class}",
            style: "{run_badge_style}",
            "{run_badge_label}"
          }
        }
        if !diagnostics.is_empty() {
          span {
            class: "{canvas_badge_class}",
            style: "{issue_badge_style}",
            "{issue_summary}"
          }
        }
        if !selection.is_empty() {
          span {
            class: "{canvas_badge_class}",
            style: "{canvas_badge_style}",
            "{selection_badge}"
          }
        }
      }

      div {
        class: "{scene_class}",
        tabindex: "0",
        onmounted: move |evt: MountedEvent| {
            let element = evt.data();
            scene_element.set(Some(element));
            refresh_scene_metrics(scene_element, scene_rect, on_canvas_size);
        },
        onmouseenter: move |_| refresh_scene_metrics(scene_element, scene_rect, on_canvas_size),
        onpointerdown: move |evt| {
            evt.prevent_default();
            focus_scene(scene_element);
            refresh_scene_metrics(scene_element, scene_rect, on_canvas_size);
            let _ = bump_revision(wheel_revision);
            let coords = evt.client_coordinates();
            let scene_point = scene_point_from_client(
                *scene_rect.read(),
                coords.x,
                coords.y,
            );
            if evt.modifiers().shift() {
                marquee_state
                    .set(
                        Some(MarqueeState {
                            start_scene: scene_point,
                            current_scene: scene_point,
                            base_selection: marquee_base_selection.clone(),
                            moved: false,
                        }),
                    );
            } else {
                pan_state
                    .set(
                        Some(PanState {
                            start_client_x: coords.x,
                            start_client_y: coords.y,
                            origin: displayed_viewport,
                            moved: false,
                        }),
                    );
            }
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
            let current_marquee = marquee_state.peek().clone();
            if let Some(marquee) = current_marquee {
                marquee_state
                    .set(
                        Some(MarqueeState {
                            current_scene: scene_point,
                            moved: marquee.moved
                                || (scene_point.x - marquee.start_scene.x).abs() > 3.0
                                || (scene_point.y - marquee.start_scene.y).abs() > 3.0,
                            ..marquee
                        }),
                    );
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
        onpointerup: move |_| finish_interaction.call(()),
        onpointercancel: move |_| finish_interaction.call(()),
        onmouseleave: move |_| finish_interaction.call(()),
        onwheel: move |evt| {
            evt.prevent_default();
            focus_scene(scene_element);
            refresh_scene_metrics(scene_element, scene_rect, on_canvas_size);

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
            let on_command = on_command;
            let mut viewport_preview = viewport_preview;
            spawn(async move {
                sleep(Duration::from_millis(140)).await;
                let next_viewport = *viewport_preview.peek();
                if *wheel_revision.peek() == revision && let Some(viewport) = next_viewport {
                    on_command
                        .call(GraphCommand::SetViewport {
                            viewport,
                        });
                    viewport_preview.set(None);
                }
            });
        },
        onkeydown: move |evt: KeyboardEvent| {
            let modifiers = evt.modifiers();
            let command_key = modifiers.meta() || modifiers.ctrl();
            if command_key && evt.key() == Key::Character("z".into()) {
                evt.prevent_default();
                if modifiers.shift() {
                    on_editor_action.call(GraphEditorAction::Redo);
                } else {
                    on_editor_action.call(GraphEditorAction::Undo);
                }
                return;
            }
            if command_key && evt.key() == Key::Character("y".into()) {
                evt.prevent_default();
                on_editor_action.call(GraphEditorAction::Redo);
                return;
            }
            if command_key && evt.key() == Key::Character("c".into()) {
                evt.prevent_default();
                on_editor_action.call(GraphEditorAction::CopySelection);
                return;
            }
            if command_key && evt.key() == Key::Character("v".into()) {
                evt.prevent_default();
                on_editor_action.call(GraphEditorAction::PasteClipboard);
                return;
            }
            if command_key && evt.key() == Key::Character("d".into()) {
                evt.prevent_default();
                on_editor_action.call(GraphEditorAction::DuplicateSelection);
                return;
            }
            if command_key && evt.key() == Key::Character("a".into()) {
                evt.prevent_default();
                on_editor_action.call(GraphEditorAction::SelectAll);
                return;
            }
            if (evt.key() == Key::Delete || evt.key() == Key::Backspace)
                && can_delete_selection
            {
                evt.prevent_default();
                on_command.call(GraphCommand::DeleteSelection);
            }
        },
        div {
          class: "pointer-events-none absolute inset-0",
          style: "{grid_overlay_style}",
        }
        if let Some(marquee_overlay) = marquee_overlay {
          div {
            class: "pointer-events-none absolute rounded-lg border",
            style: "left: {marquee_overlay.min_x}px; top: {marquee_overlay.min_y}px; width: {marquee_overlay.max_x - marquee_overlay.min_x}px; height: {marquee_overlay.max_y - marquee_overlay.min_y}px; border-color: var(--graph-selection-border); background: color-mix(in srgb, var(--graph-selection-surface) 74%, transparent); box-shadow: inset 0 0 0 1px color-mix(in srgb, var(--graph-selection-border) 45%, transparent);",
          }
        }
        div {
          class: "pointer-events-none absolute inset-x-0 top-0 h-24",
          style: "background: linear-gradient(180deg, var(--graph-canvas-top-fade) 0%, transparent 100%);",
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
                  let edge_label = edge
                      .label
                      .clone()
                      .or_else(|| {
                          edge
                              .run_state
                              .as_ref()
                              .and_then(|state| {
                                  state.label.clone().or_else(|| state.detail.clone())
                              })
                      });
                  let edge_color = edge_stroke_color(
                      edge.run_state.as_ref(),
                      edge.has_error,
                      is_selected,
                  );
                  let path_data = edge_path(from_point, to_point);
                  rsx! {
                    g { key: "{edge.id}",
                      path {
                        d: "{path_data}",
                        fill: "none",
                        stroke: "var(--graph-edge-underlay)",
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
                            if should_select_edge {
                                on_command
                                    .call(GraphCommand::Select {
                                        selection: edge_selection.clone(),
                                    });
                            }
                        },
                      }
                      if let Some(edge_label) = edge_label {
                        {
                            let label_x = (from_point.x + to_point.x) / 2.0;
                            let label_y = (from_point.y + to_point.y) / 2.0 - 10.0;
                            let label_width = edge_label.chars().count() as f64 * 6.6 + 18.0;
                            rsx! {
                              g { transform: "translate({label_x} {label_y})", pointer_events: "none",
                                rect {
                                  x: "{-label_width / 2.0}",
                                  y: "-11",
                                  width: "{label_width}",
                                  height: "22",
                                  rx: "11",
                                  fill: "var(--graph-edge-label-bg)",
                                  stroke: "var(--graph-edge-label-border)",
                                }
                                text {
                                  x: "0",
                                  y: "4",
                                  fill: "var(--graph-edge-label-text)",
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
                div {
                  class: "rounded-2xl px-6 py-5 text-center",
                  style: "border: 1px solid var(--graph-overlay-border); background: var(--graph-overlay-bg); backdrop-filter: blur(16px);",
                  div {
                    class: "text-[10px] font-semibold uppercase tracking-[0.22em]",
                    style: "color: var(--graph-overlay-muted);",
                    "{empty_title}"
                  }
                  div { class: "mt-2 text-base font-semibold text-[var(--on-surface)]",
                    "Add a node to begin shaping this graph."
                  }
                  p {
                    class: "mt-2 max-w-sm text-sm leading-6",
                    style: "color: var(--graph-overlay-muted);",
                    "{empty_message}"
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
                    let node_template_label = match template.as_ref() {
                        Some(template) => template.label.clone(),
                        None => node_template_id.clone(),
                    };
                    let node_category = template
                        .as_ref()
                        .and_then(|template| template.category.as_deref().map(category_label));
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
                    let node_run_state = node_run_states.get(&node.id).cloned();
                    let is_selected = selection.node_ids.iter().any(|selected| selected == &node.id);
                    let node_style = node_style(
                        position,
                        node_height(template.as_ref()),
                        is_selected,
                        node_run_state.as_ref().map(|state| state.status),
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
                            refresh_scene_metrics(scene_element, scene_rect, on_canvas_size);
                            let _ = bump_revision(wheel_revision);
                            let coords = evt.client_coordinates();
                            if should_select_node {
                                let command = GraphCommand::Select {
                                    selection: node_selection.clone(),
                                };
                                on_command.call(command);
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
                          style: if is_selected { "background: linear-gradient(90deg, transparent, var(--graph-node-topline-selected), transparent);" } else { "background: linear-gradient(90deg, transparent, var(--graph-node-topline), transparent);" },
                        }
                        div { style: "padding: 15px 16px 13px; border-bottom: 1px solid var(--graph-node-border); background: linear-gradient(180deg, var(--graph-node-header-start) 0%, var(--graph-node-header-end) 100%);",
                          div { class: "flex items-start justify-between gap-3",
                            div { class: "min-w-0",
                              if let Some(node_category) = node_category {
                                div {
                                  class: "inline-flex rounded-full border px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.2em]",
                                  style: "background: var(--graph-node-category-bg); border-color: var(--graph-node-category-border); color: var(--graph-node-category-text);",
                                  "{node_category}"
                                }
                              }
                              div { class: "mt-3 truncate text-[17px] font-semibold text-[var(--on-surface)]",
                                "{node_label}"
                              }
                              div {
                                class: "mt-1 truncate text-[12px]",
                                style: "color: var(--graph-node-subtitle);",
                                "{node_template_label}"
                              }
                            }
                            if !node_diagnostics.is_empty() {
                              span {
                                class: "inline-flex min-w-6 items-center justify-center rounded-full px-2.5 py-1 text-[11px] font-bold",
                                style: if node_diagnostics
                          .iter()
                          .any(|diagnostic| diagnostic.severity == GraphWidgetDiagnosticSeverity::Error) { "background: var(--graph-error-surface); color: var(--graph-error-text); border: 1px solid var(--graph-error-border);" } else { "background: var(--graph-warning-surface); color: var(--graph-warning-text); border: 1px solid var(--graph-warning-border);" },
                                "{node_diagnostics.len()}"
                              }
                            }
                            if let Some(run_state) = node_run_state.clone() {
                              span {
                                class: "inline-flex items-center rounded-full border px-2.5 py-1 text-[11px] font-semibold",
                                style: "{run_status_badge_style(run_state.status)}",
                                "{run_state.label.clone().unwrap_or_else(|| run_status_label(run_state.status).to_string())}"
                              }
                            }
                          }
                        }
                        div {
                          class: "grid gap-4",
                          style: "grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); padding: 16px; background: var(--graph-node-body-bg);",
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
                                              on_command.call(command);
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
                                          refresh_scene_metrics(scene_element, scene_rect, on_canvas_size);
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
                        if let Some(run_state) = node_run_state {
                          if run_state.detail.is_some() || run_state.output_summary.is_some()
                              || run_state.duration_ms.is_some()
                          {
                            div {
                              class: "border-t px-4 py-3",
                              style: "border-color: var(--graph-node-border); background: color-mix(in srgb, var(--graph-node-body-bg) 82%, var(--graph-overlay-bg) 18%);",
                              div { class: "flex items-center justify-between gap-3",
                                div {
                                  class: "truncate text-[11px] font-semibold uppercase tracking-[0.14em]",
                                  style: "color: var(--graph-overlay-muted);",
                                  "{run_state.detail.clone().unwrap_or_else(|| run_status_label(run_state.status).to_string())}"
                                }
                                if let Some(duration) = run_state.duration_ms {
                                  span {
                                    class: "shrink-0 rounded-full border px-2 py-0.5 text-[10px] font-semibold",
                                    style: "{run_status_badge_style(run_state.status)}",
                                    "{format_duration(duration)}"
                                  }
                                }
                              }
                              if let Some(output_summary) = run_state.output_summary.clone() {
                                p { class: "mt-2 text-[12px] leading-5 text-[var(--on-surface)]",
                                  "{output_summary}"
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
                  stroke: "var(--graph-connection-preview)",
                  stroke_width: "2.5",
                  stroke_dasharray: "8 6",
                }
              }
            }
          }
        }
      }
      if has_nodes {
        div { class: "pointer-events-none absolute bottom-4 right-4 z-10 flex items-center gap-2",
          span {
            class: "{canvas_badge_class}",
            style: "{canvas_badge_style}",
            "{zoom_label}"
          }
          button {
            class: "pointer-events-auto {canvas_control_class}",
            style: "{canvas_control_style}",
            onclick: move |_| {
                if let Some(viewport) = fit_viewport(
                    &document,
                    &templates,
                    scene_width,
                    scene_height,
                ) {
                    on_command
                        .call(GraphCommand::SetViewport {
                            viewport,
                        });
                }
            },
            "Fit View"
          }
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
struct MarqueeState {
  start_scene: GraphPoint,
  current_scene: GraphPoint,
  base_selection: GraphSelection,
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
  run_state: Option<GraphEdgeRunState>,
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
  let dot = if direction == PortDirection::Input { "var(--graph-port-input-dot)" } else { "var(--graph-port-output-dot)" };
  let background = if direction == PortDirection::Input { "var(--graph-port-input-bg)" } else { "var(--graph-port-output-bg)" };
  let border = if direction == PortDirection::Input { "1px solid var(--graph-port-input-border)" } else { "1px solid var(--graph-port-output-border)" };
  let text_color = if direction == PortDirection::Input { "var(--graph-port-input-text)" } else { "var(--graph-port-output-text)" };
  let cursor = if direction == PortDirection::Output { "crosshair" } else { "pointer" };

  rsx! {
    div {
      class: "inline-flex min-h-8 max-w-full items-center gap-2.5 rounded-full px-3 py-1.5",
      style: "background: {background}; border: {border}; margin-bottom: 10px; cursor: {cursor}; box-shadow: inset 0 1px 0 color-mix(in srgb, var(--on-surface) 4%, transparent);",
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

fn refresh_scene_metrics(scene_element: Signal<Option<Rc<MountedData>>>, mut scene_rect: Signal<Option<SceneRect>>, on_canvas_size: EventHandler<(f64, f64)>) {
  let Some(scene) = scene_element.peek().as_ref().cloned() else {
    return;
  };
  spawn(async move {
    let Ok(rect) = scene.get_client_rect().await else {
      return;
    };
    let next_rect = SceneRect { left: rect.origin.x, top: rect.origin.y, width: rect.width(), height: rect.height() };
    scene_rect.set(Some(next_rect));
    on_canvas_size.call((next_rect.width.max(1.0), next_rect.height.max(1.0)));
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

fn marquee_bounds(marquee: &MarqueeState) -> GraphBounds {
  GraphBounds {
    min_x: marquee.start_scene.x.min(marquee.current_scene.x),
    min_y: marquee.start_scene.y.min(marquee.current_scene.y),
    max_x: marquee.start_scene.x.max(marquee.current_scene.x),
    max_y: marquee.start_scene.y.max(marquee.current_scene.y),
  }
}

fn world_bounds_from_scene(bounds: GraphBounds, viewport: GraphViewport) -> GraphBounds {
  GraphBounds {
    min_x: (bounds.min_x - viewport.pan_x) / viewport.zoom,
    min_y: (bounds.min_y - viewport.pan_y) / viewport.zoom,
    max_x: (bounds.max_x - viewport.pan_x) / viewport.zoom,
    max_y: (bounds.max_y - viewport.pan_y) / viewport.zoom,
  }
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
  let zoom = (available_width / width).min(available_height / height).clamp(0.46, 1.04);
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

fn selection_for_marquee(
  document: &GraphDocument,
  templates: &[GraphNodeTemplate],
  preview_positions: &HashMap<String, GraphPoint>,
  world_bounds: GraphBounds,
  base_selection: &GraphSelection,
) -> GraphSelection {
  let mut node_ids = base_selection.node_ids.clone();

  for node in &document.nodes {
    let position = displayed_node_position(node, preview_positions);
    let node_bounds = GraphBounds {
      min_x: position.x,
      min_y: position.y,
      max_x: position.x + NODE_WIDTH,
      max_y: position.y + node_height(node_template(templates, &node.template_id)),
    };
    if bounds_intersect(world_bounds, node_bounds) && !node_ids.iter().any(|selected| selected == &node.id) {
      node_ids.push(node.id.clone());
    }
  }

  let node_id_set = node_ids.iter().cloned().collect::<std::collections::HashSet<_>>();
  let mut edge_ids = base_selection.edge_ids.clone();
  for edge in &document.edges {
    if node_id_set.contains(&edge.from.node_id) && node_id_set.contains(&edge.to.node_id) && !edge_ids.iter().any(|selected| selected == &edge.id) {
      edge_ids.push(edge.id.clone());
    }
  }

  let anchor = node_ids.first().cloned().map(GraphEntityRef::Node).or_else(|| edge_ids.first().cloned().map(GraphEntityRef::Edge));

  GraphSelection { anchor, node_ids, edge_ids }
}

fn bounds_intersect(a: GraphBounds, b: GraphBounds) -> bool {
  a.min_x <= b.max_x && a.max_x >= b.min_x && a.min_y <= b.max_y && a.max_y >= b.min_y
}

const FIT_VIEW_PADDING_X: f64 = 56.0;
const FIT_VIEW_PADDING_Y: f64 = 52.0;
const NODE_WIDTH: f64 = 288.0;
const NODE_HEADER_HEIGHT: f64 = 72.0;
const NODE_PORT_ROW_HEIGHT: f64 = 30.0;
const NODE_BODY_PADDING: f64 = 16.0;

fn node_style(position: GraphPoint, height: f64, is_selected: bool, run_status: Option<GraphRunStatus>) -> String {
  let border = if is_selected { "1px solid var(--graph-node-border-selected)" } else { run_status_border(run_status) };
  let box_shadow = if is_selected { "var(--graph-node-shadow-selected)" } else { run_status_shadow(run_status) };
  let background = run_status_background(run_status);
  format!(
    "position: absolute; left: {}px; top: {}px; width: {}px; min-height: {}px; border-radius: 22px; border: {}; box-shadow: {}; background: {}; overflow: hidden; user-select: none;",
    position.x, position.y, NODE_WIDTH, height, border, box_shadow, background
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

fn run_snapshot_label(snapshot: &GraphRunSnapshot) -> String {
  match snapshot.label.as_deref() {
    Some(label) if !label.trim().is_empty() => format!("{label} • {}", run_status_label(snapshot.status)),
    _ => format!("Run • {}", run_status_label(snapshot.status)),
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
      "border: 1px solid color-mix(in srgb, var(--outline-variant) 68%, transparent); background: color-mix(in srgb, var(--surface-container-high) 72%, transparent); color: var(--graph-overlay-muted); backdrop-filter: blur(16px);"
    },
    GraphRunStatus::Pending => {
      "border: 1px solid color-mix(in srgb, var(--warning) 32%, transparent); background: color-mix(in srgb, var(--warning) 14%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--warning) 18%); backdrop-filter: blur(16px);"
    },
    GraphRunStatus::Running => {
      "border: 1px solid color-mix(in srgb, var(--primary) 34%, transparent); background: color-mix(in srgb, var(--primary) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 84%, var(--primary) 16%); backdrop-filter: blur(16px);"
    },
    GraphRunStatus::Succeeded => {
      "border: 1px solid color-mix(in srgb, var(--success) 34%, transparent); background: color-mix(in srgb, var(--success) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 84%, var(--success) 16%); backdrop-filter: blur(16px);"
    },
    GraphRunStatus::Warning => {
      "border: 1px solid color-mix(in srgb, var(--warning) 34%, transparent); background: color-mix(in srgb, var(--warning) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 84%, var(--warning) 16%); backdrop-filter: blur(16px);"
    },
    GraphRunStatus::Failed => {
      "border: 1px solid color-mix(in srgb, var(--error) 36%, transparent); background: color-mix(in srgb, var(--error) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--error) 18%); backdrop-filter: blur(16px);"
    },
    GraphRunStatus::Cancelled => {
      "border: 1px solid color-mix(in srgb, var(--outline) 32%, transparent); background: color-mix(in srgb, var(--surface-container-high) 78%, transparent); color: var(--graph-overlay-text); backdrop-filter: blur(16px);"
    },
  }
}

fn run_status_background(status: Option<GraphRunStatus>) -> &'static str {
  match status {
    Some(GraphRunStatus::Running) => {
      "linear-gradient(180deg, color-mix(in srgb, var(--graph-node-bg-start) 88%, var(--primary) 12%) 0%, color-mix(in srgb, var(--graph-node-bg-end) 90%, var(--primary) 10%) 100%)"
    },
    Some(GraphRunStatus::Succeeded) => {
      "linear-gradient(180deg, color-mix(in srgb, var(--graph-node-bg-start) 90%, var(--success) 10%) 0%, color-mix(in srgb, var(--graph-node-bg-end) 92%, var(--success) 8%) 100%)"
    },
    Some(GraphRunStatus::Warning) | Some(GraphRunStatus::Pending) => {
      "linear-gradient(180deg, color-mix(in srgb, var(--graph-node-bg-start) 91%, var(--warning) 9%) 0%, color-mix(in srgb, var(--graph-node-bg-end) 93%, var(--warning) 7%) 100%)"
    },
    Some(GraphRunStatus::Failed) => {
      "linear-gradient(180deg, color-mix(in srgb, var(--graph-node-bg-start) 90%, var(--error) 10%) 0%, color-mix(in srgb, var(--graph-node-bg-end) 92%, var(--error) 8%) 100%)"
    },
    Some(GraphRunStatus::Cancelled) => {
      "linear-gradient(180deg, color-mix(in srgb, var(--graph-node-bg-start) 92%, var(--outline) 8%) 0%, color-mix(in srgb, var(--graph-node-bg-end) 94%, var(--outline) 6%) 100%)"
    },
    _ => "linear-gradient(180deg, var(--graph-node-bg-start) 0%, var(--graph-node-bg-end) 100%)",
  }
}

fn run_status_border(status: Option<GraphRunStatus>) -> &'static str {
  match status {
    Some(GraphRunStatus::Running) => "1px solid color-mix(in srgb, var(--primary) 30%, var(--graph-node-border))",
    Some(GraphRunStatus::Succeeded) => "1px solid color-mix(in srgb, var(--success) 30%, var(--graph-node-border))",
    Some(GraphRunStatus::Warning) | Some(GraphRunStatus::Pending) => "1px solid color-mix(in srgb, var(--warning) 28%, var(--graph-node-border))",
    Some(GraphRunStatus::Failed) => "1px solid color-mix(in srgb, var(--error) 30%, var(--graph-node-border))",
    Some(GraphRunStatus::Cancelled) => "1px solid color-mix(in srgb, var(--outline) 26%, var(--graph-node-border))",
    _ => "1px solid var(--graph-node-border)",
  }
}

fn run_status_shadow(status: Option<GraphRunStatus>) -> &'static str {
  match status {
    Some(GraphRunStatus::Running) => "0 0 0 1px color-mix(in srgb, var(--primary) 16%, transparent), 0 18px 42px rgba(0, 0, 0, 0.34)",
    Some(GraphRunStatus::Succeeded) => "0 0 0 1px color-mix(in srgb, var(--success) 14%, transparent), 0 18px 42px rgba(0, 0, 0, 0.32)",
    Some(GraphRunStatus::Warning) | Some(GraphRunStatus::Pending) => {
      "0 0 0 1px color-mix(in srgb, var(--warning) 14%, transparent), 0 18px 42px rgba(0, 0, 0, 0.32)"
    },
    Some(GraphRunStatus::Failed) => "0 0 0 1px color-mix(in srgb, var(--error) 16%, transparent), 0 18px 42px rgba(0, 0, 0, 0.34)",
    Some(GraphRunStatus::Cancelled) => "0 0 0 1px color-mix(in srgb, var(--outline) 12%, transparent), 0 18px 42px rgba(0, 0, 0, 0.32)",
    _ => "var(--graph-node-shadow)",
  }
}

fn edge_stroke_color(run_state: Option<&GraphEdgeRunState>, has_error: bool, is_selected: bool) -> &'static str {
  if has_error {
    return "var(--graph-edge-error)";
  }
  if let Some(run_state) = run_state {
    return match run_state.status {
      GraphRunStatus::Running => "color-mix(in srgb, var(--primary) 76%, white 8%)",
      GraphRunStatus::Succeeded => "color-mix(in srgb, var(--success) 74%, white 8%)",
      GraphRunStatus::Warning | GraphRunStatus::Pending => "color-mix(in srgb, var(--warning) 74%, white 8%)",
      GraphRunStatus::Failed => "color-mix(in srgb, var(--error) 82%, white 10%)",
      GraphRunStatus::Cancelled => "color-mix(in srgb, var(--outline) 64%, white 4%)",
      GraphRunStatus::Idle => {
        if is_selected {
          "var(--graph-edge-selected)"
        } else {
          "var(--graph-edge-default)"
        }
      },
    };
  }
  if is_selected { "var(--graph-edge-selected)" } else { "var(--graph-edge-default)" }
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
