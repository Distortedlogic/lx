use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Result};

use crate::field_strategy::{WalkStrategy, classify_type, node_id_expr, visitor_dispatch_path, walk_fn_path};

pub fn generate_struct_walk(input: &DeriveInput) -> Result<TokenStream> {
  let name = &input.ident;

  let Data::Struct(data) = &input.data else {
    return Err(syn::Error::new_spanned(name, "expected struct"));
  };

  let Fields::Named(fields) = &data.fields else {
    return Err(syn::Error::new_spanned(name, "AstWalk only supports named fields on structs"));
  };

  let mut field_exprs = Vec::new();
  let mut children_exprs = Vec::new();
  let mut walk_stmts = Vec::new();

  for field in &fields.named {
    let field_name = field.ident.as_ref().expect("named field");
    let strategy = classify_type(&field.ty);
    let is_vec = is_vec_type(&field.ty);

    let expr = match &strategy {
      WalkStrategy::Passthrough => {
        quote! { self.#field_name }
      },
      WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn for id");
        quote! { #walk_fn(t, self.#field_name, arena) }
      },
      WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn for option id");
        quote! { self.#field_name.map(|id| #walk_fn(t, id, arena)) }
      },
      WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn for vec id");
        quote! {
            self.#field_name.into_iter()
                .map(|id| #walk_fn(t, id, arena))
                .collect()
        }
      },
      WalkStrategy::WalkableStruct => {
        if is_vec {
          quote! {
              self.#field_name.into_iter()
                  .map(|item| item.recurse_children(t, arena))
                  .collect()
          }
        } else {
          quote! { self.#field_name.recurse_children(t, arena) }
        }
      },
      WalkStrategy::OptionWalkableStruct => {
        quote! { self.#field_name.map(|item| item.recurse_children(t, arena)) }
      },
    };

    field_exprs.push(quote! { #field_name: #expr });

    let field_ref = quote! { self.#field_name };
    if let Some(child_expr) = node_id_expr(&strategy, &field_ref, is_vec) {
      children_exprs.push(quote! { result.extend(#child_expr); });
    }

    if let Some(dispatch_fn) = visitor_dispatch_path(&strategy) {
      let walk_stmt = match &strategy {
        WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
          quote! { #dispatch_fn(v, self.#field_name, arena)?; }
        },
        WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
          quote! {
              if let Some(id) = self.#field_name {
                  #dispatch_fn(v, id, arena)?;
              }
          }
        },
        WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
          quote! {
              for &id in &self.#field_name {
                  #dispatch_fn(v, id, arena)?;
              }
          }
        },
        _ => unreachable!(),
      };
      walk_stmts.push(walk_stmt);
    } else if matches!(&strategy, WalkStrategy::WalkableStruct) {
      if is_vec {
        walk_stmts.push(quote! {
            for item in &self.#field_name {
                item.walk_children(v, arena)?;
            }
        });
      } else {
        walk_stmts.push(quote! {
            self.#field_name.walk_children(v, arena)?;
        });
      }
    } else if matches!(&strategy, WalkStrategy::OptionWalkableStruct) {
      walk_stmts.push(quote! {
          if let Some(ref item) = self.#field_name {
              item.walk_children(v, arena)?;
          }
      });
    }
  }

  Ok(quote! {
      impl #name {
          pub fn recurse_children<T: crate::visitor::transformer::AstTransformer + ?Sized>(
              self,
              t: &mut T,
              arena: &mut crate::ast::AstArena,
          ) -> Self {
              Self {
                  #(#field_exprs,)*
              }
          }

          pub fn children(&self) -> smallvec::SmallVec<[crate::ast::NodeId; 4]> {
              let mut result = smallvec::SmallVec::new();
              #(#children_exprs)*
              result
          }

          pub fn walk_children<V: crate::visitor::AstVisitor + ?Sized>(
              &self,
              v: &mut V,
              arena: &crate::ast::AstArena,
          ) -> std::ops::ControlFlow<()> {
              #(#walk_stmts)*
              std::ops::ControlFlow::Continue(())
          }
      }
  })
}

fn is_vec_type(ty: &syn::Type) -> bool {
  if let syn::Type::Path(tp) = ty
    && let Some(seg) = tp.path.segments.last()
  {
    return seg.ident == "Vec";
  }
  false
}
