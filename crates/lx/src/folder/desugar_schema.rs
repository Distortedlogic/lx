use miette::SourceSpan;

use crate::ast::{
  AgentMethod, AstArena, BinOp, BindTarget, Binding, Expr, ExprApply, ExprBinary, ExprBlock, ExprFunc, ExprId, ExprUnary, KeywordDeclData, ListElem, Literal,
  Param, RecordField, Stmt, StmtId, StrPart, TraitDeclData, TraitEntry, UseKind, UseStmt,
};
use crate::sym::intern;

fn gen_str(s: &str, span: SourceSpan, arena: &mut AstArena) -> ExprId {
  arena.alloc_expr(Expr::Literal(Literal::Str(vec![StrPart::Text(s.to_string())])), span)
}

fn stringify_expr(id: ExprId, arena: &AstArena) -> String {
  match arena.expr(id) {
    Expr::Ident(sym) => sym.as_str().to_string(),
    Expr::Literal(lit) => match lit {
      Literal::Int(n) => n.to_string(),
      Literal::Float(f) => f.to_string(),
      Literal::Str(parts) => {
        let text: String = parts
          .iter()
          .map(|p| match p {
            StrPart::Text(t) => t.clone(),
            StrPart::Interp(_) => "...".to_string(),
          })
          .collect();
        format!("\"{text}\"")
      },
      Literal::Bool(b) => b.to_string(),
      Literal::RawStr(s) => format!("\"{s}\""),
      Literal::Unit => "()".to_string(),
    },
    Expr::Binary(ExprBinary { op, left, right }) => {
      format!("{} {op} {}", stringify_expr(*left, arena), stringify_expr(*right, arena))
    },
    Expr::Unary(ExprUnary { op, operand }) => {
      format!("{op}{}", stringify_expr(*operand, arena))
    },
    Expr::Apply(ExprApply { func, arg }) => {
      format!("{} {}", stringify_expr(*func, arena), stringify_expr(*arg, arena))
    },
    _ => "<constraint>".to_string(),
  }
}

fn lx_type_to_json_type(type_name: &str) -> &'static str {
  match type_name {
    "Int" => "integer",
    "Float" => "number",
    "Str" => "string",
    "Bool" => "boolean",
    "List" => "array",
    _ => "object",
  }
}

pub(super) fn desugar_schema(data: KeywordDeclData, span: SourceSpan, arena: &mut AstArena) -> Vec<StmtId> {
  let use_path = vec![intern("std"), intern("schema_trait")];
  let schema_sym = intern("Schema");
  let use_stmt = arena.alloc_stmt(Stmt::Use(UseStmt { path: use_path, kind: UseKind::Selective(vec![schema_sym]) }), span);

  let mut entries = data.trait_entries.unwrap_or_default();

  let schema_method = build_schema_method(&entries, span, arena);
  let validate_method = build_validate_method(&entries, span, arena);

  for entry in &mut entries {
    if let TraitEntry::Field(f) = entry {
      f.default = None;
    }
  }

  let trait_decl = TraitDeclData {
    name: data.name,
    type_params: data.type_params,
    entries,
    methods: vec![],
    defaults: vec![schema_method, validate_method],
    requires: vec![],
    description: None,
    tags: vec![],
    exported: data.exported,
  };

  vec![use_stmt, arena.alloc_stmt(Stmt::TraitDecl(trait_decl), span)]
}

fn build_schema_method(entries: &[TraitEntry], span: SourceSpan, arena: &mut AstArena) -> AgentMethod {
  let property_fields: Vec<RecordField> = entries
    .iter()
    .filter_map(|e| {
      let TraitEntry::Field(f) = e else { return None };
      let json_type = lx_type_to_json_type(f.type_name.as_str());
      let type_val = gen_str(json_type, span, arena);
      let mut prop_fields = vec![RecordField::Named { name: intern("type"), value: type_val }];
      if let Some(desc_id) = f.default
        && let Expr::Literal(Literal::Str(parts)) = arena.expr(desc_id)
        && let [StrPart::Text(desc_text)] = parts.as_slice()
      {
        let desc_text = desc_text.clone();
        prop_fields.push(RecordField::Named { name: intern("description"), value: gen_str(&desc_text, span, arena) });
      }
      if let Some(constraint_id) = f.constraint {
        let constraint_str = stringify_expr(constraint_id, arena);
        prop_fields.push(RecordField::Named { name: intern("constraint"), value: gen_str(&constraint_str, span, arena) });
      }
      let prop_record = arena.alloc_expr(Expr::Record(prop_fields), span);
      Some(RecordField::Named { name: f.name, value: prop_record })
    })
    .collect();
  let properties = arena.alloc_expr(Expr::Record(property_fields), span);

  let required_elems: Vec<_> = entries
    .iter()
    .filter_map(|e| {
      let TraitEntry::Field(f) = e else { return None };
      Some(ListElem::Single(gen_str(f.name.as_str(), span, arena)))
    })
    .collect();
  let required = arena.alloc_expr(Expr::List(required_elems), span);

  let type_val = gen_str("object", span, arena);
  let envelope = arena.alloc_expr(
    Expr::Record(vec![
      RecordField::Named { name: intern("type"), value: type_val },
      RecordField::Named { name: intern("properties"), value: properties },
      RecordField::Named { name: intern("required"), value: required },
    ]),
    span,
  );

  let schema_fn = arena.alloc_expr(Expr::Func(ExprFunc { params: vec![], type_params: vec![], ret_type: None, guard: None, body: envelope }), span);
  AgentMethod { name: intern("schema"), handler: schema_fn }
}

