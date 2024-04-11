mod alias_definition;
mod crateify;
mod find_and_replace;
mod syn_util;
mod telety;
mod try_invoke;

/// Enable telety for an item.  
/// The first argument must be the path to the current module (e.g. `#[telety(crate::my_mod)]`).  
/// 
/// Optional arguments inlcude:  
/// * telety_path - Provide a path to the contents of the telety crate.  
///   `#[telety(crate::my_mod, telety_path = "::renamed_telety")]`  
///   By default this is `::telety`, but if you have renamed or re-exported the crate you can specify its location here.
/// * macro_ident - The identifier for the telety-generated macro.  
///   `#[telety(crate::my_mod, macro_ident = "MyStructImpl")]`  
///   Normally, telety generates a macro with the same identifier as the item. But items such as impls do not have identifiers,
///   so you can manually specify one.
/// * visibility - The visibility to generate macros and aliases at.  
///   `#[telety(crate::my_mod, visibility = "pub(crate)")]`  
///   telety uses the visibility of the item by default. If the item has no visibility (such as an impl) or you want a more
///   restrictive visibility, you can use this argument. The visibility must be equal or more restrictive than the item's visibility.
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
    let (Ok(ts) | Err(ts)) = crateify::crateify(arg.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}

#[proc_macro]
pub fn find_and_replace(arg: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(ts) | Err(ts)) =
        find_and_replace::find_and_replace(arg.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}
#[proc_macro]
pub fn try_invoke(arg: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let (Ok(ts) | Err(ts)) =
        try_invoke::try_invoke_impl(arg.into()).map_err(syn::Error::into_compile_error);
    ts.into()
}
