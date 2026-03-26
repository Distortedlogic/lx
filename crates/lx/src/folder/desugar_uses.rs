use miette::SourceSpan;

use crate::ast::{AgentMethod, AstArena, ClassField, Expr, ExprBlock, ExprFieldAccess, FieldKind, ListElem, Literal, Stmt, StmtId};
use crate::folder::gen_ast::{gen_apply, gen_block, gen_func, gen_method, gen_self_field};
use crate::sym::{Sym, intern};

fn conn_field_name(type_name: Sym) -> Sym {
  intern(&format!("__conn_{}", type_name.as_str().to_lowercase()))
}

pub(super) fn generate_uses_wiring(uses: &[Sym], fields: &mut Vec<ClassField>, methods: &mut Vec<AgentMethod>, span: SourceSpan, arena: &mut AstArena) {
  let mut init_stmts = Vec::new();
  let mut tool_elems = Vec::new();

  for &type_sym in uses {
    let field_name = conn_field_name(type_sym);

    let ctor = arena.alloc_expr(Expr::TypeConstructor(type_sym), span);
    let empty_record = arena.alloc_expr(Expr::Record(vec![]), span);
    let instance = gen_apply(ctor, empty_record, span, arena);
    fields.push(ClassField { name: field_name, default: instance });

    let self_conn = gen_self_field(field_name.as_str(), span, arena);
    let connect_access = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: self_conn, field: FieldKind::Named(intern("connect")) }), span);
    let unit_arg = arena.alloc_expr(Expr::Literal(Literal::Unit), span);
    let connect_call = gen_apply(connect_access, unit_arg, span, arena);
    let connect_stmt = arena.alloc_stmt(Stmt::Expr(connect_call), span);
    init_stmts.push(connect_stmt);

    let self_conn2 = gen_self_field(field_name.as_str(), span, arena);
    let tools_access = arena.alloc_expr(Expr::FieldAccess(ExprFieldAccess { expr: self_conn2, field: FieldKind::Named(intern("tools")) }), span);
    let unit_arg2 = arena.alloc_expr(Expr::Literal(Literal::Unit), span);
    let tools_call = gen_apply(tools_access, unit_arg2, span, arena);
    tool_elems.push(ListElem::Spread(tools_call));
  }

  wire_init_method(methods, init_stmts, span, arena);
  wire_tools_method(methods, tool_elems, span, arena);
}

fn wire_init_method(methods: &mut Vec<AgentMethod>, init_stmts: Vec<StmtId>, span: SourceSpan, arena: &mut AstArena) {
  let init_sym = intern("init");
  if let Some(existing) = methods.iter_mut().find(|m| m.name == init_sym) {
    let old_handler = existing.handler;
    let old_expr = arena.expr(old_handler).clone();
    if let Expr::Func(mut func) = old_expr {
      let old_body_expr = arena.expr(func.body).clone();
      let new_body = if let Expr::Block(ExprBlock { stmts }) = old_body_expr {
        let mut combined = init_stmts;
        combined.extend(stmts);
        gen_block(combined, span, arena)
      } else {
        let mut stmts = init_stmts;
        let old_body_stmt = arena.alloc_stmt(Stmt::Expr(func.body), span);
        stmts.push(old_body_stmt);
        gen_block(stmts, span, arena)
      };
      func.body = new_body;
      existing.handler = arena.alloc_expr(Expr::Func(func), span);
    }
  } else {
    let body = gen_block(init_stmts, span, arena);
    let init_func = gen_func(&[], body, span, arena);
    methods.push(gen_method("init", init_func));
  }
}

fn wire_tools_method(methods: &mut Vec<AgentMethod>, tool_elems: Vec<ListElem>, span: SourceSpan, arena: &mut AstArena) {
  let tools_sym = intern("tools");
  if let Some(existing) = methods.iter_mut().find(|m| m.name == tools_sym) {
    let old_handler = existing.handler;
    let old_expr = arena.expr(old_handler).clone();
    if let Expr::Func(mut func) = old_expr {
      let old_tools_call = func.body;
      let mut elems = tool_elems;
      elems.push(ListElem::Spread(old_tools_call));
      let combined_list = arena.alloc_expr(Expr::List(elems), span);
      func.body = combined_list;
      existing.handler = arena.alloc_expr(Expr::Func(func), span);
    }
  } else {
    let tools_list = arena.alloc_expr(Expr::List(tool_elems), span);
    let tools_func = gen_func(&[], tools_list, span, arena);
    methods.push(gen_method("tools", tools_func));
  }
}
