use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{Item, parse2};
use telety_impl::visitor;

pub(crate) fn crateify(arg: TokenStream) -> syn::Result<TokenStream> {
    let mut item: Item = parse2(arg)?;
    directed_visit::visit_mut(
        &mut directed_visit::syn::direct::FullDefault,
        &mut visitor::Crateify::new(),
        &mut item,
    );

    Ok(item.to_token_stream())
}