fn build_validate_method(entries: &[TraitEntry], span: SourceSpan, arena: &mut AstArena) -> AgentMethod {
  let data_sym = intern("data");
  let missing_sym = intern("missing");

  let field_names: Vec<_> = entries
    .iter()
    .filter_map(|e| {
      let TraitEntry::Field(f) = e else { return None };
      Some(ListElem::Single(gen_str(f.name.as_str(), span, arena)))
    })
    .collect();
  let names_list = arena.alloc_expr(Expr::List(field_names), span);

  let k_sym = intern("k");
  let keys_fn = arena.alloc_expr(Expr::Ident(intern("keys")), span);
  let data_ref2 = arena.alloc_expr(Expr::Ident(data_sym), span);
  let data_keys = arena.alloc_expr(Expr::Apply(ExprApply { func: keys_fn, arg: data_ref2 }), span);
  let eq_x_sym = intern("__eq_x");
  let eq_x_ref = arena.alloc_expr(Expr::Ident(eq_x_sym), span);
  let k_ref_eq = arena.alloc_expr(Expr::Ident(k_sym), span);
  let eq_body = arena.alloc_expr(Expr::Binary(ExprBinary { op: BinOp::Eq, left: eq_x_ref, right: k_ref_eq }), span);
  let eq_lambda = arena.alloc_expr(
    Expr::Func(ExprFunc {
      params: vec![Param { name: eq_x_sym, type_ann: None, default: None }],
      type_params: vec![],
      ret_type: None,
      guard: None,
      body: eq_body,
    }),
    span,
  );
  let any_fn = arena.alloc_expr(Expr::Ident(intern("any?")), span);
  let any_with_pred = arena.alloc_expr(Expr::Apply(ExprApply { func: any_fn, arg: eq_lambda }), span);
  let keys_any = arena.alloc_expr(Expr::Apply(ExprApply { func: any_with_pred, arg: data_keys }), span);
  let not_fn = arena.alloc_expr(Expr::Ident(intern("not")), span);
  let not_any = arena.alloc_expr(Expr::Apply(ExprApply { func: not_fn, arg: keys_any }), span);
  let filter_body = arena.alloc_expr(
    Expr::Func(ExprFunc {
      params: vec![Param { name: k_sym, type_ann: None, default: None }],
      type_params: vec![],
      ret_type: None,
      guard: None,
      body: not_any,
    }),
    span,
  );
  let filter_fn = arena.alloc_expr(Expr::Ident(intern("filter")), span);
  let filter_with_pred = arena.alloc_expr(Expr::Apply(ExprApply { func: filter_fn, arg: filter_body }), span);
  let missing_val = arena.alloc_expr(Expr::Apply(ExprApply { func: filter_with_pred, arg: names_list }), span);

  let missing_binding = arena
    .alloc_stmt(Stmt::Binding(Binding { exported: false, mutable: false, target: BindTarget::Name(missing_sym), type_ann: None, value: missing_val }), span);

  let missing_ref = arena.alloc_expr(Expr::Ident(missing_sym), span);
  let len_fn = arena.alloc_expr(Expr::Ident(intern("len")), span);
  let missing_len = arena.alloc_expr(Expr::Apply(ExprApply { func: len_fn, arg: missing_ref }), span);
  let zero = arena.alloc_expr(Expr::Literal(Literal::Int(0.into())), span);
  let cond = arena.alloc_expr(Expr::Binary(ExprBinary { op: BinOp::Eq, left: missing_len, right: zero }), span);

  let data_ref3 = arena.alloc_expr(Expr::Ident(data_sym), span);
  let ok_ctor = arena.alloc_expr(Expr::TypeConstructor(intern("Ok")), span);
  let ok_result = arena.alloc_expr(Expr::Apply(ExprApply { func: ok_ctor, arg: data_ref3 }), span);

  let missing_ref2 = arena.alloc_expr(Expr::Ident(missing_sym), span);
  let err_record = arena.alloc_expr(Expr::Record(vec![RecordField::Named { name: intern("missing"), value: missing_ref2 }]), span);
  let err_ctor = arena.alloc_expr(Expr::TypeConstructor(intern("Err")), span);
  let err_result = arena.alloc_expr(Expr::Apply(ExprApply { func: err_ctor, arg: err_record }), span);

  let ternary = super::desugar::desugar_ternary(cond, ok_result, Some(err_result), span, arena);
  let ternary_expr = arena.alloc_expr(ternary, span);

  let ternary_stmt = arena.alloc_stmt(Stmt::Expr(ternary_expr), span);
  let body = arena.alloc_expr(Expr::Block(ExprBlock { stmts: vec![missing_binding, ternary_stmt] }), span);

  let validate_fn = arena.alloc_expr(
    Expr::Func(ExprFunc { params: vec![Param { name: data_sym, type_ann: None, default: None }], type_params: vec![], ret_type: None, guard: None, body }),
    span,
  );
  AgentMethod { name: intern("validate"), handler: validate_fn }
}
