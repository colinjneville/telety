use std::borrow::Cow;

use quote::{quote, quote_spanned, TokenStreamExt as _};
use syn::{parse_quote, spanned::Spanned as _};

use crate::{syn_util, alias, Alias};


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
            target,
            index,
        } = self.0;

        if let alias::Index::Primary = index {
            // We know a telety macro exists for this type, so all we need is to unify
            // the macro and the trait in the public alias.
            return;
        }

        let aliased_type = &target.aliased_type;
        let ident = index.ident();
        let ident_internal = index.ident_internal();

        let span = target.aliased_type.span();
        let visibility = map.visibility();
        let super_visibility = syn_util::super_visibility(visibility);
        let super2_visibility = syn_util::super_visibility(&super_visibility);
    
        let item_use =
            if !self.0.is_generic_parameter_type() {
                if let Some(_qself) = &aliased_type.qself {
                    // Only simple paths can be used in use statements
                    quote!()
                } else {
                    // If path length is one, we may not be allowed to reexport at the desired vis,
                    // so export as `pub(super)` and the alias generation will use a convoluted workaround
                    let use_visibility = if aliased_type.path.segments.len() == 1 {
                        Cow::Owned(parse_quote!(pub(super)))
                    } else {
                        Cow::Borrowed(&super2_visibility)
                    };
    
                    let mut aliased_type_no_generics = aliased_type.clone();
                    aliased_type_no_generics
                        .path
                        .segments
                        .last_mut()
                        .expect("Path should have at least one segment")
                        .arguments = syn::PathArguments::None;
    
                    quote_spanned! { span =>
                        // Create a fixed alias for our submodule to reference
                        #use_visibility use #aliased_type_no_generics as #ident;
                    }
                }
            } else {
                quote!()
            };
    
        let mut parameters = target.generics.clone();
        // `type` aliases should not have bounds
        syn_util::remove_generics_bounds(&mut parameters);
        
        let alias_tokens = quote_spanned! { span =>
            #item_use
            #super_visibility type #ident_internal #parameters = #aliased_type;
        };

        tokens.append_all(alias_tokens);
    }
}