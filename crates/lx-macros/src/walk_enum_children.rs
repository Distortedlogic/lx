use proc_macro2::TokenStream;
use quote::quote;
use syn::{Result, Type};

use crate::field_strategy::{classify_type, is_single_id, is_vec_type, node_id_expr};

pub fn generate_single_field_children_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, field_ty: &Type) -> TokenStream {
  let strategy = classify_type(field_ty);
  let is_vec = is_vec_type(field_ty);
  let field_ref = if is_single_id(&strategy) {
    quote! { *inner }
  } else {
    quote! { inner }
  };

  match node_id_expr(&strategy, &field_ref, is_vec) {
    Some(expr) => {
      quote! { #enum_name::#variant_name(inner) => #expr }
    },
    None => {
      quote! { #enum_name::#variant_name(_) => smallvec::smallvec![] }
    },
  }
}

pub fn generate_named_fields_children_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, fields: &syn::FieldsNamed) -> Result<TokenStream> {
  let mut bindings = Vec::new();
  let mut child_extends = Vec::new();
  let mut has_any_children = false;

  for field in &fields.named {
    let fname = field.ident.as_ref().expect("named field");
    let strategy = classify_type(&field.ty);
    let is_vec = is_vec_type(&field.ty);
    let field_ref = if is_single_id(&strategy) {
      quote! { *#fname }
    } else {
      quote! { #fname }
    };
    if let Some(expr) = node_id_expr(&strategy, &field_ref, is_vec) {
      bindings.push(quote! { #fname });
      child_extends.push(quote! { result.extend(#expr); });
      has_any_children = true;
    }
  }

  if !has_any_children {
    Ok(quote! {
        #enum_name::#variant_name { .. } => smallvec::smallvec![]
    })
  } else {
    Ok(quote! {
        #enum_name::#variant_name { #(#bindings,)* .. } => {
            let mut result = smallvec::SmallVec::new();
            #(#child_extends)*
            result
        }
    })
  }
}

pub fn generate_multi_unnamed_children_arm(enum_name: &syn::Ident, variant_name: &syn::Ident, fields: &syn::FieldsUnnamed) -> Result<TokenStream> {
  let mut bindings = Vec::new();
  let mut child_extends = Vec::new();

  for (i, field) in fields.unnamed.iter().enumerate() {
    let binding = syn::Ident::new(&format!("f{i}"), proc_macro2::Span::call_site());
    let strategy = classify_type(&field.ty);
    let is_vec = is_vec_type(&field.ty);
    let field_ref = if is_single_id(&strategy) {
      quote! { *#binding }
    } else {
      quote! { #binding }
    };
    if let Some(expr) = node_id_expr(&strategy, &field_ref, is_vec) {
      child_extends.push(quote! { result.extend(#expr); });
    }
    bindings.push(binding);
  }

  if child_extends.is_empty() {
    Ok(quote! {
        #enum_name::#variant_name(..) => smallvec::smallvec![]
    })
  } else {
    Ok(quote! {
        #enum_name::#variant_name(#(#bindings),*) => {
            let mut result = smallvec::SmallVec::new();
            #(#child_extends)*
            result
        }
    })
  }
}
