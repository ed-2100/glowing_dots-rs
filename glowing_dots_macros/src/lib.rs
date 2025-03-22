use std::ffi::CString;

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{parse::{Parse, ParseStream}, parse_macro_input, Ident};

struct FooInput {
    ident: Ident
}

impl Parse for FooInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(FooInput { ident })
    }
}

#[proc_macro]
pub fn cstringify(input: TokenStream) -> TokenStream {
    let FooInput { ident } = parse_macro_input!(input as FooInput);
    let ident_str = CString::new(ident.to_string()).unwrap();
    let cstr_lit = proc_macro2::Literal::c_string(&ident_str);
    cstr_lit.to_token_stream().into()
}
