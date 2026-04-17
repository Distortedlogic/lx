use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MermaidDirection {
  #[default]
  TopDown,
  LeftRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MermaidSemanticKind {
  Step,
  Agent,
  Decision,
  Tool,
  Io,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MermaidNodeMetadata {
  pub task_summary: Option<String>,
  pub prompt: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MermaidNode {
  pub id: String,
  pub semantic_kind: MermaidSemanticKind,
  pub display_label: String,
  pub subgraph_id: Option<String>,
  pub metadata: MermaidNodeMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MermaidEdge {
  pub id: String,
  pub from: String,
  pub to: String,
  pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MermaidSubgraph {
  pub id: String,
  pub title: String,
  pub parent_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MermaidChart {
  pub title: String,
  pub notes: Option<String>,
  pub direction: MermaidDirection,
  pub nodes: Vec<MermaidNode>,
  pub edges: Vec<MermaidEdge>,
  pub subgraphs: Vec<MermaidSubgraph>,
}

impl MermaidSemanticKind {
  pub fn class_name(self) -> &'static str {
    match self {
      Self::Step => "step",
      Self::Agent => "agent",
      Self::Decision => "decision",
      Self::Tool => "tool",
      Self::Io => "io",
    }
  }

  pub fn template_id(self) -> &'static str {
    match self {
      Self::Step => "mermaid_step",
      Self::Agent => "mermaid_agent",
      Self::Decision => "mermaid_decision",
      Self::Tool => "mermaid_tool",
      Self::Io => "mermaid_io",
    }
  }

  pub fn from_class_name(value: &str) -> Option<Self> {
    match value {
      "step" => Some(Self::Step),
      "agent" => Some(Self::Agent),
      "decision" => Some(Self::Decision),
      "tool" => Some(Self::Tool),
      "io" => Some(Self::Io),
      _ => None,
    }
  }

  pub fn from_template_id(value: &str) -> Option<Self> {
    match value {
      "mermaid_step" => Some(Self::Step),
      "mermaid_agent" => Some(Self::Agent),
      "mermaid_decision" => Some(Self::Decision),
      "mermaid_tool" => Some(Self::Tool),
      "mermaid_io" => Some(Self::Io),
      _ => None,
    }
  }
}

impl MermaidDirection {
  pub fn keyword(self) -> &'static str {
    match self {
      Self::TopDown => "TD",
      Self::LeftRight => "LR",
    }
  }
}

impl MermaidChart {
  pub fn node(&self, node_id: &str) -> Option<&MermaidNode> {
    self.nodes.iter().find(|node| node.id == node_id)
  }

  pub fn subgraph(&self, subgraph_id: &str) -> Option<&MermaidSubgraph> {
    self.subgraphs.iter().find(|subgraph| subgraph.id == subgraph_id)
  }
}
