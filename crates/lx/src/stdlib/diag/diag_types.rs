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
