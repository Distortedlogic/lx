use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::span::Span;
use crate::value::Value;

use crate::ast::Program;
use crate::visitor::AstVisitor;

use super::diag_walk::{DiagEdge, DiagNode, Graph, Subgraph, Walker};

pub fn build() -> IndexMap<String, Value> {
    let mut m = IndexMap::new();
    m.insert("extract".into(), mk("diag.extract", 1, bi_extract));
    m.insert(
        "extract_file".into(),
        mk("diag.extract_file", 1, bi_extract_file),
    );
    m.insert("to_mermaid".into(), mk("diag.to_mermaid", 1, bi_to_mermaid));
    m
}

fn bi_extract(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let src = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("diag.extract expects Str", span))?;
    let graph = extract_graph(src, span)?;
    Ok(graph_to_value(&graph))
}

fn bi_extract_file(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let path = args[0]
        .as_str()
        .ok_or_else(|| LxError::type_err("diag.extract_file expects Str", span))?;
    let src = std::fs::read_to_string(path)
        .map_err(|e| LxError::runtime(format!("diag.extract_file: {e}"), span))?;
    let graph = extract_graph(&src, span)?;
    Ok(graph_to_value(&graph))
}

fn bi_to_mermaid(args: &[Value], span: Span, _ctx: &Arc<RuntimeCtx>) -> Result<Value, LxError> {
    let graph = value_to_graph(&args[0], span)?;
    Ok(Value::Str(Arc::from(to_mermaid(&graph).as_str())))
}

pub fn extract_mermaid(program: &Program) -> String {
    let mut walker = Walker::new();
    walker.visit_program(program);
    to_mermaid(&walker.into_graph())
}

fn extract_graph(src: &str, span: Span) -> Result<Graph, LxError> {
    let tokens = crate::lexer::lex(src)
        .map_err(|e| LxError::runtime(format!("diag: lex error: {e}"), span))?;
    let program = crate::parser::parse(tokens)
        .map_err(|e| LxError::runtime(format!("diag: parse error: {e}"), span))?;
    let mut walker = Walker::new();
    walker.visit_program(&program);
    Ok(walker.into_graph())
}

fn node_to_value(node: &DiagNode) -> Value {
    let children: Vec<Value> = node.children.iter().map(node_to_value).collect();
    record! {
        "id" => Value::Str(Arc::from(node.id.as_str())),
        "label" => Value::Str(Arc::from(node.label.as_str())),
        "kind" => Value::Str(Arc::from(node.kind.as_str())),
        "children" => Value::List(Arc::new(children)),
    }
}

fn edge_to_value(edge: &DiagEdge) -> Value {
    record! {
        "from" => Value::Str(Arc::from(edge.from.as_str())),
        "to" => Value::Str(Arc::from(edge.to.as_str())),
        "label" => Value::Str(Arc::from(edge.label.as_str())),
        "style" => Value::Str(Arc::from(edge.style.as_str())),
    }
}

fn graph_to_value(graph: &Graph) -> Value {
    let nodes: Vec<Value> = graph.nodes.iter().map(node_to_value).collect();
    let edges: Vec<Value> = graph.edges.iter().map(edge_to_value).collect();
    record! {
        "nodes" => Value::List(Arc::new(nodes)),
        "edges" => Value::List(Arc::new(edges)),
    }
}

fn value_to_graph(val: &Value, span: Span) -> Result<Graph, LxError> {
    let Value::Record(rec) = val else {
        return Err(LxError::type_err(
            "diag.to_mermaid expects Graph record",
            span,
        ));
    };
    let nodes_val = rec
        .get("nodes")
        .ok_or_else(|| LxError::type_err("graph missing 'nodes'", span))?;
    let edges_val = rec
        .get("edges")
        .ok_or_else(|| LxError::type_err("graph missing 'edges'", span))?;
    let nodes = match nodes_val {
        Value::List(l) => l
            .iter()
            .map(|v| value_to_node(v, span))
            .collect::<Result<_, _>>()?,
        _ => return Err(LxError::type_err("graph.nodes must be List", span)),
    };
    let edges = match edges_val {
        Value::List(l) => l
            .iter()
            .map(|v| value_to_edge(v, span))
            .collect::<Result<_, _>>()?,
        _ => return Err(LxError::type_err("graph.edges must be List", span)),
    };
    Ok(Graph {
        nodes,
        edges,
        subgraphs: vec![],
    })
}

