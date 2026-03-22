mod echart;
mod mermaid;

use std::sync::Arc;

use indexmap::IndexMap;

use crate::builtins::mk;
use crate::error::LxError;
use crate::record;
use crate::runtime::RuntimeCtx;
use crate::value::LxVal;
use miette::SourceSpan;

use crate::ast::Program;
use crate::visitor::AstVisitor;

use super::diag_walk::{DiagEdge, DiagNode, EdgeStyle, EdgeType, Graph, NodeKind, Walker};
use echart::graph_to_echart_json;
use mermaid::to_mermaid;

pub fn build() -> IndexMap<String, LxVal> {
  let mut m = IndexMap::new();
  m.insert("extract".into(), mk("diag.extract", 1, bi_extract));
  m.insert("extract_file".into(), mk("diag.extract_file", 1, bi_extract_file));
  m.insert("to_mermaid".into(), mk("diag.to_mermaid", 1, bi_to_mermaid));
  m.insert("to_graph_chart".into(), mk("diag.to_graph_chart", 1, bi_to_graph_chart));
  m
}

fn bi_extract(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let src = args[0].require_str("diag.extract", span)?;
  let graph = extract_graph(src, span)?;
  Ok(graph_to_value(&graph))
}

fn bi_extract_file(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let path = args[0].require_str("diag.extract_file", span)?;
  let src = std::fs::read_to_string(path).map_err(|e| LxError::runtime(format!("diag.extract_file: {e}"), span))?;
  let graph = extract_graph(&src, span)?;
  Ok(graph_to_value(&graph))
}

fn bi_to_mermaid(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let graph = value_to_graph(&args[0], span)?;
  Ok(LxVal::str(to_mermaid(&graph).as_str()))
}

fn bi_to_graph_chart(args: &[LxVal], span: SourceSpan, _ctx: &Arc<RuntimeCtx>) -> Result<LxVal, LxError> {
  let graph = value_to_graph(&args[0], span)?;
  Ok(LxVal::str(graph_to_echart_json(&graph).as_str()))
}

pub fn extract_mermaid(program: &Program) -> String {
  let mut walker = Walker::new();
  walker.visit_program(program);
  to_mermaid(&walker.into_graph())
}

pub fn extract_echart_json(program: &Program) -> String {
  let mut walker = Walker::new();
  walker.visit_program(program);
  let graph = walker.into_graph();
  graph_to_echart_json(&graph)
}

fn extract_graph(src: &str, span: SourceSpan) -> Result<Graph, LxError> {
  let tokens = crate::lexer::lex(src).map_err(|e| LxError::runtime(format!("diag: lex error: {e}"), span))?;
  let program = crate::parser::parse(tokens).map_err(|e| LxError::runtime(format!("diag: parse error: {e}"), span))?;
  let mut walker = Walker::new();
  walker.visit_program(&program);
  Ok(walker.into_graph())
}

fn node_to_value(node: &DiagNode) -> LxVal {
  let children: Vec<LxVal> = node.children.iter().map(node_to_value).collect();
  let offset_val = node.source_offset.map_or(LxVal::None, LxVal::int);
  record! {
      "id" => LxVal::str(node.id.as_str()),
      "label" => LxVal::str(node.label.as_str()),
      "kind" => LxVal::str(node.kind.as_str()),
      "children" => LxVal::list(children),
      "source_offset" => offset_val,
  }
}

fn edge_to_value(edge: &DiagEdge) -> LxVal {
  record! {
      "from" => LxVal::str(edge.from.as_str()),
      "to" => LxVal::str(edge.to.as_str()),
      "label" => LxVal::str(edge.label.as_str()),
      "style" => LxVal::str(edge.style.as_str()),
      "edge_type" => LxVal::str(edge.edge_type.as_str()),
  }
}

fn graph_to_value(graph: &Graph) -> LxVal {
  let nodes: Vec<LxVal> = graph.nodes.iter().map(node_to_value).collect();
  let edges: Vec<LxVal> = graph.edges.iter().map(edge_to_value).collect();
  record! {
      "nodes" => LxVal::list(nodes),
      "edges" => LxVal::list(edges),
  }
}

