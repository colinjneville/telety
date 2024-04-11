use std::mem;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse2, parse_quote, Macro, Token,
};

struct TryInvokeArgs {
    maybe_macro: Macro,
    semicolon: Option<Token![;]>,
    fallback_tokens: TokenStream,
}

impl Parse for TryInvokeArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            maybe_macro: input.parse()?,
            semicolon: input.parse()?,
            fallback_tokens: input.parse()?,
        })
    }
}

pub(crate) fn try_invoke_impl(arg: TokenStream) -> syn::Result<TokenStream> {
    let TryInvokeArgs {
        mut maybe_macro,
        semicolon,
        fallback_tokens,
    } = parse2(arg)?;

    let maybe_macro_path = mem::replace(&mut maybe_macro.path, parse_quote!(__macro_fallback));

    Ok(quote! {
        const _: () = {
            macro_rules! __macro_fallback_adapter {
                ($($tokens:tt)*) => {
                    #fallback_tokens
                };
            }

            #[allow(unused_imports)]
            use __macro_fallback_adapter as __macro_fallback;
            const _: () = {
                #[allow(unused_imports)]
                use #maybe_macro_path as __macro_fallback;
                #maybe_macro #semicolon
            };
        };
    })
}
