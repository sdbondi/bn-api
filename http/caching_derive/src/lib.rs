extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{Meta, MetaNameValue, Lit};
use syn;


#[proc_macro_derive(ToETag, attributes(etag_field))]
pub fn to_etag_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_to_etag(&ast)
}

fn impl_to_etag(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let attrs = &ast.attrs;

    let etag_field = attrs.into_iter()
	.map(|attr| attr.parse_meta().unwrap())
	.find_map(|meta| {
	     match meta {
		// Match '#[ident = lit]' attributes. Match guard makes it `#[etag_fields = "a,b,c"]`
		Meta::NameValue(MetaNameValue { ref ident, ref lit, .. }) if ident == "etag_field" => {
		    if let Lit::Str(lit) = lit {
			Some(lit.value())
		    } else {
			None
		    }
		},

		_ => None
	    }
	}).unwrap_or("updated_at".to_string());

    let path: syn::ExprPath =

    let var_ident = syn::Ident::new(&format!("self.{}", etag_field), );

    let gen = quote! {
	use bigneon_http::caching::{ETag, ToETag, EntityTag};

	impl ToETag for #name {
	    fn to_etag(&self) -> ETag {
		ETag(EntityTag::weak(self.#etag_field.to_string()))
	    }
	}
    };
    gen.into()
}
