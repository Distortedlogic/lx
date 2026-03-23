use std::collections::HashMap;

use crate::ast::{AstArena, BinOp, Expr, ExprId, Literal, Pattern};
use crate::sym::Sym;

use super::type_arena::{TypeArena, TypeId};
use super::types::Type;

pub struct NarrowingEnv {
  stack: Vec<HashMap<Sym, TypeId>>,
}

impl NarrowingEnv {
  pub fn new() -> Self {
    Self { stack: vec![] }
  }

  pub fn push(&mut self) {
    self.stack.push(HashMap::new());
  }

  pub fn pop(&mut self) {
    self.stack.pop();
  }

  pub fn narrow(&mut self, name: Sym, ty: TypeId) {
    if let Some(top) = self.stack.last_mut() {
      top.insert(name, ty);
    }
  }

  pub fn lookup(&self, name: Sym) -> Option<TypeId> {
    for scope in self.stack.iter().rev() {
      if let Some(&ty) = scope.get(&name) {
        return Some(ty);
      }
    }
    None
  }
}

pub fn compute_narrowed_type(pattern: &Pattern, scrutinee_type: TypeId, type_arena: &mut TypeArena, resolved_type: &Type) -> TypeId {
  match (pattern, resolved_type) {
    (Pattern::Constructor(ctor), Type::Union { name, variants }) => {
      if let Some(matched) = variants.iter().find(|v| v.name == ctor.name) {
        type_arena.alloc(Type::Union { name: *name, variants: vec![matched.clone()] })
      } else {
        scrutinee_type
      }
    },
    _ => scrutinee_type,
  }
}

pub struct BranchNarrowings {
  pub then_narrowings: Vec<(Sym, TypeId)>,
  pub else_narrowings: Vec<(Sym, TypeId)>,
}

pub fn analyze_condition(cond_id: ExprId, arena: &AstArena, type_arena: &TypeArena) -> BranchNarrowings {
  let expr = arena.expr(cond_id);
  let mut result = BranchNarrowings { then_narrowings: vec![], else_narrowings: vec![] };

  if let Expr::Binary(b) = expr {
    let left = arena.expr(b.left);
    let right = arena.expr(b.right);
    match b.op {
      BinOp::Eq => {
        if let Expr::Ident(name) = left
          && let Expr::Literal(lit) = right
        {
          result.then_narrowings.push((*name, literal_type(lit, type_arena)));
        }
        if let Expr::Ident(name) = right
          && let Expr::Literal(lit) = left
        {
          result.then_narrowings.push((*name, literal_type(lit, type_arena)));
        }
      },
      BinOp::NotEq => {
        if let Expr::Ident(name) = left
          && let Expr::Literal(lit) = right
        {
          result.else_narrowings.push((*name, literal_type(lit, type_arena)));
        }
        if let Expr::Ident(name) = right
          && let Expr::Literal(lit) = left
        {
          result.else_narrowings.push((*name, literal_type(lit, type_arena)));
        }
      },
      _ => {},
    }
  }
  result
}

fn literal_type(lit: &Literal, ta: &TypeArena) -> TypeId {
  match lit {
    Literal::Int(_) => ta.int(),
    Literal::Float(_) => ta.float(),
    Literal::Str(_) | Literal::RawStr(_) => ta.str(),
    Literal::Bool(_) => ta.bool(),
    Literal::Unit => ta.unit(),
  }
}
