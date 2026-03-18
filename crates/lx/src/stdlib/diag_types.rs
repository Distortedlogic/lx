pub(crate) struct DiagNode {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub children: Vec<DiagNode>,
}

pub(crate) struct DiagEdge {
    pub from: String,
    pub to: String,
    pub label: String,
    pub style: String,
}

pub(crate) struct Subgraph {
    pub label: String,
    pub node_ids: Vec<String>,
}

pub(crate) struct Graph {
    pub nodes: Vec<DiagNode>,
    pub edges: Vec<DiagEdge>,
    pub subgraphs: Vec<Subgraph>,
}
