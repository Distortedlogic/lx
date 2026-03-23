use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, PathArguments, Result, Type};

use crate::field_strategy::{WalkStrategy, classify_type, walk_fn_path};
use crate::walk_enum_children::{generate_multi_unnamed_children_arm, generate_named_fields_children_arm, generate_single_field_children_arm};

pub fn generate_enum_walk(input: &DeriveInput) -> Result<TokenStream> {
  let name = &input.ident;

  let Data::Enum(data) = &input.data else {
    return Err(syn::Error::new_spanned(name, "expected enum"));
  };

  let mut recurse_arms = Vec::new();
  let mut children_arms = Vec::new();

  for variant in &data.variants {
    let vname = &variant.ident;
    let has_skip = variant.attrs.iter().any(|a| a.path().is_ident("walk"));

    if has_skip {
      let (recurse_arm, children_arm) = generate_skip_arms(name, vname, &variant.fields);
      recurse_arms.push(recurse_arm);
      children_arms.push(children_arm);
      continue;
    }

    match &variant.fields {
      Fields::Unit => {
        recurse_arms.push(quote! { #name::#vname => #name::#vname });
        children_arms.push(quote! { #name::#vname => Vec::new() });
      },
      Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
        let field_ty = &fields.unnamed[0].ty;
        recurse_arms.push(generate_single_field_recurse_arm(name, vname, field_ty));
        children_arms.push(generate_single_field_children_arm(name, vname, field_ty));
      },
      Fields::Named(fields) => {
        recurse_arms.push(generate_named_fields_recurse_arm(name, vname, fields)?);
        children_arms.push(generate_named_fields_children_arm(name, vname, fields)?);
      },
      Fields::Unnamed(fields) => {
        recurse_arms.push(generate_multi_unnamed_recurse_arm(name, vname, fields)?);
        children_arms.push(generate_multi_unnamed_children_arm(name, vname, fields)?);
      },
    }
  }

  Ok(quote! {
      impl #name {
          pub fn recurse_children<T: crate::visitor::transformer::AstTransformer + ?Sized>(
              self,
              t: &mut T,
              arena: &mut crate::ast::AstArena,
          ) -> Self {
              match self {
                  #(#recurse_arms,)*
              }
          }

          pub fn children(&self) -> Vec<crate::ast::NodeId> {
              match self {
                  #(#children_arms,)*
              }
          }
      }
  })
}

fn generate_skip_arms(enum_name: &syn::Ident, variant_name: &syn::Ident, fields: &Fields) -> (TokenStream, TokenStream) {
  match fields {
    Fields::Unit => (quote! { #enum_name::#variant_name => #enum_name::#variant_name }, quote! { #enum_name::#variant_name => Vec::new() }),
    Fields::Unnamed(_) => {
      (quote! { #enum_name::#variant_name(inner) => #enum_name::#variant_name(inner) }, quote! { #enum_name::#variant_name(_) => Vec::new() })
    },
    Fields::Named(fields) => {
      let field_names: Vec<_> = fields.named.iter().map(|f| f.ident.as_ref().expect("named field")).collect();
      (
        quote! {
            #enum_name::#variant_name { #(#field_names),* } => #enum_name::#variant_name { #(#field_names),* }
        },
        quote! {
            #enum_name::#variant_name { .. } => Vec::new()
        },
      )
    },
  }
}

fn generate_single_field_recurse_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, field_ty: &Type) -> TokenStream {
  let strategy = classify_type(field_ty);
  match &strategy {
    WalkStrategy::Passthrough => {
      quote! { #enum_name::#variant_name(inner) => #enum_name::#variant_name(inner) }
    },
    WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
      let walk_fn = walk_fn_path(&strategy).expect("walk fn");
      quote! {
          #enum_name::#variant_name(inner) => {
              #enum_name::#variant_name(#walk_fn(t, inner, arena))
          }
      }
    },
    WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
      let walk_fn = walk_fn_path(&strategy).expect("walk fn");
      quote! {
          #enum_name::#variant_name(inner) => {
              #enum_name::#variant_name(inner.map(|id| #walk_fn(t, id, arena)))
          }
      }
    },
    WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
      let walk_fn = walk_fn_path(&strategy).expect("walk fn");
      quote! {
          #enum_name::#variant_name(inner) => {
              #enum_name::#variant_name(
                  inner.into_iter().map(|id| #walk_fn(t, id, arena)).collect()
              )
          }
      }
    },
    WalkStrategy::WalkableStruct => {
      if is_vec_type(field_ty) {
        quote! {
            #enum_name::#variant_name(inner) => {
                #enum_name::#variant_name(
                    inner.into_iter()
                        .map(|item| item.recurse_children(t, arena))
                        .collect()
                )
            }
        }
      } else {
        quote! {
            #enum_name::#variant_name(inner) => {
                #enum_name::#variant_name(inner.recurse_children(t, arena))
            }
        }
      }
    },
    WalkStrategy::OptionWalkableStruct => {
      quote! {
          #enum_name::#variant_name(inner) => {
              #enum_name::#variant_name(inner.map(|item| item.recurse_children(t, arena)))
          }
      }
    },
  }
}

