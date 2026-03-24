use std::collections::HashMap;

use super::{AstArena, NodeId};

type Map = HashMap<NodeId, NodeId>;

pub fn build_parent_map(arena: &AstArena) -> Map {
  let mut m = Map::new();
  for (id, s) in arena.iter_stmts() {
    let parent = NodeId::Stmt(id);
    for child in s.node.children() {
      m.insert(child, parent);
    }
  }
  for (id, s) in arena.iter_exprs() {
    let parent = NodeId::Expr(id);
    for child in s.node.children() {
      m.insert(child, parent);
    }
  }
  for (id, s) in arena.iter_patterns() {
    let parent = NodeId::Pattern(id);
    for child in s.node.children() {
      m.insert(child, parent);
    }
  }
  for (id, s) in arena.iter_type_exprs() {
    let parent = NodeId::TypeExpr(id);
    for child in s.node.children() {
      m.insert(child, parent);
    }
  }
  m
}
