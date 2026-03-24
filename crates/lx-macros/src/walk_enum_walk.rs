use proc_macro2::TokenStream;
use quote::quote;
use syn::{Result, Type};

use crate::field_strategy::{WalkStrategy, classify_type, is_single_id, visitor_dispatch_path};

pub fn generate_single_field_walk_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, field_ty: &Type) -> TokenStream {
  let strategy = classify_type(field_ty);
  let is_vec = is_vec_type(field_ty);
  let field_ref = if is_single_id(&strategy) {
    quote! { *inner }
  } else {
    quote! { inner }
  };

  if let Some(dispatch_fn) = visitor_dispatch_path(&strategy) {
    match &strategy {
      WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
        quote! { #enum_name::#variant_name(inner) => { #dispatch_fn(v, #field_ref, arena)?; } }
      },
      WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
        quote! {
            #enum_name::#variant_name(inner) => {
                if let Some(id) = #field_ref {
                    #dispatch_fn(v, id, arena)?;
                }
            }
        }
      },
      WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
        quote! {
            #enum_name::#variant_name(inner) => {
                for &id in #field_ref.iter() {
                    #dispatch_fn(v, id, arena)?;
                }
            }
        }
      },
      _ => unreachable!(),
    }
  } else if matches!(&strategy, WalkStrategy::WalkableStruct) {
    if is_vec {
      quote! {
          #enum_name::#variant_name(inner) => {
              for item in inner.iter() {
                  item.walk_children(v, arena)?;
              }
          }
      }
    } else {
      quote! {
          #enum_name::#variant_name(inner) => {
              inner.walk_children(v, arena)?;
          }
      }
    }
  } else {
    quote! { #enum_name::#variant_name(_) => {} }
  }
}

pub fn generate_named_fields_walk_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, fields: &syn::FieldsNamed) -> Result<TokenStream> {
  let mut bindings = Vec::new();
  let mut walk_stmts = Vec::new();
  let mut has_walkable = false;

  for field in &fields.named {
    let fname = field.ident.as_ref().expect("named field");
    let strategy = classify_type(&field.ty);
    let is_vec = is_vec_type(&field.ty);

    if let Some(dispatch_fn) = visitor_dispatch_path(&strategy) {
      bindings.push(quote! { #fname });
      has_walkable = true;
      let stmt = match &strategy {
        WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
          quote! { #dispatch_fn(v, *#fname, arena)?; }
        },
        WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
          quote! {
              if let Some(id) = #fname {
                  #dispatch_fn(v, *id, arena)?;
              }
          }
        },
        WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
          quote! {
              for &id in #fname.iter() {
                  #dispatch_fn(v, id, arena)?;
              }
          }
        },
        _ => unreachable!(),
      };
      walk_stmts.push(stmt);
    } else if matches!(&strategy, WalkStrategy::WalkableStruct) {
      bindings.push(quote! { #fname });
      has_walkable = true;
      if is_vec {
        walk_stmts.push(quote! {
            for item in #fname.iter() {
                item.walk_children(v, arena)?;
            }
        });
      } else {
        walk_stmts.push(quote! {
            #fname.walk_children(v, arena)?;
        });
      }
    } else if matches!(&strategy, WalkStrategy::OptionWalkableStruct) {
      bindings.push(quote! { #fname });
      has_walkable = true;
      walk_stmts.push(quote! {
          if let Some(ref item) = #fname {
              item.walk_children(v, arena)?;
          }
      });
    }
  }

  if !has_walkable {
    Ok(quote! { #enum_name::#variant_name { .. } => {} })
  } else {
    Ok(quote! {
        #enum_name::#variant_name { #(#bindings,)* .. } => {
            #(#walk_stmts)*
        }
    })
  }
}

pub fn generate_multi_unnamed_walk_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, fields: &syn::FieldsUnnamed) -> Result<TokenStream> {
  let mut bindings = Vec::new();
  let mut walk_stmts = Vec::new();

  for (i, field) in fields.unnamed.iter().enumerate() {
    let binding = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
    let strategy = classify_type(&field.ty);
    let is_vec = is_vec_type(&field.ty);

    if let Some(dispatch_fn) = visitor_dispatch_path(&strategy) {
      let stmt = match &strategy {
        WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
          quote! { #dispatch_fn(v, *#binding, arena)?; }
        },
        WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
          quote! {
              if let Some(id) = #binding {
                  #dispatch_fn(v, *id, arena)?;
              }
          }
        },
        WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
          quote! {
              for &id in #binding.iter() {
                  #dispatch_fn(v, id, arena)?;
              }
          }
        },
        _ => unreachable!(),
      };
      walk_stmts.push(stmt);
    } else if matches!(&strategy, WalkStrategy::WalkableStruct) {
      if is_vec {
        walk_stmts.push(quote! {
            for item in #binding.iter() {
                item.walk_children(v, arena)?;
            }
        });
      } else {
        walk_stmts.push(quote! {
            #binding.walk_children(v, arena)?;
        });
      }
    } else if matches!(&strategy, WalkStrategy::OptionWalkableStruct) {
      walk_stmts.push(quote! {
          if let Some(ref item) = #binding {
              item.walk_children(v, arena)?;
          }
      });
    }
    bindings.push(binding);
  }

  if walk_stmts.is_empty() {
    Ok(quote! { #enum_name::#variant_name(..) => {} })
  } else {
    Ok(quote! {
        #enum_name::#variant_name(#(#bindings),*) => {
            #(#walk_stmts)*
        }
    })
  }
}

fn is_vec_type(ty: &Type) -> bool {
  if let syn::Type::Path(tp) = ty
    && let Some(seg) = tp.path.segments.last()
    && seg.ident == "Vec"
    && let syn::PathArguments::AngleBracketed(args) = &seg.arguments
  {
    return !args.args.is_empty();
  }
  false
}
