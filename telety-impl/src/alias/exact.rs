use std::borrow::Cow;

use quote::{TokenStreamExt as _, quote_spanned};
use syn::{parse_quote, spanned::Spanned as _};

use crate::{Alias, alias, syn_util};

#[repr(transparent)]
#[derive(Debug)]
pub(crate) struct Exact<'m>(Alias<'m>);

impl<'m> Exact<'m> {
    pub(crate) fn new(alias: Alias<'m>) -> Self {
        Self(alias)
    }
}

impl<'m> quote::ToTokens for Exact<'m> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Alias {
            map,
            path,
            index,
            ref arguments,
            kind,
        } = self.0;

        if index == alias::Index::Primary {
            // We already know the full path to Self, so we don't need an exact alias
            return;
        }

        let alias_path = &path.truncated_path;
        let ident = index.ident(path.friendly_path());
        let ident_internal = index.ident_internal(path.friendly_path());

        let span = path.truncated_path.span();
        let visibility = map.visibility();
        let super_visibility = syn_util::super_visibility(visibility);
        let super2_visibility = syn_util::super_visibility(&super_visibility);

        let item_use = {
            // If path length is one, we may not be allowed to reexport at the desired vis,
            // so export as `pub(super)` and the alias generation will use a convoluted workaround
            let use_visibility = if kind == alias::Kind::Trait {
                Cow::Borrowed(&super2_visibility)
            } else if alias_path.segments.len() == 1 {
                Cow::Owned(parse_quote!(pub(super)))
            } else {
                Cow::Borrowed(&super2_visibility)
            };

            quote_spanned! { span =>
                // Create a fixed alias for our submodule to reference
                #use_visibility use #alias_path as #ident;
            }
        };
        let item_type = match kind {
            alias::Kind::Type => {
                let parameters = &arguments.args;
                Some(quote_spanned! { span =>
                    #super_visibility type #ident_internal #parameters = #alias_path #parameters;
                })
            }
            // Traits can't have type aliases
            alias::Kind::Trait => None,
        };

        let alias_tokens = quote_spanned! { span =>
            #item_use
            #item_type
        };

        tokens.append_all(alias_tokens);
    }
}
