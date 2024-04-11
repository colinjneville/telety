use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned, TokenStreamExt as _};
use syn::{
    parse2, parse_quote, parse_quote_spanned, spanned::Spanned as _, visit_mut::VisitMut as _,
    Attribute, GenericParam, Ident, Item, LitInt, PathArguments, Type, Visibility,
};
use telety_impl::{version, visitor, Alias, Options, Telety};

use crate::{alias_definition::Definition, syn_util};

pub(crate) fn telety_impl(attr_args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let item: Item = parse2(item)?;
    let mut options: Options = parse2(attr_args)?;
    visitor::Decrateify::new().visit_path_mut(&mut options.module_path);

    let telety = Telety::new_with_options(&item, options)?;

    generate_output(&telety)
}

fn generate_output(telety: &Telety) -> syn::Result<TokenStream> {
    let item = telety.item();

    let textual_ident = telety.unique_ident();

    let vis = telety.visibility();

    let alias_mod = generate_alias_mod(telety)?;

    let item_macro = generate_macro(telety, textual_ident)?;

    let macro_ident = telety.macro_ident();

    let span = Span::call_site();

    Ok(quote_spanned! { span =>
        #item

        #item_macro

        #[doc(hidden)]
        #vis use #textual_ident as #macro_ident;

        #alias_mod
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
        arms.append_all(quote_spanned! { span =>
            (#version $command:ident $($tokens:tt)*) => {
                compile_error!(concat!("No command '",  stringify!($command), "' for version ", stringify!(#version)));
            };
            (#version $($tokens:tt)*) => {
                compile_error!("Expected a command");
            };
        });
    }
    arms.append_all(quote_spanned! { span =>
        ($version:literal $($tokens:tt)*) => {
            compile_error!(concat!("Unsupported version ", stringify!($version)));
        };
        ($($tokens:tt)*) => {
            compile_error!("Version not provided");
        };
    });

    let macro_export = macro_export(telety);

    Ok(quote_spanned! { span =>
        #[doc(hidden)]
        #macro_export
        macro_rules! #ident {
            #arms
        }
    })
}

pub(crate) fn generate_alias_mod(telety: &Telety) -> syn::Result<TokenStream> {
    let vis = telety.visibility();

    // items inside our module need to convert blank vis to `pub(super)`, etc.
    let super_vis = syn_util::sublevel_visibility(vis);

    let mut aliases = TokenStream::new();
    for alias in telety.iter_aliases() {
        aliases.append_all(generate_alias(telety, &super_vis, alias));
    }

    let mod_ident = telety.module_ident();

    let exact_alias_mod = generate_exact_alias_mod(telety, &super_vis)?;

    let span = Span::call_site();
    Ok(quote_spanned! { span =>
        #[doc(hidden)]
        #[allow(dead_code)]
        #[allow(unused_macros)]
        #[allow(unused_imports)]
        #vis mod #mod_ident {
            #exact_alias_mod

            #aliases
        }
    })
}

fn generate_exact_alias_mod(telety: &Telety, vis: &Visibility) -> syn::Result<TokenStream> {
    let span = Span::call_site();

    let super_vis = syn_util::sublevel_visibility(vis);

    let exact_aliases: syn::Result<Vec<TokenStream>> = telety
        .iter_aliases()
        .map(|a| generate_exact_alias(telety, &super_vis, a))
        .collect();
    let exact_aliases = exact_aliases?;

    Ok(quote_spanned! { span =>
        mod exact {
            #super_vis use super::super::*;

            #(#exact_aliases)*
        }
    })
}

fn generate_exact_alias(
    telety: &Telety,
    vis: &Visibility,
    alias: Alias,
) -> syn::Result<TokenStream> {
    let aliased_type = alias.aliased_type();
    let span = aliased_type.span();

    let Definition {
        ident,
        internal_ident,
        ..
    } = Definition::new(alias, telety.unique_ident().clone());

    let item_use =
        if let (Type::Path(type_path), false) = (aliased_type, is_generic_parameter_type(alias)) {
            // If path length is one, we may not be allowed to reexport at the desired vis,
            // so export as `pub(super)` and the alias generation will use a convoluted workaround
            let vis = if type_path.path.segments.len() == 1 {
                Cow::Owned(parse_quote!(pub(super)))
            } else {
                Cow::Borrowed(vis)
            };

            let mut aliased_type_no_generics = type_path.clone();
            aliased_type_no_generics
                .path
                .segments
                .last_mut()
                .expect("Path should have at least one segment")
                .arguments = PathArguments::None;

            quote_spanned! { span =>
                // Create a fixed alias for our submodule to reference
                #vis use #aliased_type_no_generics as #ident;
            }
        } else {
            quote!()
        };

    let mut parameters = alias.parameters().clone();
    // `type` aliases should not have bounds
    syn_util::remove_generics_bounds(&mut parameters);

    Ok(quote_spanned! { span =>
        #item_use
        #vis type #internal_ident #parameters = #aliased_type;
    })
}

fn generate_alias(telety: &Telety, vis: &Visibility, alias: Alias) -> syn::Result<TokenStream> {
    let aliased_type = alias.aliased_type();
    let span = aliased_type.span();

    let Definition {
        ident,
        internal_ident,
        macro_maker_ident,
        alias_unique_ident,
        submodule_ident,
    } = Definition::new(alias, telety.unique_ident().clone());

    let macro_vis = macro_export(telety);

    let super_vis = syn_util::sublevel_visibility(vis);
    let super_super_vis = syn_util::sublevel_visibility(&super_vis);

    let mut parameters = alias.parameters().clone();
    // `type` aliases should not have bounds
    syn_util::remove_generics_bounds(&mut parameters);

    let telety_path = telety.options().telety_path();

    // Only non-type parameter path types can have an 'embedded' macro
    if let (Type::Path(type_path), false) = (aliased_type, is_generic_parameter_type(alias)) {
        let mut aliased_type_no_generics = type_path.clone();
        aliased_type_no_generics
            .path
            .segments
            .last_mut()
            .expect("Path should have at least one segment")
            .arguments = PathArguments::None;

        // We are allowed to export items *in* private modules at their original visibility. e.g.
        // ```rust
        // use my_crate;
        // pub use my_crate::MyPubStruct;
        // ```
        // But we can't re-export items themselves at greater visibility than our import, even if their
        // original visibility is greater. This is invalid:
        // ```rust
        // use my_crate::MyPubStruct;
        // pub use MyPubStruct as MyPubReexport;
        // ```
        // We can work around this, but it requires 2 extra exported macros per item, so prefer the simple
        // way if we have a multi-segment path as our type.
        if type_path.path.segments.len() == 1 {
            let needle: Ident = parse_quote!(__needle);

            let haystack = quote! {
                #[doc(hidden)]
                #macro_vis
                macro_rules! #alias_unique_ident {
                    ($($tokens:tt)*) => {
                        #telety_path::__private::crateify! {
                            #needle!($($tokens)*);
                        };
                    };
                }
            };

            let mut exported_apply = version::v0::PATH
                .apply(parse_quote!(self::exact::#ident), needle.clone(), &haystack)
                .with_fallback(&TokenStream::new())
                .with_macro_forwarding(macro_maker_ident);

            if let Some(telety_path) = telety.options().telety_path.as_ref() {
                exported_apply = exported_apply.with_telety_path(telety_path.clone());
            }

            Ok(quote! {
                // Create an exported macro. If the type's macro existed, it is a forwarder.
                // If it did not exist, it is a noop
                #exported_apply

                // Create an alias for just the type
                #vis type #alias_unique_ident #parameters = self::exact::#internal_ident #parameters;

                #vis use #alias_unique_ident as #ident;
            })
        } else {
            Ok(quote_spanned! { span =>
                // Setup for a glob import
                mod #submodule_ident {
                    #super_vis use super::exact::#ident as #ident;

                    pub(super) mod globbed {
                        // Use the macro if it exists. The type will be imported, but...
                        #super_super_vis use super::*;
                        // Overwritten by our 'reduced generics' type alias
                        #super_super_vis type #ident #parameters = super::super::exact::#internal_ident #parameters;
                    }
                }

                #vis use #submodule_ident::globbed::#ident;
            })
        }
    } else {
        // TODO In the future, we could do some special handling to support some non-path types,
        // but for now do not provide a macro
        Ok(quote_spanned! { span =>
            #vis use self::exact::#internal_ident as #ident;
        })
    }
}

/// Is this alias for a type parameter?  
/// Only lone type parameters are included (i.e. `T`, but not `Vec<T>`).  
fn is_generic_parameter_type(alias: Alias) -> bool {
    if let Type::Path(type_path) = &alias.aliased_type() {
        let mut iter = alias.parameters().params.iter();
        if let (Some(GenericParam::Type(single)), None) = (iter.next(), iter.next()) {
            return type_path.path.is_ident(&single.ident);
        }
    }
    false
}
