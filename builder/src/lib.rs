use proc_macro::TokenStream;
use quote::ToTokens;
use syn::DeriveInput;

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let _ = input;
    let ast: DeriveInput = syn::parse(input).unwrap();

    derive_impl(&ast);
    TokenStream::new()
}

fn derive_impl(ast: &DeriveInput) -> TokenStream {
    TokenStream::from(ast.to_token_stream())
}
