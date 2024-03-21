use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{parse2, visit_mut::VisitMut as _, Item};
use telety_impl::visitor;

pub(crate) fn crateify(arg: TokenStream) -> syn::Result<TokenStream> {
    let mut item: Item = parse2(arg)?;
    let mut visitor = visitor::Crateify::new();
    visitor.visit_item_mut(&mut item);
    
    Ok(item.to_token_stream())
}

