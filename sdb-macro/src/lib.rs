extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro_derive(Sdb)]
pub fn sdb(input: TokenStream) -> TokenStream {
  "fn answer() -> u32 { 42 }".parse().unwrap()
}
