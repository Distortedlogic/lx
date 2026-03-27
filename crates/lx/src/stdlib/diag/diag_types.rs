#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
  Agent,
  Tool,
  Decision,
  Fork,
  Join,
  Loop,
  Resource,
  User,
  Io,
  Type,
}

impl NodeKind {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Agent => "agent",
      Self::Tool => "tool",
      Self::Decision => "decision",
      Self::Fork => "fork",
      Self::Join => "join",
      Self::Loop => "loop",
      Self::Resource => "resource",
      Self::User => "user",
      Self::Io => "io",
      Self::Type => "type",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeStyle {
  Solid,
  Dashed,
  Double,
}

impl EdgeStyle {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Solid => "solid",
      Self::Dashed => "dashed",
      Self::Double => "double",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
  Agent,
  Stream,
  Data,
  Io,
  Exec,
}

impl EdgeType {
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::Agent => "agent",
      Self::Stream => "stream",
      Self::Data => "data",
      Self::Io => "io",
      Self::Exec => "exec",
    }
  }
}

pub struct DiagNode {
  pub id: String,
  pub label: String,
  pub kind: NodeKind,
  pub children: Vec<DiagNode>,
  pub source_offset: Option<u32>,
}

pub struct DiagEdge {
  pub from: String,
  pub to: String,
  pub label: String,
  pub style: EdgeStyle,
  pub edge_type: EdgeType,
}

pub struct Subgraph {
  pub label: String,
  pub node_ids: Vec<String>,
}

pub struct Graph {
  pub nodes: Vec<DiagNode>,
  pub edges: Vec<DiagEdge>,
  pub subgraphs: Vec<Subgraph>,
}

use std::str::FromStr;

impl FromStr for NodeKind {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "agent" => Ok(Self::Agent),
      "tool" => Ok(Self::Tool),
      "decision" => Ok(Self::Decision),
      "fork" => Ok(Self::Fork),
      "join" => Ok(Self::Join),
      "loop" => Ok(Self::Loop),
      "resource" => Ok(Self::Resource),
      "user" => Ok(Self::User),
      "io" => Ok(Self::Io),
      "type" => Ok(Self::Type),
      other => Err(format!("unknown node kind: {other}")),
    }
  }
}

impl FromStr for EdgeStyle {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "solid" => Ok(Self::Solid),
      "dashed" => Ok(Self::Dashed),
      "double" => Ok(Self::Double),
      other => Err(format!("unknown edge style: {other}")),
    }
  }
}

impl FromStr for EdgeType {
  type Err = String;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "agent" => Ok(Self::Agent),
      "stream" => Ok(Self::Stream),
      "data" => Ok(Self::Data),
      "io" => Ok(Self::Io),
      "exec" => Ok(Self::Exec),
      other => Err(format!("unknown edge type: {other}")),
    }
  }
}
