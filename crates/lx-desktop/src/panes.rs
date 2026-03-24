use common_pane_tree::Pane;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DesktopPane {
  Terminal { id: String, working_dir: String, command: Option<String>, name: Option<String> },
  Browser { id: String, url: String, devtools: bool, name: Option<String> },
  Editor { id: String, file_path: String, language: Option<String>, name: Option<String> },
  Agent { id: String, session_id: String, model: String, name: Option<String> },
  Canvas { id: String, widget_type: String, config: serde_json::Value, name: Option<String> },
  Chart { id: String, chart_json: String, title: Option<String>, name: Option<String> },
}

impl Pane for DesktopPane {
  fn pane_id(&self) -> &str {
    match self {
      Self::Terminal { id, .. }
      | Self::Browser { id, .. }
      | Self::Editor { id, .. }
      | Self::Agent { id, .. }
      | Self::Canvas { id, .. }
      | Self::Chart { id, .. } => id,
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
    }
  }

  pub fn name(&self) -> Option<&str> {
    match self {
      Self::Terminal { name, .. }
      | Self::Browser { name, .. }
      | Self::Editor { name, .. }
      | Self::Agent { name, .. }
      | Self::Canvas { name, .. }
      | Self::Chart { name, .. } => name.as_deref(),
    }
  }

  pub fn make_default(kind: PaneKind, id: String) -> Self {
    match kind {
      PaneKind::Terminal => Self::Terminal { id, working_dir: ".".into(), command: None, name: None },
      PaneKind::Browser => Self::Browser { id, url: "about:blank".into(), devtools: false, name: None },
      PaneKind::Editor => Self::Editor { id, file_path: String::new(), language: None, name: None },
      PaneKind::Agent => Self::Agent { id: id.clone(), session_id: uuid::Uuid::new_v4().to_string(), model: "claude-sonnet-4-6".into(), name: None },
      PaneKind::Canvas => Self::Canvas { id, widget_type: "markdown".into(), config: serde_json::Value::Object(Default::default()), name: None },
      PaneKind::Chart => Self::Chart { id, chart_json: String::new(), title: None, name: None },
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
    }
  }
}

impl PaneKind {
  pub const ALL: &[PaneKind] = &[PaneKind::Terminal, PaneKind::Browser, PaneKind::Editor, PaneKind::Agent, PaneKind::Canvas, PaneKind::Chart];

  pub fn icon(self) -> &'static str {
    match self {
      Self::Terminal => "\u{25B8}",
      Self::Browser => "\u{1F310}",
      Self::Editor => "\u{25C7}",
      Self::Agent => "\u{25CF}",
      Self::Canvas => "\u{25FB}",
      Self::Chart => "\u{25A3}",
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
    }
  }
}