fn value_to_node(val: &Value, span: Span) -> Result<DiagNode, LxError> {
    let Value::Record(rec) = val else {
        return Err(LxError::type_err("node must be Record", span));
    };
    let str_field = |k: &str| -> Result<String, LxError> {
        rec.get(k)
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| LxError::type_err(format!("node missing '{k}'"), span))
    };
    let children = match rec.get("children") {
        Some(Value::List(l)) => l
            .iter()
            .map(|v| value_to_node(v, span))
            .collect::<Result<_, _>>()?,
        _ => vec![],
    };
    Ok(DiagNode {
        id: str_field("id")?,
        label: str_field("label")?,
        kind: str_field("kind")?,
        children,
    })
}

fn value_to_edge(val: &Value, span: Span) -> Result<DiagEdge, LxError> {
    let Value::Record(rec) = val else {
        return Err(LxError::type_err("edge must be Record", span));
    };
    let str_field = |k: &str| -> Result<String, LxError> {
        rec.get(k)
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| LxError::type_err(format!("edge missing '{k}'"), span))
    };
    Ok(DiagEdge {
        from: str_field("from")?,
        to: str_field("to")?,
        label: str_field("label")?,
        style: str_field("style")?,
    })
}

fn node_shape(node: &DiagNode, indent: &str) -> String {
    match node.kind.as_str() {
        "agent" => format!("{indent}{}[\"{}\"]", node.id, node.label),
        "tool" => format!("{indent}{}([\"{}\"])", node.id, node.label),
        "decision" => format!("{indent}{}{{\"{}\"}}", node.id, node.label),
        "fork" | "join" => format!("{indent}{}[[\"{}\"]]", node.id, node.label),
        "loop" => format!("{indent}{}{{{{\"{}\"}}}}", node.id, node.label),
        "resource" => format!("{indent}{}[(\"{}\")]", node.id, node.label),
        "user" => format!("{indent}{}[/\"{}\"\\]", node.id, node.label),
        "io" => format!("{indent}{}>\"{}\"]", node.id, node.label),
        "type" => format!("{indent}{}((\"{}\"))", node.id, node.label),
        _ => format!("{indent}{}[\"{}\"]", node.id, node.label),
    }
}

fn to_mermaid(graph: &Graph) -> String {
    let mut out = String::from("flowchart TD\n");
    let sg_ids: std::collections::HashSet<&str> = graph
        .subgraphs
        .iter()
        .flat_map(|sg| sg.node_ids.iter().map(|s| s.as_str()))
        .collect();
    for sg in &graph.subgraphs {
        emit_subgraph(&mut out, sg, &graph.nodes);
    }
    for node in &graph.nodes {
        if !sg_ids.contains(node.id.as_str()) {
            out.push_str(&node_shape(node, "    "));
            out.push('\n');
        }
    }
    for edge in &graph.edges {
        let arrow = match edge.style.as_str() {
            "dashed" => "-.->",
            "double" => "==>",
            _ => "-->",
        };
        if edge.label.is_empty() {
            out.push_str(&format!("    {} {} {}\n", edge.from, arrow, edge.to));
        } else {
            out.push_str(&format!(
                "    {} {}|\"{}\"| {}\n",
                edge.from, arrow, edge.label, edge.to
            ));
        }
    }
    out.push_str("    classDef agent fill:#e1f5fe,stroke:#0288d1\n");
    out.push_str("    classDef tool fill:#f3e5f5,stroke:#7b1fa2\n");
    out.push_str("    classDef decision fill:#fff3e0,stroke:#ef6c00\n");
    out.push_str("    classDef loop fill:#e8f5e9,stroke:#388e3c\n");
    out.push_str("    classDef resource fill:#fce4ec,stroke:#c62828\n");
    out.push_str("    classDef user fill:#ede7f6,stroke:#4527a0\n");
    out.push_str("    classDef io fill:#e0f2f1,stroke:#00695c\n");
    out.push_str("    classDef type fill:#f5f5f5,stroke:#616161\n");
    for node in &graph.nodes {
        let class = match node.kind.as_str() {
            "agent" | "tool" | "decision" | "loop" | "resource" | "user" | "io" | "type" => {
                Some(node.kind.as_str())
            }
            "fork" | "join" => Some("agent"),
            _ => None,
        };
        if let Some(c) = class {
            out.push_str(&format!("    class {} {c}\n", node.id));
        }
    }
    out
}

fn emit_subgraph(out: &mut String, sg: &Subgraph, nodes: &[DiagNode]) {
    out.push_str(&format!(
        "    subgraph sg_{} [\"{}\"]\n",
        sg.label, sg.label
    ));
    for nid in &sg.node_ids {
        if let Some(node) = nodes.iter().find(|n| n.id == *nid) {
            out.push_str(&node_shape(node, "        "));
            out.push('\n');
        }
    }
    out.push_str("    end\n");
}
