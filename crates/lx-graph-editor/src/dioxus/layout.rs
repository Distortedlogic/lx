use crate::model::GraphViewport;

const FIT_VIEW_PADDING_X: f64 = 56.0;
const FIT_VIEW_PADDING_Y: f64 = 52.0;
const VIEWPORT_EDGE_INSET: f64 = 36.0;
const MIN_WIDTH_FILL: f64 = 0.56;
const MIN_HEIGHT_FILL: f64 = 0.34;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GraphCanvasSafeArea {
  pub top: f64,
  pub right: f64,
  pub bottom: f64,
  pub left: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct GraphBounds {
  pub min_x: f64,
  pub min_y: f64,
  pub max_x: f64,
  pub max_y: f64,
}

pub(crate) fn scene_frame_style(safe_area: GraphCanvasSafeArea) -> String {
  let safe_area = resolved_safe_area(safe_area);
  format!("left: {}px; top: {}px; right: {}px; bottom: {}px;", safe_area.left, safe_area.top, safe_area.right, safe_area.bottom,)
}

pub(crate) fn fit_viewport(bounds: GraphBounds, scene_width: f64, scene_height: f64) -> GraphViewport {
  let width = (bounds.max_x - bounds.min_x).max(1.0);
  let height = (bounds.max_y - bounds.min_y).max(1.0);
  let available_width = (scene_width - FIT_VIEW_PADDING_X * 2.0).max(280.0);
  let available_height = (scene_height - FIT_VIEW_PADDING_Y * 2.0).max(220.0);
  let zoom = (available_width / width).min(available_height / height).clamp(0.46, 1.04);
  let center_x = bounds.min_x + width * 0.5;
  let center_y = bounds.min_y + height * 0.5;

  GraphViewport { pan_x: scene_width * 0.5 - center_x * zoom, pan_y: scene_height * 0.5 - center_y * zoom, zoom }
}

pub(crate) fn viewport_needs_fit(bounds: GraphBounds, viewport: GraphViewport, scene_width: f64, scene_height: f64) -> bool {
  let screen_bounds = screen_bounds(bounds, viewport);
  let horizontal_inset = VIEWPORT_EDGE_INSET.min(scene_width * 0.24);
  let vertical_inset = VIEWPORT_EDGE_INSET.min(scene_height * 0.24);
  let width_fill = (screen_bounds.max_x - screen_bounds.min_x).max(1.0) / scene_width.max(1.0);
  let height_fill = (screen_bounds.max_y - screen_bounds.min_y).max(1.0) / scene_height.max(1.0);

  screen_bounds.min_x < horizontal_inset
    || screen_bounds.max_x > scene_width - horizontal_inset
    || screen_bounds.min_y < vertical_inset
    || screen_bounds.max_y > scene_height - vertical_inset
    || width_fill < MIN_WIDTH_FILL
    || height_fill < MIN_HEIGHT_FILL
}

fn resolved_safe_area(safe_area: GraphCanvasSafeArea) -> GraphCanvasSafeArea {
  GraphCanvasSafeArea { top: safe_area.top.max(0.0), right: safe_area.right.max(0.0), bottom: safe_area.bottom.max(0.0), left: safe_area.left.max(0.0) }
}

fn screen_bounds(bounds: GraphBounds, viewport: GraphViewport) -> GraphBounds {
  GraphBounds {
    min_x: bounds.min_x * viewport.zoom + viewport.pan_x,
    min_y: bounds.min_y * viewport.zoom + viewport.pan_y,
    max_x: bounds.max_x * viewport.zoom + viewport.pan_x,
    max_y: bounds.max_y * viewport.zoom + viewport.pan_y,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn assert_close(left: f64, right: f64) {
    assert!((left - right).abs() < 1e-6, "left={left} right={right}");
  }

  fn sample_bounds() -> GraphBounds {
    GraphBounds { min_x: 0.0, min_y: 0.0, max_x: 980.0, max_y: 300.0 }
  }

  #[test]
  fn fit_viewport_centers_bounds_inside_the_scene() {
    let viewport = fit_viewport(sample_bounds(), 1280.0, 720.0);
    let screen = screen_bounds(sample_bounds(), viewport);

    assert_close((screen.min_x + screen.max_x) * 0.5, 640.0);
    assert_close((screen.min_y + screen.max_y) * 0.5, 360.0);
  }

  #[test]
  fn viewport_needs_fit_detects_bounds_outside_the_scene() {
    let viewport = fit_viewport(sample_bounds(), 1280.0, 720.0);

    assert!(!viewport_needs_fit(sample_bounds(), viewport, 1280.0, 720.0));
    assert!(viewport_needs_fit(sample_bounds(), GraphViewport { pan_x: viewport.pan_x - 160.0, ..viewport }, 1280.0, 720.0));
  }

  #[test]
  fn scene_frame_style_reserves_top_band_and_left_drawer_space() {
    let style = scene_frame_style(GraphCanvasSafeArea { top: 56.0, left: 336.0, ..Default::default() });

    assert_eq!(style, "left: 336px; top: 56px; right: 0px; bottom: 0px;");
  }
}
