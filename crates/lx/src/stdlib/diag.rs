use std::sync::Arc;

use indexmap::IndexMap;

use crate::backends::RuntimeCtx;
use crate::builtins::mk;
use crate::error::LxError;
use crate::span::Span;
use crate::value::Value;

use crate::ast::Program;

use super::diag_walk::{DiagEdge, DiagNode, Graph, Walker};

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
    walker.walk_program(program);
    to_mermaid(&walker.into_graph())
}

fn extract_graph(src: &str, span: Span) -> Result<Graph, LxError> {
    let tokens = crate::lexer::lex(src)
        .map_err(|e| LxError::runtime(format!("diag: lex error: {e}"), span))?;
    let program = crate::parser::parse(tokens)
        .map_err(|e| LxError::runtime(format!("diag: parse error: {e}"), span))?;
    let mut walker = Walker::new();
    walker.walk_program(&program);
    Ok(walker.into_graph())
}

fn node_to_value(node: &DiagNode) -> Value {
    let children: Vec<Value> = node.children.iter().map(node_to_value).collect();
    let mut fields = IndexMap::new();
    fields.insert("id".into(), Value::Str(Arc::from(node.id.as_str())));
    fields.insert("label".into(), Value::Str(Arc::from(node.label.as_str())));
    fields.insert("kind".into(), Value::Str(Arc::from(node.kind.as_str())));
    fields.insert("children".into(), Value::List(Arc::new(children)));
    Value::Record(Arc::new(fields))
}

fn edge_to_value(edge: &DiagEdge) -> Value {
    let mut fields = IndexMap::new();
    fields.insert("from".into(), Value::Str(Arc::from(edge.from.as_str())));
    fields.insert("to".into(), Value::Str(Arc::from(edge.to.as_str())));
    fields.insert("label".into(), Value::Str(Arc::from(edge.label.as_str())));
    fields.insert("style".into(), Value::Str(Arc::from(edge.style.as_str())));
    Value::Record(Arc::new(fields))
}

fn graph_to_value(graph: &Graph) -> Value {
    let nodes: Vec<Value> = graph.nodes.iter().map(node_to_value).collect();
    let edges: Vec<Value> = graph.edges.iter().map(edge_to_value).collect();
    let mut fields = IndexMap::new();
    fields.insert("nodes".into(), Value::List(Arc::new(nodes)));
    fields.insert("edges".into(), Value::List(Arc::new(edges)));
    Value::Record(Arc::new(fields))
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
    Ok(Graph { nodes, edges })
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

fn to_mermaid(graph: &Graph) -> String {
    let mut out = String::from("flowchart TD\n");
    for node in &graph.nodes {
        let shape = match node.kind.as_str() {
            "agent" => format!("    {}[\"{}\"]", node.id, node.label),
            "tool" => format!("    {}([\"{}\"])", node.id, node.label),
            "decision" => format!("    {}{{\"{}\"}}", node.id, node.label),
            "fork" | "join" => format!("    {}[[\"{}\"]]\n", node.id, node.label),
            _ => format!("    {}[\"{}\"]", node.id, node.label),
        };
        out.push_str(&shape);
        out.push('\n');
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
    for node in &graph.nodes {
        match node.kind.as_str() {
            "agent" => out.push_str(&format!("    class {} agent\n", node.id)),
            "tool" => out.push_str(&format!("    class {} tool\n", node.id)),
            "decision" => out.push_str(&format!("    class {} decision\n", node.id)),
            _ => {}
        }
    }
    out
}
