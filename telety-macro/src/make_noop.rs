use proc_macro2::TokenStream;
use quote::quote;
use syn::{ parse::{Parse, ParseStream}, parse2, Ident, Token};

struct MakeNoopArgs {
    export_ident: Ident,
    _comma: Token![,],
    textual_ident: Ident,
}

impl Parse for MakeNoopArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            export_ident: input.parse()?,
            _comma: input.parse()?,
            textual_ident: input.parse()?,
        })
    }
}

pub(crate) fn make_noop_impl(arg: TokenStream) -> syn::Result<TokenStream> {
    let MakeNoopArgs {
        export_ident,
        _comma,
        textual_ident,
    } = parse2(arg)?;

    Ok(quote! {
        #[doc(hidden)]
        #[macro_export]
        macro_rules! #export_ident {
            ($dollar:tt) => {
                macro_rules! #textual_ident {
                    ($dollar ($dollar tokens:tt)*) => { };
                }
            };
        }
    })
}