use proc_macro2::Ident;
use syn::{
    parse_quote, punctuated::Punctuated, Expr, ExprPath, GenericArgument, GenericParam, Generics, Path, PathArguments, PathSegment, Token, Type, TypePath, VisRestricted, Visibility
};

pub(crate) fn sublevel_visibility(visibility: &Visibility) -> Visibility {
    match visibility {
        Visibility::Public(_) => visibility.clone(),
        Visibility::Restricted(restricted) => {
            let VisRestricted { 
                in_token, 
                path,
                .. 
            } = restricted;

            let first_ident = &path.segments.first().unwrap().ident;

            if in_token.is_some() {
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
            } else if first_ident == "crate" {
                visibility.clone()
            } else if first_ident == "self" {
                parse_quote!(pub(super))
            } else if first_ident == "super" {
                parse_quote!(pub(in super::super))
            } else {
                unreachable!("Invalid visibility keyword '{first_ident}'")
            }
        },
        Visibility::Inherited => parse_quote!(pub(super)),
    }
}

#[cfg(test)]
mod test_sublevel_visibility {
    use quote::{quote, ToTokens as _};
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test() {
        let after = sublevel_visibility(&parse_quote!(pub(in self)));
        assert_eq!(after.to_token_stream().to_string(), quote!(pub(in super)).to_string());

        let after = sublevel_visibility(&parse_quote!(pub(self)));
        assert_eq!(after.to_token_stream().to_string(), quote!(pub(super)).to_string());

        let after = sublevel_visibility(&parse_quote!(pub(super)));
        assert_eq!(after.to_token_stream().to_string(), quote!(pub(in super::super)).to_string());

        let after = sublevel_visibility(&parse_quote!(pub(in self::asdf)));
        assert_eq!(after.to_token_stream().to_string(), quote!(pub(in super::asdf)).to_string());
    }
}


pub(crate) fn generic_params_to_arguments(params: &Generics) -> Punctuated<GenericArgument, Token![,]> {
    let mut args = Punctuated::new();
    for pair in params.params.pairs() {
        args.push_value(generic_param_to_argument(pair.value()));
        if let Some(&comma) = pair.punct() {
            args.push_punct(*comma);
        }
    }
    args
}

pub(crate) fn generic_param_to_argument(param: &GenericParam) -> GenericArgument {
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

pub fn remove_generics_bounds(generics: &mut Generics) {
    for param in generics.params.iter_mut() {
        match param {
            syn::GenericParam::Lifetime(lifetime) => lifetime.bounds.clear(),
            syn::GenericParam::Type(ty) => ty.bounds.clear(),
            syn::GenericParam::Const(_) => {}
        }
    }
}

pub fn ident_to_type_path(ident: Ident) -> TypePath {
    let mut segments = Punctuated::new();
    segments.push_value(PathSegment {
        ident,
        arguments: PathArguments::None,
    });

    TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments,
        },
    }
}
