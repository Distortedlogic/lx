use miette::SourceSpan;

use lx_ast::ast::{
  AgentMethod, AstArena, Expr, ExprApply, ExprBlock, ExprFieldAccess, ExprFunc, ExprId, ExprPropagate, FieldKind, ListElem, Literal, Param, RecordField, Stmt,
  StmtFieldUpdate, StmtId, StrPart,
};
use lx_span::sym::{Sym, intern};

pub fn gen_ident(name: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Ident(intern(name)), span)
}

pub fn gen_self_field(field: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let self_id = arena.alloc_expr(Expr::Ident(intern("self")), span);
  arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: self_id, field: FieldKind::Named(intern(field)) }), span)
}

pub fn gen_apply(func: ExprId, arg: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Apply(ExprApply { func, arg }), span)
}

pub fn gen_apply_chain(func: ExprId, args: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let mut result = func;
  for &arg in args {
    result = arena.alloc_expr(Expr::Apply(ExprApply { func: result, arg }), span);
  }
  result
}

pub fn gen_field_call(obj: &str, method: &str, args: &[ExprId], span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let obj_id = arena.alloc_expr(Expr::Ident(intern(obj)), span);
  let access = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: obj_id, field: FieldKind::Named(intern(method)) }), span);
  gen_apply_chain(access, args, span, arena)
}

pub fn gen_block(stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Block(ExprBlock { stmts }), span)
}

pub fn gen_func(params: &[&str], body: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let params = params.iter().map(|name| Param { name: intern(name), type_ann: None, default: None }).collect();
  arena.alloc_expr(Expr::Func(ExprFunc { params, type_params: vec![], ret_type: None, guard: None, body }), span)
}

pub fn gen_propagate(inner: ExprId, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Propagate(ExprPropagate { inner }), span)
}

pub fn gen_field_update(obj: &str, field: &str, value: ExprId, span: SourceSpan, arena: &mut AstArena) -> StmtId {
  arena.alloc_stmt(Stmt::FieldUpdate(StmtFieldUpdate { name: intern(obj), fields: vec![intern(field)], value }), span)
}

pub fn gen_literal_str(s: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Literal(Literal::Str(vec![StrPart::Text(s.to_string())])), span)
}

pub fn gen_none(span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Ident(intern("None")), span)
}

pub fn gen_record(fields: Vec<(Sym, ExprId)>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let record_fields = fields.into_iter().map(|(name, value)| RecordField::Named { name, value }).collect();
  arena.alloc_expr(Expr::Record(record_fields), span)
}

pub fn gen_list(elems: Vec<ExprId>, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  let list_elems = elems.into_iter().map(ListElem::Single).collect();
  arena.alloc_expr(Expr::List(list_elems), span)
}

pub fn gen_method(name: &str, func: ExprId) -> AgentMethod {
  AgentMethod { name: intern(name), handler: func }
}
