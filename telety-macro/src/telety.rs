use proc_macro2::TokenStream;
use syn::{parse2, Item};
use telety_impl::Telety;

pub(crate) fn telety_impl(attr_args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let mut item: Item = parse2(item)?;
    // The telety attribute needs to be 'reattached' to the token stream so that
    // the Telety object can be fully recreated from the macro contents.
    Telety::prepend_attribute(&mut item, attr_args)?;

    let telety = Telety::new(&item)?;

    telety.appendix()
}
