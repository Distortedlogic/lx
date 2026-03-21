pub struct DiagNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub children: Vec<DiagNode>,
    pub source_offset: Option<u32>,
}

pub struct DiagEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub style: String,
    pub edge_type: String,
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
