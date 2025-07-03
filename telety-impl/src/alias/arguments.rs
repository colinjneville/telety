use quote::format_ident;
use syn::{parse_quote_spanned, spanned::Spanned as _};

use crate::alias;

#[derive(Debug, Default, Clone)]
pub(crate) struct Arguments {
    pub(crate) args: Option<syn::AngleBracketedGenericArguments>,
    pub(crate) lifetime_count: usize,
    pub(crate) type_count: usize,
    pub(crate) const_count: usize,
}

impl Arguments {
    pub(crate) fn new(args: syn::PathArguments) -> alias::Result<Self> {
        let mut lifetime_count = 0;
        let mut type_count = 0;
        let mut const_count = 0;

        let args = match args {
            syn::PathArguments::None => 
                None,
            syn::PathArguments::AngleBracketed(angle_bracketed_generic_arguments) => {
                for generic in &angle_bracketed_generic_arguments.args {
                    match generic {
                        syn::GenericArgument::Lifetime(_lifetime) => 
                            lifetime_count += 1,
                        syn::GenericArgument::Type(_type_) => 
                            type_count += 1,
                        syn::GenericArgument::Const(_const_) => 
                            const_count += 1,
                        // TODO I believe the only time the following show in TypePaths are type aliases, which (mostly) ignore them
                        syn::GenericArgument::AssocType(_assoc_type) => { },
                        syn::GenericArgument::AssocConst(_assoc_const) => { },
                        syn::GenericArgument::Constraint(_constraint) => { },
                        _ => { },
                    }
                }

                if const_count == 0  && type_count == 0 && lifetime_count == 0 {
                    // Treat `<>` the same as no args for simplicity
                    None
                } else {
                    Some(angle_bracketed_generic_arguments)
                }
            }
            syn::PathArguments::Parenthesized(parenthesized_generic_arguments) => 
                return Err(alias::error::Kind::AssociatedType.error(parenthesized_generic_arguments.span())),
        };

        Ok(Self {
            args,
            lifetime_count,
            type_count,
            const_count,
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.args.is_none()
    }

    pub(crate) fn parameterize(&mut self) {
        if let Some(args) = self.args.as_mut() {
            let mut lifetime_index = 0;
            let mut type_index = 0;
            let mut const_index = 0;
            
            for generic in &mut args.args {
                match generic {
                    syn::GenericArgument::Lifetime(lifetime) => {
                        let span = lifetime.span();
                        lifetime.ident = format_ident!("l{lifetime_index}");
                        lifetime.ident.set_span(span);
                        lifetime_index += 1;
                    },
                    syn::GenericArgument::Type(type_) => {
                        let span = type_.span();
                        let ident = format_ident!("T{type_index}");
                        *type_ = parse_quote_spanned!(span => #ident);
                        type_index += 1;
                    }
                    syn::GenericArgument::Const(const_) => {
                        let span = const_.span();
                        let ident = format_ident!("C{const_index}");
                        *const_ = parse_quote_spanned!(span => #ident);
                        const_index += 1;
                    }
                    // TODO I believe the only time the following show in TypePaths are type aliases, which (mostly) ignore them
                    syn::GenericArgument::AssocType(_assoc_type) => { },
                    syn::GenericArgument::AssocConst(_assoc_const) => { },
                    syn::GenericArgument::Constraint(_constraint) => { },
                    _ => { },
                }
            }
        }
    }
}
