use proc_macro2::Ident;
use syn::{
    parse_quote, parse_quote_spanned, punctuated::Punctuated, Attribute, Expr, ExprPath, GenericArgument, GenericParam, Generics, Path, PathArguments, PathSegment, Token, Type, TypePath, VisRestricted, Visibility
};

pub(crate) fn visibility_macro_export(visibility: &Visibility) -> Option<Attribute> {
    match visibility {
        Visibility::Public(vis_pub) => {
            let span = vis_pub.span;
            Some(parse_quote_spanned!(span => #[macro_export]))
        }
        Visibility::Restricted(_vis_restricted) => None,
        Visibility::Inherited => None,
    }
}

pub(crate) fn generic_params_to_arguments(
    params: &Generics,
) -> Punctuated<GenericArgument, Token![,]> {
    let mut args = Punctuated::new();
    for pair in params.params.pairs() {
        args.push_value(generic_param_to_argument(pair.value()));
        if let Some(&comma) = pair.punct() {
            args.push_punct(*comma);
        }
    }
    args
}

fn generic_param_to_argument(param: &GenericParam) -> GenericArgument {
    match param {
        GenericParam::Lifetime(lifetime) => GenericArgument::Lifetime(lifetime.lifetime.clone()),
        GenericParam::Type(ty) => {
            let mut segments = Punctuated::new();
            segments.push_value(PathSegment {
                ident: ty.ident.clone(),
                arguments: PathArguments::None,
            });
            GenericArgument::Type(Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }))
        }
        GenericParam::Const(cnst) => {
            let mut segments = Punctuated::new();
            segments.push_value(PathSegment {
                ident: cnst.ident.clone(),
                arguments: PathArguments::None,
            });
            GenericArgument::Const(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }))
        }
    }
}

pub(crate) fn super_visibility(visibility: &Visibility) -> Visibility {
    match visibility {
        Visibility::Public(_) => visibility.clone(),
        Visibility::Restricted(restricted) => {
            let VisRestricted { in_token, path, .. } = restricted;

            if in_token.is_some() {
                let first_ident = &path.segments.first().unwrap().ident;

                if first_ident == "crate" {
                    visibility.clone()
                } else if first_ident == "self" {
                    let mut new_restricted = restricted.clone();
                    let new_ident = &mut new_restricted.path.segments.first_mut().unwrap().ident;
                    let span = new_ident.span();
                    *new_ident = Ident::new("super", span);
                    Visibility::Restricted(new_restricted)
                } else {
                    // Whether the path starts with super or a relative path, prepend a super
                    let mut new_restricted = restricted.clone();
                    new_restricted.path = Box::new(parse_quote!(super::#path));
                    Visibility::Restricted(new_restricted)
                }
            } else if path.is_ident("crate") {
                visibility.clone()
            } else if path.is_ident("self") {
                parse_quote!(pub(super))
            } else if path.is_ident("super") {
                parse_quote!(pub(in super::super))
            } else {
                unreachable!("Invalid visibility '{}'", path.get_ident().unwrap());
            }
        }
        Visibility::Inherited => parse_quote!(pub(super)),
    }
}

#[cfg(test)]
mod test_sublevel_visibility {
    use quote::ToTokens as _;
    use syn::parse_quote;

    use super::*;

    fn test_sublevel_visibility(before: &syn::Visibility, after: &syn::Visibility) {
        let actual_after = super_visibility(before);
        assert_eq!(
            after.to_token_stream().to_string(),
            actual_after.to_token_stream().to_string(),
        );
    }

    #[test]
    fn test() {
        test_sublevel_visibility(
            &parse_quote!(pub(in self)),
            &parse_quote!(pub(in super)),
        );

        test_sublevel_visibility(
            &parse_quote!(pub(self)),
            &parse_quote!(pub(super)),
        );

        test_sublevel_visibility(
            &parse_quote!(pub(super)),
            &parse_quote!(pub(in super::super)),
        );

        test_sublevel_visibility(
            &parse_quote!(pub(in self::asdf)),
            &parse_quote!(pub(in super::asdf)),
        );

        test_sublevel_visibility(
            &parse_quote!(pub(in crate::asdf)),
            &parse_quote!(pub(in crate::asdf)),
        );

        test_sublevel_visibility(
            &parse_quote!(pub(crate)),
            &parse_quote!(pub(crate)),
        );
    }
}

pub fn remove_generics_bounds(generics: &mut Generics) {
    for param in generics.params.iter_mut() {
        match param {
            syn::GenericParam::Lifetime(lifetime) => lifetime.bounds.clear(),
            syn::GenericParam::Type(ty) => ty.bounds.clear(),
            syn::GenericParam::Const(_) => {}
        }
    }
}