fn parse_node_kind(s: &str) -> Result<NodeKind, String> {
  match s {
    "agent" => Ok(NodeKind::Agent),
    "tool" => Ok(NodeKind::Tool),
    "decision" => Ok(NodeKind::Decision),
    "fork" => Ok(NodeKind::Fork),
    "join" => Ok(NodeKind::Join),
    "loop" => Ok(NodeKind::Loop),
    "resource" => Ok(NodeKind::Resource),
    "user" => Ok(NodeKind::User),
    "io" => Ok(NodeKind::Io),
    "type" => Ok(NodeKind::Type),
    other => Err(format!("unknown node kind: {other}")),
  }
}

fn parse_edge_style(s: &str) -> Result<EdgeStyle, String> {
  match s {
    "solid" => Ok(EdgeStyle::Solid),
    "dashed" => Ok(EdgeStyle::Dashed),
    "double" => Ok(EdgeStyle::Double),
    other => Err(format!("unknown edge style: {other}")),
  }
}

fn parse_edge_type(s: &str) -> Result<EdgeType, String> {
  match s {
    "agent" => Ok(EdgeType::Agent),
    "stream" => Ok(EdgeType::Stream),
    "data" => Ok(EdgeType::Data),
    "io" => Ok(EdgeType::Io),
    "exec" => Ok(EdgeType::Exec),
    other => Err(format!("unknown edge type: {other}")),
  }
}

fn value_to_graph(val: &LxVal, span: SourceSpan) -> Result<Graph, LxError> {
  let LxVal::Record(rec) = val else {
    return Err(LxError::type_err("diag.to_mermaid expects Graph record", span));
  };
  let nodes_val = rec.get("nodes").ok_or_else(|| LxError::type_err("graph missing 'nodes'", span))?;
  let edges_val = rec.get("edges").ok_or_else(|| LxError::type_err("graph missing 'edges'", span))?;
  let nodes = match nodes_val {
    LxVal::List(l) => l.iter().map(|v| value_to_node(v, span)).collect::<Result<_, _>>()?,
    _ => return Err(LxError::type_err("graph.nodes must be List", span)),
  };
  let edges = match edges_val {
    LxVal::List(l) => l.iter().map(|v| value_to_edge(v, span)).collect::<Result<_, _>>()?,
    _ => return Err(LxError::type_err("graph.edges must be List", span)),
  };
  Ok(Graph { nodes, edges, subgraphs: vec![] })
}

fn value_to_node(val: &LxVal, span: SourceSpan) -> Result<DiagNode, LxError> {
  let LxVal::Record(rec) = val else {
    return Err(LxError::type_err("node must be Record", span));
  };
  let str_field = |k: &str| -> Result<String, LxError> {
    rec.get(k).and_then(|v| v.as_str()).map(String::from).ok_or_else(|| LxError::type_err(format!("node missing '{k}'"), span))
  };
  let children = match rec.get("children") {
    Some(LxVal::List(l)) => l.iter().map(|v| value_to_node(v, span)).collect::<Result<_, _>>()?,
    _ => vec![],
  };
  let source_offset = rec.get("source_offset").and_then(|v| v.as_int()).and_then(|n| {
    use num_traits::ToPrimitive;
    n.to_u32()
  });
  let kind_str = str_field("kind")?;
  let kind = parse_node_kind(&kind_str).map_err(|e| LxError::type_err(e, span))?;
  Ok(DiagNode { id: str_field("id")?, label: str_field("label")?, kind, source_offset, children })
}

fn value_to_edge(val: &LxVal, span: SourceSpan) -> Result<DiagEdge, LxError> {
  let LxVal::Record(rec) = val else {
    return Err(LxError::type_err("edge must be Record", span));
  };
  let str_field = |k: &str| -> Result<String, LxError> {
    rec.get(k).and_then(|v| v.as_str()).map(String::from).ok_or_else(|| LxError::type_err(format!("edge missing '{k}'"), span))
  };
  let style_str = str_field("style")?;
  let style = parse_edge_style(&style_str).map_err(|e| LxError::type_err(e, span))?;
  let edge_type_str = rec.get("edge_type").and_then(|v| v.as_str()).unwrap_or("exec");
  let edge_type = parse_edge_type(edge_type_str).map_err(|e| LxError::type_err(e, span))?;
  Ok(DiagEdge { from: str_field("from")?, to: str_field("to")?, label: str_field("label")?, style, edge_type })
}
