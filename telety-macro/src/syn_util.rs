use proc_macro2::Ident;
use syn::{parse_quote, Generics, VisRestricted, Visibility};

pub(crate) fn sublevel_visibility(visibility: &Visibility) -> Visibility {
    match visibility {
        Visibility::Public(_) => visibility.clone(),
        Visibility::Restricted(restricted) => {
            let VisRestricted { in_token, path, .. } = restricted;

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
        }
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
        assert_eq!(
            after.to_token_stream().to_string(),
            quote!(pub(in super)).to_string()
        );

        let after = sublevel_visibility(&parse_quote!(pub(self)));
        assert_eq!(
            after.to_token_stream().to_string(),
            quote!(pub(super)).to_string()
        );

        let after = sublevel_visibility(&parse_quote!(pub(super)));
        assert_eq!(
            after.to_token_stream().to_string(),
            quote!(pub(in super::super)).to_string()
        );

        let after = sublevel_visibility(&parse_quote!(pub(in self::asdf)));
        assert_eq!(
            after.to_token_stream().to_string(),
            quote!(pub(in super::asdf)).to_string()
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
