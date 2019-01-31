#![deny(unreachable_patterns)]
#![deny(unknown_lints)]
#![cfg_attr(not(debug_assertions), deny(unused_variables))]
#![cfg_attr(not(debug_assertions), deny(unused_imports))]
#![cfg_attr(not(debug_assertions), deny(dead_code))]
// Unused results is more often than not an error
#![deny(unused_must_use)]
#![cfg_attr(not(debug_assertions), deny(unused_extern_crates))]
extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::*;

#[proc_macro_derive(ToETag)]
pub fn to_etag_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_to_etag(&ast)
}

fn help(msg: &str) -> ! {
    panic!("#[derive(ToETag)] {}", msg);
}

fn impl_to_etag(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    match &ast.data {
	Data::Struct(_) => {}
	_ => help("must be used on a struct"),
    }

    let name_str = format!("{}", name);
    let convert_to_etag_code = quote! {
    {
    let mut s = format!("{}{}", #name_str, json!(self));
    ::bigneon_http::caching::etag_hash(&s)
    }
    };

    let gen = quote! {
    use bigneon_http::caching::{ETag, ToETag, EntityTag};

    impl ToETag for #name {
    fn to_etag(&self) -> ETag {
    let etag = #convert_to_etag_code;
    ETag(EntityTag::weak(etag))
    }
    }
    };
    gen.into()
}
