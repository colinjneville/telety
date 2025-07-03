use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote_spanned, TokenStreamExt as _};
use syn::{
    parse2, parse_quote, parse_quote_spanned, spanned::Spanned as _, Attribute, Ident, Item, LitInt, Visibility
};
use telety_impl::{version, visitor, Options, Telety};

pub(crate) fn telety_impl(attr_args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let item: Item = parse2(item)?;
    let mut options: Options = parse2(attr_args)?;
    directed_visit::visit_mut(
        &mut directed_visit::syn::direct::FullDefault,
        &mut visitor::Decrateify::new(),
        &mut options.module_path
    );

    let telety = Telety::new_with_options(&item, options)?;

    generate_output(&telety)
}

fn generate_output(telety: &Telety) -> syn::Result<TokenStream> {
    let map = telety.alias_map();

    let unique_ident = map.unique_ident();
    let textual_ident = format_ident!("{unique_ident}_telety_impl");

    let vis = telety.visibility();

    let item_macro = generate_macro(telety, &textual_ident)?;

    let macro_ident = telety.macro_ident();

    let item = if let Some(proxy) = &telety.options().proxy {
        Cow::Owned(parse_quote! {
            #vis use #proxy::{self as #textual_ident};
        })
    } else {
        Cow::Borrowed(telety.item())
    };

    let span = Span::call_site();

    let map_module = map.with_module();

    Ok(quote_spanned! { span =>
        #item

        #item_macro

        #[doc(hidden)]
        #vis use #textual_ident as #macro_ident;

        #map_module
    })
}

pub(crate) fn macro_export(telety: &Telety) -> Option<Attribute> {
    match telety.visibility() {
        Visibility::Public(vis_pub) => {
            let span = vis_pub.span;
            Some(parse_quote_spanned!(span => #[macro_export]))
        }
        Visibility::Restricted(_vis_restricted) => None,
        Visibility::Inherited => None,
    }
}

pub(crate) fn generate_macro(telety: &Telety, ident: &Ident) -> syn::Result<TokenStream> {
    let span = telety.item().span();

    let mut arms = TokenStream::new();
    for &(version, commands) in version::VERSIONS {
        for command in commands {
            let arm = command.generate_macro_arm(telety)?;
            arms.append_all(arm);
        }

        let version = LitInt::new(&version.to_string(), span);

        if cfg!(feature = "full-errors") {
            arms.append_all(quote_spanned! { span =>
                (#version $command:ident $($tokens:tt)*) => {
                    compile_error!(concat!("No command '",  stringify!($command), "' for version ", stringify!(#version)));
                };
                (#version $($tokens:tt)*) => {
                    compile_error!("Expected a command");
                };
            });
        }
    }

    if cfg!(feature = "full-errors") {
        arms.append_all(quote_spanned! { span =>
            ($version:literal $($tokens:tt)*) => {
                compile_error!(concat!("Unsupported version ", stringify!($version)));
            };
            ($($tokens:tt)*) => {
                compile_error!("Version not provided");
            };
        });
    }

    let macro_export = macro_export(telety);

    Ok(quote_spanned! { span =>
        #[doc(hidden)]
        #macro_export
        macro_rules! #ident {
            #arms
        }
    })
}
