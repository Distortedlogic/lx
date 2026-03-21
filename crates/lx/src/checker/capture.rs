use std::collections::HashSet;

use crate::ast::{Expr, ListElem, SExpr, SStmt, Section, Stmt, StrPart};

pub fn free_vars(expr: &SExpr) -> HashSet<String> {
  let mut vars = HashSet::new();
  let mut bound = HashSet::new();
  collect_free(&expr.node, &mut vars, &mut bound);
  vars
}

fn free_vars_stmts(stmts: &[SStmt], vars: &mut HashSet<String>, bound: &mut HashSet<String>) {
  for s in stmts {
    match &s.node {
      Stmt::Binding(b) => {
        collect_free(&b.value.node, vars, bound);
        match &b.target {
          crate::ast::BindTarget::Name(n) => {
            bound.insert(n.clone());
          },
          crate::ast::BindTarget::Reassign(n) => {
            if !bound.contains(n) {
              vars.insert(n.clone());
            }
          },
          crate::ast::BindTarget::Pattern(_) => {},
        }
      },
      Stmt::Expr(e) => collect_free(&e.node, vars, bound),
      Stmt::FieldUpdate { value, .. } => collect_free(&value.node, vars, bound),
      _ => {},
    }
  }
}

fn collect_free(expr: &Expr, vars: &mut HashSet<String>, bound: &mut HashSet<String>) {
  match expr {
    Expr::Ident(name) => {
      if !bound.contains(name) {
        vars.insert(name.clone());
      }
    },
    Expr::Func { params, body, .. } => {
      let mut inner_bound = bound.clone();
      for p in params {
        inner_bound.insert(p.name.clone());
      }
      collect_free(&body.node, vars, &mut inner_bound);
    },
    Expr::Binary { left, right, .. } => {
      collect_free(&left.node, vars, bound);
      collect_free(&right.node, vars, bound);
    },
    Expr::Unary { operand, .. } => collect_free(&operand.node, vars, bound),
    Expr::Pipe { left, right } => {
      collect_free(&left.node, vars, bound);
      collect_free(&right.node, vars, bound);
    },
    Expr::Apply { func, arg } => {
      collect_free(&func.node, vars, bound);
      collect_free(&arg.node, vars, bound);
    },
    Expr::Block(stmts) => {
      let mut inner = bound.clone();
      free_vars_stmts(stmts, vars, &mut inner);
    },
    Expr::Tuple(elems) => {
      for e in elems {
        collect_free(&e.node, vars, bound);
      }
    },
    Expr::List(elems) => {
      for e in elems {
        match e {
          ListElem::Single(e) | ListElem::Spread(e) => {
            collect_free(&e.node, vars, bound);
          },
        }
      }
    },
    Expr::Record(fields) => {
      for f in fields {
        collect_free(&f.value.node, vars, bound);
      }
    },
    Expr::Map(entries) => {
      for e in entries {
        if let Some(k) = &e.key {
          collect_free(&k.node, vars, bound);
        }
        collect_free(&e.value.node, vars, bound);
      }
    },
    Expr::Match { scrutinee, arms } => {
      collect_free(&scrutinee.node, vars, bound);
      for arm in arms {
        collect_free(&arm.body.node, vars, bound);
      }
    },
    Expr::Ternary { cond, then_, else_, .. } => {
      collect_free(&cond.node, vars, bound);
      collect_free(&then_.node, vars, bound);
      if let Some(e) = else_ {
        collect_free(&e.node, vars, bound);
      }
    },
    Expr::With { name, value, body, .. } => {
      collect_free(&value.node, vars, bound);
      let mut inner = bound.clone();
      inner.insert(name.clone());
      free_vars_stmts(body, vars, &mut inner);
    },
    Expr::WithResource { resources, body } => {
      let mut inner = bound.clone();
      for (r, name) in resources {
        collect_free(&r.node, vars, bound);
        inner.insert(name.clone());
      }
      free_vars_stmts(body, vars, &mut inner);
    },
    Expr::WithContext { fields, body } => {
      for (_, expr) in fields {
        collect_free(&expr.node, vars, bound);
      }
      free_vars_stmts(body, vars, bound);
    },
    Expr::Shell { parts, .. } => {
      for part in parts {
        if let StrPart::Interp(e) = part {
          collect_free(&e.node, vars, bound);
        }
      }
    },
    Expr::Par(stmts) => free_vars_stmts(stmts, vars, bound),
    Expr::Sel(arms) => {
      for arm in arms {
        collect_free(&arm.expr.node, vars, bound);
        collect_free(&arm.handler.node, vars, bound);
      }
    },
    Expr::Propagate(inner) => collect_free(&inner.node, vars, bound),
    Expr::Coalesce { expr, default } => {
      collect_free(&expr.node, vars, bound);
      collect_free(&default.node, vars, bound);
    },
    Expr::FieldAccess { expr, .. } => collect_free(&expr.node, vars, bound),
    Expr::Yield { value } | Expr::Emit { value } => {
      collect_free(&value.node, vars, bound);
    },
    Expr::Loop(stmts) => free_vars_stmts(stmts, vars, bound),
    Expr::Break(val) => {
      if let Some(v) = val {
        collect_free(&v.node, vars, bound);
      }
    },
    Expr::Assert { expr, msg } => {
      collect_free(&expr.node, vars, bound);
      if let Some(m) = msg {
        collect_free(&m.node, vars, bound);
      }
    },
    Expr::NamedArg { value, .. } => collect_free(&value.node, vars, bound),
    Expr::Slice { expr, start, end } => {
      collect_free(&expr.node, vars, bound);
      if let Some(s) = start {
        collect_free(&s.node, vars, bound);
      }
      if let Some(e) = end {
        collect_free(&e.node, vars, bound);
      }
    },
    Expr::Section(section) => match section {
      Section::Right { operand, .. } | Section::Left { operand, .. } => {
        collect_free(&operand.node, vars, bound);
      },
      Section::BinOp(_) | Section::Field(_) | Section::Index(_) => {},
    },
    Expr::Literal(_) | Expr::TypeConstructor(_) => {},
  }
}
