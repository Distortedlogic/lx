mod field_strategy;
mod walk_enum;
mod walk_enum_children;
mod walk_enum_walk;
mod walk_struct;

use proc_macro::TokenStream;
use syn::{Data, DeriveInput, parse_macro_input};

#[proc_macro_derive(AstWalk, attributes(walk))]
pub fn derive_ast_walk(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  let result = match &input.data {
    Data::Struct(_) => walk_struct::generate_struct_walk(&input),
    Data::Enum(_) => walk_enum::generate_enum_walk(&input),
    Data::Union(_) => {
      return syn::Error::new_spanned(&input.ident, "AstWalk cannot be derived for unions").to_compile_error().into();
    },
  };
  match result {
    Ok(tokens) => tokens.into(),
    Err(e) => e.to_compile_error().into(),
  }
}