fn generate_named_fields_recurse_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, fields: &syn::FieldsNamed) -> Result<TokenStream> {
  let mut bindings = Vec::new();
  let mut constructions = Vec::new();

  for field in &fields.named {
    let fname = field.ident.as_ref().expect("named field");
    bindings.push(quote! { #fname });
    let strategy = classify_type(&field.ty);
    let expr = match &strategy {
      WalkStrategy::Passthrough => quote! { #fname },
      WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn");
        quote! { #walk_fn(t, #fname, arena) }
      },
      WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn");
        quote! { #fname.map(|id| #walk_fn(t, id, arena)) }
      },
      WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn");
        quote! {
            #fname.into_iter().map(|id| #walk_fn(t, id, arena)).collect()
        }
      },
      WalkStrategy::WalkableStruct => {
        if is_vec_type(&field.ty) {
          quote! {
              #fname.into_iter()
                  .map(|item| item.recurse_children(t, arena))
                  .collect()
          }
        } else {
          quote! { #fname.recurse_children(t, arena) }
        }
      },
      WalkStrategy::OptionWalkableStruct => {
        quote! { #fname.map(|item| item.recurse_children(t, arena)) }
      },
    };
    constructions.push(quote! { #fname: #expr });
  }

  Ok(quote! {
      #enum_name::#variant_name { #(#bindings),* } => {
          #enum_name::#variant_name { #(#constructions),* }
      }
  })
}

fn generate_multi_unnamed_recurse_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, fields: &syn::FieldsUnnamed) -> Result<TokenStream> {
  let mut bindings = Vec::new();
  let mut exprs = Vec::new();

  for (i, field) in fields.unnamed.iter().enumerate() {
    let binding = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
    let strategy = classify_type(&field.ty);
    let expr = match &strategy {
      WalkStrategy::Passthrough => quote! { #binding },
      WalkStrategy::ExprId | WalkStrategy::StmtId | WalkStrategy::PatternId | WalkStrategy::TypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn");
        quote! { #walk_fn(t, #binding, arena) }
      },
      WalkStrategy::VecExprId | WalkStrategy::VecStmtId | WalkStrategy::VecPatternId | WalkStrategy::VecTypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn");
        quote! { #binding.into_iter().map(|id| #walk_fn(t, id, arena)).collect() }
      },
      WalkStrategy::OptionExprId | WalkStrategy::OptionStmtId | WalkStrategy::OptionPatternId | WalkStrategy::OptionTypeExprId => {
        let walk_fn = walk_fn_path(&strategy).expect("walk fn");
        quote! { #binding.map(|id| #walk_fn(t, id, arena)) }
      },
      WalkStrategy::WalkableStruct => {
        if is_vec_type(&field.ty) {
          quote! { #binding.into_iter().map(|item| item.recurse_children(t, arena)).collect() }
        } else {
          quote! { #binding.recurse_children(t, arena) }
        }
      },
      WalkStrategy::OptionWalkableStruct => {
        quote! { #binding.map(|item| item.recurse_children(t, arena)) }
      },
    };
    bindings.push(binding);
    exprs.push(expr);
  }

  Ok(quote! {
      #enum_name::#variant_name(#(#bindings),*) => {
          #enum_name::#variant_name(#(#exprs),*)
      }
  })
}

fn is_vec_type(ty: &Type) -> bool {
  if let Type::Path(tp) = ty
    && let Some(seg) = tp.path.segments.last()
    && seg.ident == "Vec"
    && let PathArguments::AngleBracketed(args) = &seg.arguments
  {
    return !args.args.is_empty();
  }
  false
}
