use miette::SourceSpan;

use crate::ast::{AgentMethod, AstArena, ClassField, Expr, ListElem, Literal, StrPart};
use crate::folder::gen_ast::{gen_apply, gen_func, gen_method};
use crate::sym::{Sym, intern};

fn tool_field_name(type_name: Sym) -> Sym {
  intern(&format!("__tool_{}", type_name.as_str().to_lowercase()))
}

pub(super) fn generate_uses_wiring(uses: &[Sym], fields: &mut Vec<ClassField>, methods: &mut Vec<AgentMethod>, span: SourceSpan, arena: &mut AstArena) {
  let mut tool_elems = Vec::new();

  for &type_sym in uses {
    let field_name = tool_field_name(type_sym);

    let ctor = arena.alloc_expr(Expr::TypeConstructor(type_sym), span);
    let empty_record = arena.alloc_expr(Expr::Record(vec![]), span);
    let instance = gen_apply(ctor, empty_record, span, arena);
    fields.push(ClassField { name: field_name, default: instance });

    let name_str = arena.alloc_expr(Expr::Literal(Literal::Str(vec![StrPart::Text(type_sym.as_str().to_string())])), span);
    tool_elems.push(ListElem::Single(name_str));
  }

  wire_tools_method(methods, tool_elems, span, arena);
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
