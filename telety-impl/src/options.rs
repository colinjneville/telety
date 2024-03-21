use quote::ToTokens;
use syn::{parse::Parse, parse2, spanned::Spanned as _, visit_mut::VisitMut as _, Attribute, Path};

use crate::visitor;

pub struct Options {
    pub containing_path: Path,
}

impl Options {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = None;
        for attr in attrs {
            let mut segments_iter = attr.path().segments.iter();
            if let (Some(attr_name), None) = (segments_iter.next(), segments_iter.next()) {
                if attr_name.ident == "telety"
                    && args
                        .replace(parse2(attr.meta.require_list()?.tokens.clone())?)
                        .is_some()
                {
                    return Err(syn::Error::new(
                        attr.span(),
                        "Only one 'telety' attribute is allowed",
                    ));
                }
            }
        }

        args.ok_or_else(|| {
            syn::Error::new(
                attrs.first().span(),
                "'telety' attribute not found (aliasing the attribute is not supported)",
            )
        })
    }

    pub fn converted_containing_path(&self) -> Path {
        let mut containing_path = self.containing_path.clone();
        visitor::Crateify::new().visit_path_mut(&mut containing_path);
        
        containing_path
    }
}

impl Parse for Options {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut containing_path: Path = input.parse()?;
        visitor::Decrateify::new().visit_path_mut(&mut containing_path);

        Ok(Self { containing_path })
    }
}

impl ToTokens for Options {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            containing_path,
        } = self;

        containing_path.to_tokens(tokens);
    }
}
