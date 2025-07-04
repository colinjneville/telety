//! Based on <https://crates.io/crates/macro_find_and_replace>
use proc_macro2::{Group, Literal, Punct, TokenStream, TokenTree};
use quote::{ToTokens, TokenStreamExt};
use syn::{
    Ident,
    parse::{Parse, ParseStream},
};

pub enum SingleToken {
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl From<Ident> for SingleToken {
    fn from(value: Ident) -> Self {
        Self::Ident(value)
    }
}

impl From<Punct> for SingleToken {
    fn from(value: Punct) -> Self {
        Self::Punct(value)
    }
}

impl From<Literal> for SingleToken {
    fn from(value: Literal) -> Self {
        Self::Literal(value)
    }
}

impl Parse for SingleToken {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let tt: TokenTree = input.parse()?;
        match tt {
            TokenTree::Group(g) => Err(syn::Error::new(
                g.span(),
                "Only single tokens are allowed as needles",
            )),
            TokenTree::Ident(i) => Ok(Self::Ident(i)),
            TokenTree::Punct(p) => Ok(Self::Punct(p)),
            TokenTree::Literal(l) => Ok(Self::Literal(l)),
        }
    }
}

impl ToTokens for SingleToken {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SingleToken::Ident(i) => i.to_tokens(tokens),
            SingleToken::Punct(p) => p.to_tokens(tokens),
            SingleToken::Literal(l) => l.to_tokens(tokens),
        }
    }
}

impl PartialEq<TokenTree> for SingleToken {
    fn eq(&self, other: &TokenTree) -> bool {
        match (self, other) {
            (SingleToken::Ident(a), TokenTree::Ident(b)) if a == b => true,
            (SingleToken::Punct(a), TokenTree::Punct(b)) if a.as_char() == b.as_char() => true,
            (SingleToken::Literal(a), TokenTree::Literal(b)) if a.to_string() == b.to_string() => {
                true
            }
            _ => false,
        }
    }
}

pub fn find_and_replace(
    needle: impl Into<SingleToken>,
    replacement: TokenStream,
    haystack: TokenStream,
) -> TokenStream {
    fn far(needle: &SingleToken, replacement: &TokenStream, haystack: TokenStream) -> TokenStream {
        let mut output = TokenStream::new();

        for tt in haystack {
            let tt = match tt {
                TokenTree::Group(g) => TokenTree::Group(Group::new(
                    g.delimiter(),
                    far(needle, replacement, g.stream()),
                )),
                tt if needle == &tt => {
                    output.extend(replacement.clone());
                    continue;
                }
                tt => tt,
            };

            output.append(tt);
        }

        output
    }

    let needle = needle.into();

    far(&needle, &replacement, haystack)
}
