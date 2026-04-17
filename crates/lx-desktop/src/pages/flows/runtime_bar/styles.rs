use lx_graph_editor::protocol::GraphRunStatus;

pub fn run_status_label(status: GraphRunStatus) -> &'static str {
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

pub fn run_snapshot_badge_style(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => {
      "border-color: color-mix(in srgb, var(--outline-variant) 70%, transparent); background: color-mix(in srgb, var(--surface-container-high) 76%, transparent); color: var(--on-surface-variant);"
    },
    GraphRunStatus::Pending => {
      "border-color: color-mix(in srgb, var(--warning) 32%, transparent); background: color-mix(in srgb, var(--warning) 14%, transparent); color: color-mix(in srgb, var(--on-surface) 80%, var(--warning) 20%);"
    },
    GraphRunStatus::Running => {
      "border-color: color-mix(in srgb, var(--primary) 34%, transparent); background: color-mix(in srgb, var(--primary) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--primary) 18%);"
    },
    GraphRunStatus::Succeeded => {
      "border-color: color-mix(in srgb, var(--success) 34%, transparent); background: color-mix(in srgb, var(--success) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--success) 18%);"
    },
    GraphRunStatus::Warning => {
      "border-color: color-mix(in srgb, var(--warning) 34%, transparent); background: color-mix(in srgb, var(--warning) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--warning) 18%);"
    },
    GraphRunStatus::Failed => {
      "border-color: color-mix(in srgb, var(--error) 34%, transparent); background: color-mix(in srgb, var(--error) 16%, transparent); color: color-mix(in srgb, var(--on-surface) 82%, var(--error) 18%);"
    },
    GraphRunStatus::Cancelled => {
      "border-color: color-mix(in srgb, var(--outline) 34%, transparent); background: color-mix(in srgb, var(--surface-container-high) 74%, transparent); color: var(--on-surface-variant);"
    },
  }
}

pub fn run_snapshot_surface_style(status: GraphRunStatus) -> &'static str {
  match status {
    GraphRunStatus::Idle => {
      "border-color: color-mix(in srgb, var(--outline-variant) 58%, transparent); background: color-mix(in srgb, var(--surface-container-high) 44%, transparent);"
    },
    GraphRunStatus::Pending => {
      "border-color: color-mix(in srgb, var(--warning) 22%, transparent); background: color-mix(in srgb, var(--warning) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Running => {
      "border-color: color-mix(in srgb, var(--primary) 22%, transparent); background: color-mix(in srgb, var(--primary) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Succeeded => {
      "border-color: color-mix(in srgb, var(--success) 22%, transparent); background: color-mix(in srgb, var(--success) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Warning => {
      "border-color: color-mix(in srgb, var(--warning) 22%, transparent); background: color-mix(in srgb, var(--warning) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Failed => {
      "border-color: color-mix(in srgb, var(--error) 22%, transparent); background: color-mix(in srgb, var(--error) 8%, var(--surface-container-low));"
    },
    GraphRunStatus::Cancelled => {
      "border-color: color-mix(in srgb, var(--outline) 22%, transparent); background: color-mix(in srgb, var(--surface-container-high) 48%, transparent);"
    },
  }
}

pub fn format_duration(duration_ms: u64) -> String {
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
