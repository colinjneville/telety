use std::borrow::Cow;

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, TokenStreamExt as _};
use syn::{parse_quote, spanned::Spanned as _};

use crate::{syn_util, version, alias, Alias};

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct Public<'map>(Alias<'map>);

impl<'map> Public<'map> {
    pub(crate) fn new(alias: Alias<'map>) -> Self {
        Self(alias)
    }
}

impl<'map> quote::ToTokens for Public<'map> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Alias {
            map,
            path,
            index,
            ref arguments,
        } = self.0;

        let ident = index.ident();

        let visibility = map.visibility();
        let super_visibility = syn_util::super_visibility(visibility);

        let alias_tokens = if let alias::Index::Primary = index {
            let aliased_type_path = &path.truncated_path;

            quote!(#super_visibility use #aliased_type_path as #ident;)
        } else {
            let ident_internal = index.ident_internal();

            let aliased_path = &path.truncated_path;
            let span = aliased_path.span();
            let parameters = &arguments.args;

            let unique_ident = map.unique_ident();

            let alias_unique_ident = format_ident!("{unique_ident}_{ident}");
            let macro_maker_ident = format_ident!("make_{alias_unique_ident}");
            let submodule_ident = format_ident!("{ident}_mod");

            let macro_vis = syn_util::visibility_macro_export(map.visibility());            

            let telety_path_override = map.telety_path();
            let telety_path = telety_path_override
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(parse_quote!(::telety)));

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
            if aliased_path.segments.len() == 1 {
                let needle = syn::Ident::new("__needle", Span::call_site());

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
                    .with_fallback(TokenStream::new())
                    .with_macro_forwarding(macro_maker_ident);

                if let Some(telety_path_override) = telety_path_override {
                    exported_apply = exported_apply.with_telety_path(telety_path_override.clone());
                }

                quote! {
                    // Create an exported macro. If the type's macro existed, it is a forwarder.
                    // If it did not exist, it is a noop
                    #exported_apply

                    // Create an alias for just the type
                    #super_visibility type #alias_unique_ident #parameters = self::exact::#ident_internal #parameters;

                    #super_visibility use #alias_unique_ident as #ident;
                }
            } else {
                let super2_visibility = syn_util::super_visibility(&super_visibility);
                let super3_visibility = syn_util::super_visibility(&super2_visibility);

                quote_spanned! { span =>
                    // Setup for a glob import
                    mod #submodule_ident {
                        #super2_visibility use super::exact::#ident as #ident;

                        pub(super) mod globbed {
                            // Use the macro if it exists. The type will be imported, but...
                            #super3_visibility use super::*;
                            // Overwritten by our 'reduced generics' type alias
                            #super3_visibility type #ident #parameters = super::super::exact::#ident_internal #parameters;
                        }
                    }

                    #super_visibility use #submodule_ident::globbed::#ident;
                }
            }
        };

        tokens.append_all(alias_tokens);
    }
}
