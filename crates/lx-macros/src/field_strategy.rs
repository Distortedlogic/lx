use syn::{GenericArgument, PathArguments, PathSegment, Type};

pub enum WalkStrategy {
  ExprId,
  StmtId,
  PatternId,
  TypeExprId,
  VecExprId,
  VecStmtId,
  VecPatternId,
  VecTypeExprId,
  OptionExprId,
  OptionStmtId,
  OptionPatternId,
  OptionTypeExprId,
  WalkableStruct,
  OptionWalkableStruct,
  Passthrough,
}

fn last_segment(ty: &Type) -> Option<&PathSegment> {
  match ty {
    Type::Path(tp) => tp.path.segments.last(),
    _ => None,
  }
}

fn id_kind(name: &str) -> Option<WalkStrategy> {
  match name {
    "ExprId" => Some(WalkStrategy::ExprId),
    "StmtId" => Some(WalkStrategy::StmtId),
    "PatternId" => Some(WalkStrategy::PatternId),
    "TypeExprId" => Some(WalkStrategy::TypeExprId),
    _ => None,
  }
}

fn inner_type_from_generic(seg: &PathSegment) -> Option<&Type> {
  match &seg.arguments {
    PathArguments::AngleBracketed(args) => args.args.iter().find_map(|arg| match arg {
      GenericArgument::Type(ty) => Some(ty),
      _ => None,
    }),
    _ => None,
  }
}

const PASSTHROUGH_TYPES: &[&str] =
  &["Sym", "BinOp", "UnaryOp", "bool", "i64", "f64", "usize", "String", "BigInt", "UseKind", "UseStmt", "StmtTypeDef", "TraitUnionDef"];

const WALKABLE_TYPES: &[&str] = &[
  "ExprBinary",
  "ExprUnary",
  "ExprPipe",
  "ExprApply",
  "ExprFieldAccess",
  "ExprFunc",
  "ExprMatch",
  "ExprTernary",
  "ExprCoalesce",
  "ExprSlice",
  "ExprNamedArg",
  "ExprAssert",
  "ExprTimeout",
  "ExprEmit",
  "ExprYield",
  "ExprWith",
  "Literal",
  "StrPart",
  "Section",
  "FieldKind",
  "ListElem",
  "RecordField",
  "MapEntry",
  "Param",
  "MatchArm",
  "SelArm",
  "Binding",
  "BindTarget",
  "WithKind",
  "StmtFieldUpdate",
  "TraitDeclData",
  "ClassDeclData",
  "PatternList",
  "PatternRecord",
  "PatternConstructor",
  "FieldPattern",
  "TypeField",
];

pub fn classify_type(ty: &Type) -> WalkStrategy {
  let Some(seg) = last_segment(ty) else {
    return WalkStrategy::Passthrough;
  };
  let name = seg.ident.to_string();

  if let Some(strategy) = id_kind(&name) {
    return strategy;
  }

  if name == "Vec" {
    if let Some(inner) = inner_type_from_generic(seg)
      && let Some(inner_seg) = last_segment(inner)
    {
      let inner_name = inner_seg.ident.to_string();
      return match inner_name.as_str() {
        "ExprId" => WalkStrategy::VecExprId,
        "StmtId" => WalkStrategy::VecStmtId,
        "PatternId" => WalkStrategy::VecPatternId,
        "TypeExprId" => WalkStrategy::VecTypeExprId,
        _ if WALKABLE_TYPES.contains(&inner_name.as_str()) => WalkStrategy::WalkableStruct,
        _ => WalkStrategy::Passthrough,
      };
    }
    return WalkStrategy::Passthrough;
  }

  if name == "Option" {
    if let Some(inner) = inner_type_from_generic(seg)
      && let Some(inner_seg) = last_segment(inner)
    {
      let inner_name = inner_seg.ident.to_string();
      return match inner_name.as_str() {
        "ExprId" => WalkStrategy::OptionExprId,
        "StmtId" => WalkStrategy::OptionStmtId,
        "PatternId" => WalkStrategy::OptionPatternId,
        "TypeExprId" => WalkStrategy::OptionTypeExprId,
        _ if WALKABLE_TYPES.contains(&inner_name.as_str()) => WalkStrategy::OptionWalkableStruct,
        _ => WalkStrategy::Passthrough,
      };
    }
    return WalkStrategy::Passthrough;
  }

  if PASSTHROUGH_TYPES.contains(&name.as_str()) {
    return WalkStrategy::Passthrough;
  }

  if WALKABLE_TYPES.contains(&name.as_str()) {
    return WalkStrategy::WalkableStruct;
  }

  WalkStrategy::Passthrough
}

