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

#[derive(Clone, Copy, Debug, PartialEq)]
struct CanvasFrame {
  left: f64,
  top: f64,
  width: f64,
  height: f64,
}

pub(crate) fn fit_viewport(bounds: GraphBounds, scene_width: f64, scene_height: f64, safe_area: GraphCanvasSafeArea) -> GraphViewport {
  let width = (bounds.max_x - bounds.min_x).max(1.0);
  let height = (bounds.max_y - bounds.min_y).max(1.0);
  let frame = visible_frame(scene_width, scene_height, safe_area);
  let zoom = (frame.width / width).min(frame.height / height).clamp(0.46, 1.04);
  let center_x = bounds.min_x + width * 0.5;
  let center_y = bounds.min_y + height * 0.5;

  GraphViewport { pan_x: frame.left + frame.width * 0.5 - center_x * zoom, pan_y: frame.top + frame.height * 0.5 - center_y * zoom, zoom }
}

pub(crate) fn viewport_needs_fit(bounds: GraphBounds, viewport: GraphViewport, scene_width: f64, scene_height: f64, safe_area: GraphCanvasSafeArea) -> bool {
  let frame = visible_frame(scene_width, scene_height, safe_area);
  let screen_bounds = screen_bounds(bounds, viewport);
  let horizontal_inset = VIEWPORT_EDGE_INSET.min(frame.width * 0.24);
  let vertical_inset = VIEWPORT_EDGE_INSET.min(frame.height * 0.24);
  let width_fill = (screen_bounds.max_x - screen_bounds.min_x).max(1.0) / frame.width.max(1.0);
  let height_fill = (screen_bounds.max_y - screen_bounds.min_y).max(1.0) / frame.height.max(1.0);
  let frame_right = frame.left + frame.width;
  let frame_bottom = frame.top + frame.height;
  let frame_center_x = frame.left + frame.width * 0.5;
  let frame_center_y = frame.top + frame.height * 0.5;
  let screen_center_x = (screen_bounds.min_x + screen_bounds.max_x) * 0.5;
  let screen_center_y = (screen_bounds.min_y + screen_bounds.max_y) * 0.5;

  screen_bounds.min_x < frame.left + horizontal_inset
    || screen_bounds.max_x > frame_right - horizontal_inset
    || screen_bounds.min_y < frame.top + vertical_inset
    || screen_bounds.max_y > frame_bottom - vertical_inset
    || (screen_center_x - frame_center_x).abs() > horizontal_inset * 0.5
    || (screen_center_y - frame_center_y).abs() > vertical_inset * 0.5
    || width_fill < MIN_WIDTH_FILL
    || height_fill < MIN_HEIGHT_FILL
}

fn resolved_safe_area(safe_area: GraphCanvasSafeArea) -> GraphCanvasSafeArea {
  GraphCanvasSafeArea {
    top: safe_area.top.max(FIT_VIEW_PADDING_Y),
    right: safe_area.right.max(FIT_VIEW_PADDING_X),
    bottom: safe_area.bottom.max(FIT_VIEW_PADDING_Y),
    left: safe_area.left.max(FIT_VIEW_PADDING_X),
  }
}

fn visible_frame(scene_width: f64, scene_height: f64, safe_area: GraphCanvasSafeArea) -> CanvasFrame {
  let scene_width = scene_width.max(1.0);
  let scene_height = scene_height.max(1.0);
  let safe_area = resolved_safe_area(safe_area);
  let width = (scene_width - safe_area.left - safe_area.right).max(1.0);
  let height = (scene_height - safe_area.top - safe_area.bottom).max(1.0);

  CanvasFrame { left: safe_area.left.min(scene_width - width), top: safe_area.top.min(scene_height - height), width, height }
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
    GraphBounds { min_x: 0.0, min_y: 0.0, max_x: 640.0, max_y: 320.0 }
  }

  #[test]
  fn fit_viewport_centers_bounds_inside_the_visible_frame() {
    let safe_area = GraphCanvasSafeArea { top: 56.0, left: 112.0, ..Default::default() };
    let viewport = fit_viewport(sample_bounds(), 1280.0, 720.0, safe_area);
    let frame = visible_frame(1280.0, 720.0, safe_area);
    let screen = screen_bounds(sample_bounds(), viewport);

    assert!(screen.min_x >= frame.left);
    assert!(screen.min_y >= frame.top);
    assert_close((screen.min_x + screen.max_x) * 0.5, frame.left + frame.width * 0.5);
    assert_close((screen.min_y + screen.max_y) * 0.5, frame.top + frame.height * 0.5);
  }

  #[test]
  fn viewport_needs_fit_when_a_new_safe_area_claims_the_top_left_corner() {
    let bounds = GraphBounds { min_x: 0.0, min_y: 0.0, max_x: 980.0, max_y: 300.0 };
    let viewport = fit_viewport(bounds, 1280.0, 720.0, GraphCanvasSafeArea::default());

    assert!(!viewport_needs_fit(bounds, viewport, 1280.0, 720.0, GraphCanvasSafeArea::default()));
    assert!(viewport_needs_fit(bounds, viewport, 1280.0, 720.0, GraphCanvasSafeArea { top: 56.0, left: 112.0, ..Default::default() },));
  }

  #[test]
  fn fit_viewport_respects_a_wide_left_drawer_safe_area() {
    let safe_area = GraphCanvasSafeArea { top: 56.0, left: 336.0, ..Default::default() };
    let viewport = fit_viewport(sample_bounds(), 1280.0, 720.0, safe_area);

    assert!(!viewport_needs_fit(sample_bounds(), viewport, 1280.0, 720.0, safe_area));
  }
}
