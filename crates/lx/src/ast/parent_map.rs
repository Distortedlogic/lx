use std::collections::HashMap;

use super::{AstArena, BindTarget, Expr, ExprWith, FieldKind, ListElem, MapEntry, NodeId, Pattern, RecordField, Section, Stmt, TypeExpr, WithKind};

type Map = HashMap<NodeId, NodeId>;

pub fn build_parent_map(arena: &AstArena) -> Map {
  let mut m = Map::new();
  for (id, s) in arena.stmts.iter() {
    visit_stmt(&mut m, &s.node, NodeId::Stmt(id));
  }
  for (id, s) in arena.exprs.iter() {
    visit_expr(&mut m, &s.node, NodeId::Expr(id));
  }
  for (id, s) in arena.patterns.iter() {
    visit_pattern(&mut m, &s.node, NodeId::Pattern(id));
  }
  for (id, s) in arena.type_exprs.iter() {
    visit_type_expr(&mut m, &s.node, NodeId::TypeExpr(id));
  }
  m
}

fn ins(m: &mut Map, child: NodeId, parent: NodeId) {
  m.insert(child, parent);
}

fn ins_expr(m: &mut Map, eid: super::ExprId, p: NodeId) {
  ins(m, NodeId::Expr(eid), p);
}
fn ins_stmt(m: &mut Map, sid: super::StmtId, p: NodeId) {
  ins(m, NodeId::Stmt(sid), p);
}
fn ins_pat(m: &mut Map, pid: super::PatternId, p: NodeId) {
  ins(m, NodeId::Pattern(pid), p);
}
fn ins_ty(m: &mut Map, tid: super::TypeExprId, p: NodeId) {
  ins(m, NodeId::TypeExpr(tid), p);
}

fn visit_stmt(m: &mut Map, stmt: &Stmt, p: NodeId) {
  match stmt {
    Stmt::Binding(b) => {
      ins_expr(m, b.value, p);
      if let BindTarget::Pattern(pid) = &b.target {
        ins_pat(m, *pid, p);
      }
      if let Some(tid) = b.type_ann {
        ins_ty(m, tid, p);
      }
    },
    Stmt::Expr(eid) => ins_expr(m, *eid, p),
    Stmt::FieldUpdate(fu) => ins_expr(m, fu.value, p),
    Stmt::TraitDecl(data) => {
      for entry in &data.entries {
        if let super::TraitEntry::Field(f) = entry {
          if let Some(eid) = f.default {
            ins_expr(m, eid, p);
          }
          if let Some(eid) = f.constraint {
            ins_expr(m, eid, p);
          }
        }
      }
      for method in &data.methods {
        for f in &method.input {
          if let Some(eid) = f.default {
            ins_expr(m, eid, p);
          }
          if let Some(eid) = f.constraint {
            ins_expr(m, eid, p);
          }
        }
      }
      for d in &data.defaults {
        ins_expr(m, d.handler, p);
      }
    },
    Stmt::ClassDecl(data) => {
      for f in &data.fields {
        ins_expr(m, f.default, p);
      }
      for method in &data.methods {
        ins_expr(m, method.handler, p);
      }
    },
    Stmt::TypeDef(_) | Stmt::TraitUnion(_) | Stmt::Use(_) => {},
  }
}

