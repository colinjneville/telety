mod crateify;
mod find_and_replace;
mod make_noop;
mod telety;

#[proc_macro_attribute]
pub fn telety(
    attr_arg: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let (Ok(ts) | Err(ts)) =
        telety::telety_impl(attr_arg.into(), item.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}

#[proc_macro]
pub fn crateify(arg: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(ts) | Err(ts)) =
        crateify::crateify(arg.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}

#[proc_macro]
pub fn find_and_replace(arg: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(ts) | Err(ts)) =
        find_and_replace::find_and_replace(arg.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}

#[proc_macro]
pub fn make_noop(arg: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(ts) | Err(ts)) =
        make_noop::make_noop_impl(arg.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}
