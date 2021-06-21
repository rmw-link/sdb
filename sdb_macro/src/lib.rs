extern crate proc_macro;
use proc_macro::TokenStream;
extern crate syn;
#[macro_use]
extern crate quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Sdb)]
pub fn sdb(input: TokenStream) -> TokenStream {
  let ast = parse_macro_input!(input as DeriveInput);
  let name = &ast.ident;

  let expanded = quote! {
    sdb::repr!(#name);
  };
  TokenStream::from(expanded)
}

/*
fn sdb_repr(ast: &syn::MacroInput) -> quote::Tokens {
  quote! {
  }
}
*/