pub fn walk_fn_path(strategy: &WalkStrategy) -> Option<proc_macro2::TokenStream> {
  use quote::quote;
  match strategy {
    WalkStrategy::ExprId | WalkStrategy::VecExprId | WalkStrategy::OptionExprId => Some(quote! { crate::visitor::walk_transform::walk_transform_expr }),
    WalkStrategy::StmtId | WalkStrategy::VecStmtId | WalkStrategy::OptionStmtId => Some(quote! { crate::visitor::walk_transform::walk_transform_stmt }),
    WalkStrategy::PatternId | WalkStrategy::VecPatternId | WalkStrategy::OptionPatternId => {
      Some(quote! { crate::visitor::walk_transform::walk_transform_pattern })
    },
    WalkStrategy::TypeExprId | WalkStrategy::VecTypeExprId | WalkStrategy::OptionTypeExprId => {
      Some(quote! { crate::visitor::walk_transform::walk_transform_type_expr })
    },
    _ => None,
  }
}

pub fn node_id_expr(strategy: &WalkStrategy, field_expr: &proc_macro2::TokenStream, is_vec_walkable: bool) -> Option<proc_macro2::TokenStream> {
  use quote::quote;
  match strategy {
    WalkStrategy::ExprId => Some(quote! { vec![crate::ast::NodeId::Expr(#field_expr)] }),
    WalkStrategy::StmtId => Some(quote! { vec![crate::ast::NodeId::Stmt(#field_expr)] }),
    WalkStrategy::PatternId => Some(quote! { vec![crate::ast::NodeId::Pattern(#field_expr)] }),
    WalkStrategy::TypeExprId => Some(quote! { vec![crate::ast::NodeId::TypeExpr(#field_expr)] }),
    WalkStrategy::VecExprId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::Expr(*id)).collect::<Vec<_>>() }),
    WalkStrategy::VecStmtId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::Stmt(*id)).collect::<Vec<_>>() }),
    WalkStrategy::VecPatternId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::Pattern(*id)).collect::<Vec<_>>() }),
    WalkStrategy::VecTypeExprId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::TypeExpr(*id)).collect::<Vec<_>>() }),
    WalkStrategy::OptionExprId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::Expr(*id)).collect::<Vec<_>>() }),
    WalkStrategy::OptionStmtId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::Stmt(*id)).collect::<Vec<_>>() }),
    WalkStrategy::OptionPatternId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::Pattern(*id)).collect::<Vec<_>>() }),
    WalkStrategy::OptionTypeExprId => Some(quote! { #field_expr.iter().map(|id| crate::ast::NodeId::TypeExpr(*id)).collect::<Vec<_>>() }),
    WalkStrategy::WalkableStruct => {
      if is_vec_walkable {
        Some(quote! { #field_expr.iter().flat_map(|item| item.children()).collect::<Vec<_>>() })
      } else {
        Some(quote! { #field_expr.children() })
      }
    },
    WalkStrategy::OptionWalkableStruct => Some(quote! { #field_expr.as_ref().map(|item| item.children()).unwrap_or_default() }),
    WalkStrategy::Passthrough => None,
  }
}

pub fn is_single_id(strategy: &WalkStrategy) -> bool {
  matches!(strategy, WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId)
}