fn visit_expr(m: &mut Map, expr: &Expr, p: NodeId) {
  match expr {
    Expr::Literal(lit) => {
      if let super::Literal::Str(parts) = lit {
        for part in parts {
          if let super::StrPart::Interp(eid) = part {
            ins_expr(m, *eid, p);
          }
        }
      }
    },
    Expr::Binary(b) => {
      ins_expr(m, b.left, p);
      ins_expr(m, b.right, p);
    },
    Expr::Unary(u) => ins_expr(m, u.operand, p),
    Expr::Pipe(pipe) => {
      ins_expr(m, pipe.left, p);
      ins_expr(m, pipe.right, p);
    },
    Expr::Apply(a) => {
      ins_expr(m, a.func, p);
      ins_expr(m, a.arg, p);
    },
    Expr::Section(s) => match s {
      Section::Right { operand, .. } | Section::Left { operand, .. } => ins_expr(m, *operand, p),
      _ => {},
    },
    Expr::FieldAccess(fa) => {
      ins_expr(m, fa.expr, p);
      if let FieldKind::Computed(eid) = &fa.field {
        ins_expr(m, *eid, p);
      }
    },
    Expr::Block(stmts) | Expr::Loop(stmts) | Expr::Par(stmts) => {
      for sid in stmts {
        ins_stmt(m, *sid, p);
      }
    },
    Expr::Tuple(elems) => {
      for eid in elems {
        ins_expr(m, *eid, p);
      }
    },
    Expr::List(elems) => {
      for elem in elems {
        match elem {
          ListElem::Single(eid) | ListElem::Spread(eid) => ins_expr(m, *eid, p),
        }
      }
    },
    Expr::Record(fields) => {
      for f in fields {
        match f {
          RecordField::Named { value, .. } | RecordField::Spread(value) => ins_expr(m, *value, p),
        }
      }
    },
    Expr::Map(entries) => {
      for e in entries {
        match e {
          MapEntry::Keyed { key, value } => {
            ins_expr(m, *key, p);
            ins_expr(m, *value, p);
          },
          MapEntry::Spread(eid) => ins_expr(m, *eid, p),
        }
      }
    },
    Expr::Func(func) => {
      for param in &func.params {
        if let Some(tid) = param.type_ann {
          ins_ty(m, tid, p);
        }
        if let Some(eid) = param.default {
          ins_expr(m, eid, p);
        }
      }
      if let Some(tid) = func.ret_type {
        ins_ty(m, tid, p);
      }
      if let Some(eid) = func.guard {
        ins_expr(m, eid, p);
      }
      ins_expr(m, func.body, p);
    },
    Expr::Match(mt) => {
      ins_expr(m, mt.scrutinee, p);
      for arm in &mt.arms {
        ins_pat(m, arm.pattern, p);
        if let Some(eid) = arm.guard {
          ins_expr(m, eid, p);
        }
        ins_expr(m, arm.body, p);
      }
    },
    Expr::Ternary(t) => {
      ins_expr(m, t.cond, p);
      ins_expr(m, t.then_, p);
      if let Some(eid) = t.else_ {
        ins_expr(m, eid, p);
      }
    },
    Expr::Propagate(eid) => ins_expr(m, *eid, p),
    Expr::Coalesce(c) => {
      ins_expr(m, c.expr, p);
      ins_expr(m, c.default, p);
    },
    Expr::Slice(s) => {
      ins_expr(m, s.expr, p);
      if let Some(eid) = s.start {
        ins_expr(m, eid, p);
      }
      if let Some(eid) = s.end {
        ins_expr(m, eid, p);
      }
    },
    Expr::NamedArg(na) => ins_expr(m, na.value, p),
    Expr::Break(val) => {
      if let Some(eid) = val {
        ins_expr(m, *eid, p);
      }
    },
    Expr::Assert(a) => {
      ins_expr(m, a.expr, p);
      if let Some(eid) = a.msg {
        ins_expr(m, eid, p);
      }
    },
    Expr::Sel(arms) => {
      for arm in arms {
        ins_expr(m, arm.expr, p);
        ins_expr(m, arm.handler, p);
      }
    },
    Expr::Timeout(t) => {
      ins_expr(m, t.ms, p);
      ins_expr(m, t.body, p);
    },
    Expr::Emit(e) => ins_expr(m, e.value, p),
    Expr::Yield(y) => ins_expr(m, y.value, p),
    Expr::With(ExprWith { kind, body }) => {
      match kind {
        WithKind::Binding { value, .. } => ins_expr(m, *value, p),
        WithKind::Resources { resources } => {
          for (eid, _) in resources {
            ins_expr(m, *eid, p);
          }
        },
        WithKind::Context { fields } => {
          for (_, eid) in fields {
            ins_expr(m, *eid, p);
          }
        },
      }
      for sid in body {
        ins_stmt(m, *sid, p);
      }
    },
    Expr::Ident(_) | Expr::TypeConstructor(_) => {},
  }
}

fn visit_pattern(m: &mut Map, pattern: &Pattern, p: NodeId) {
  match pattern {
    Pattern::Tuple(pats) => {
      for pid in pats {
        ins_pat(m, *pid, p);
      }
    },
    Pattern::List(pl) => {
      for pid in &pl.elems {
        ins_pat(m, *pid, p);
      }
    },
    Pattern::Record(pr) => {
      for fp in &pr.fields {
        if let Some(pid) = fp.pattern {
          ins_pat(m, pid, p);
        }
      }
    },
    Pattern::Constructor(pc) => {
      for pid in &pc.args {
        ins_pat(m, *pid, p);
      }
    },
    Pattern::Literal(_) | Pattern::Bind(_) | Pattern::Wildcard => {},
  }
}

fn visit_type_expr(m: &mut Map, te: &TypeExpr, p: NodeId) {
  match te {
    TypeExpr::Applied(_, args) | TypeExpr::Tuple(args) => args.iter().for_each(|tid| ins_ty(m, *tid, p)),
    TypeExpr::List(tid) => ins_ty(m, *tid, p),
    TypeExpr::Map { key, value } | TypeExpr::Func { param: key, ret: value } | TypeExpr::Fallible { ok: key, err: value } => {
      ins_ty(m, *key, p);
      ins_ty(m, *value, p);
    },
    TypeExpr::Record(fields) => fields.iter().for_each(|f| ins_ty(m, f.ty, p)),
    TypeExpr::Named(_) | TypeExpr::Var(_) => {},
  }
}
