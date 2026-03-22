use pane_tree::Pane;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DesktopPane {
  Terminal { id: String, working_dir: String, command: Option<String> },
  Browser { id: String, url: String, devtools: bool },
  Editor { id: String, file_path: String, language: Option<String> },
  Agent { id: String, session_id: String, model: String },
  Canvas { id: String, widget_type: String, config: serde_json::Value },
  Chart { id: String, chart_json: String, title: Option<String> },
  Voice { id: String },
}

impl Pane for DesktopPane {
  fn pane_id(&self) -> &str {
    match self {
      Self::Terminal { id, .. }
      | Self::Browser { id, .. }
      | Self::Editor { id, .. }
      | Self::Agent { id, .. }
      | Self::Canvas { id, .. }
      | Self::Chart { id, .. }
      | Self::Voice { id, .. } => id,
    }
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneKind {
  Terminal,
  Browser,
  Editor,
  Agent,
  Canvas,
  Chart,
  Voice,
}

impl DesktopPane {
  pub fn kind(&self) -> PaneKind {
    match self {
      Self::Terminal { .. } => PaneKind::Terminal,
      Self::Browser { .. } => PaneKind::Browser,
      Self::Editor { .. } => PaneKind::Editor,
      Self::Agent { .. } => PaneKind::Agent,
      Self::Canvas { .. } => PaneKind::Canvas,
      Self::Chart { .. } => PaneKind::Chart,
      Self::Voice { .. } => PaneKind::Voice,
    }
  }

  pub fn make_default(kind: PaneKind, id: String) -> Self {
    match kind {
      PaneKind::Terminal => Self::Terminal { id, working_dir: ".".into(), command: None },
      PaneKind::Browser => Self::Browser { id, url: "about:blank".into(), devtools: false },
      PaneKind::Editor => Self::Editor { id, file_path: String::new(), language: None },
      PaneKind::Agent => Self::Agent { id: id.clone(), session_id: uuid::Uuid::new_v4().to_string(), model: "claude-sonnet-4-6".into() },
      PaneKind::Canvas => Self::Canvas { id, widget_type: "markdown".into(), config: serde_json::Value::Object(Default::default()) },
      PaneKind::Chart => Self::Chart { id, chart_json: String::new(), title: None },
      PaneKind::Voice => Self::Voice { id },
    }
  }

  pub fn icon(&self) -> &'static str {
    match self {
      Self::Terminal { .. } => "\u{25B8}",
      Self::Browser { .. } => "\u{1F310}",
      Self::Editor { .. } => "\u{25C7}",
      Self::Agent { .. } => "\u{25CF}",
      Self::Canvas { .. } => "\u{25FB}",
      Self::Chart { .. } => "\u{25A3}",
      Self::Voice { .. } => "\u{1F3A4}",
    }
  }
}

impl PaneKind {
  pub const ALL: &[PaneKind] = &[PaneKind::Terminal, PaneKind::Browser, PaneKind::Editor, PaneKind::Agent, PaneKind::Canvas, PaneKind::Chart, PaneKind::Voice];

  pub fn icon(self) -> &'static str {
    match self {
      Self::Terminal => "\u{25B8}",
      Self::Browser => "\u{1F310}",
      Self::Editor => "\u{25C7}",
      Self::Agent => "\u{25CF}",
      Self::Canvas => "\u{25FB}",
      Self::Chart => "\u{25A3}",
      Self::Voice => "\u{1F3A4}",
    }
  }

  pub fn label(self) -> &'static str {
    match self {
      Self::Terminal => "Terminal",
      Self::Browser => "Browser",
      Self::Editor => "Editor",
      Self::Agent => "Agent",
      Self::Canvas => "Canvas",
      Self::Chart => "Chart",
      Self::Voice => "Voice",
    }
  }
}
