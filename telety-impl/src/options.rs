use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::Parse, parse2, parse_quote, punctuated::Punctuated, spanned::Spanned as _,
    visit_mut::VisitMut as _, Attribute, Expr, ExprLit, Ident, Lit, MetaNameValue, Path, Token,
    Visibility,
};

use crate::visitor;

pub struct Options {
    pub module_path: Path,
    pub telety_path: Option<Path>,
    pub macro_ident: Option<Ident>,
    pub visibility: Option<Visibility>,
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
        let mut containing_path = self.module_path.clone();
        visitor::Crateify::new().visit_path_mut(&mut containing_path);

        containing_path
    }

    pub fn unique_ident(&self, ident: &Ident) -> Ident {
        let mut iter = self.module_path.segments.iter();
        let mut unique_ident = iter
            .next()
            .expect("Path must have at least one segment")
            .ident
            .clone();
        for segment in iter {
            let i = &segment.ident;
            unique_ident = format_ident!("{unique_ident}_{i}");
        }
        format_ident!("{unique_ident}_{ident}")
    }

    pub fn telety_path(&self) -> Path {
        self.telety_path
            .clone()
            .unwrap_or_else(|| parse_quote!(::telety))
    }
}

impl Parse for Options {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut module_path: Path = input.parse()?;
        visitor::Decrateify::new().visit_path_mut(&mut module_path);

        let mut telety_path = None;
        let mut macro_ident = None;
        let mut visibility = None;

        if let Some(_comma) = input.parse::<Option<Token![,]>>()? {
            let named_args: Punctuated<MetaNameValue, Token![,]> =
                Punctuated::parse_terminated(input)?;
            for named_arg in named_args {
                if let Some(ident) = named_arg.path.get_ident() {
                    let Expr::Lit(ExprLit {
                        lit: Lit::Str(value),
                        ..
                    }) = &named_arg.value
                    else {
                        return Err(syn::Error::new(
                            named_arg.value.span(),
                            "Expected a string literal",
                        ));
                    };

                    if ident == "telety_path" {
                        telety_path = Some(value.parse()?);
                    } else if ident == "macro_ident" {
                        macro_ident = Some(value.parse()?);
                    } else if ident == "visibility" {
                        visibility = Some(value.parse()?);
                    } else {
                        return Err(syn::Error::new(
                            named_arg.path.span(),
                            "Invalid parameter name",
                        ));
                    }
                } else {
                    return Err(syn::Error::new(
                        named_arg.path.span(),
                        "Expected a parameter name",
                    ));
                }
            }
        }

        Ok(Self {
            module_path,
            telety_path,
            macro_ident,
            visibility,
        })
    }
}

impl ToTokens for Options {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            module_path,
            telety_path,
            macro_ident,
            visibility,
        } = self;

        // Convert to string literals
        let telety_path = telety_path
            .as_ref()
            .map(ToTokens::to_token_stream)
            .as_ref()
            .map(ToString::to_string)
            .into_iter();
        let macro_ident = macro_ident
            .as_ref()
            .map(ToTokens::to_token_stream)
            .as_ref()
            .map(ToString::to_string)
            .into_iter();
        let visibility = visibility
            .as_ref()
            .map(ToTokens::to_token_stream)
            .as_ref()
            .map(ToString::to_string)
            .into_iter();

        // module_path.to_tokens(tokens);
        // <Token![,]>::default().to_tokens(tokens);

        quote!(
            #module_path,
            #(telety_path = #telety_path,)*
            #(macro_ident = #macro_ident,)*
            #(visibility = #visibility,)*
        )
        .to_tokens(tokens);
        // if let Some(telety_path) = telety_path {
        //     quote!(telety_path = stringify!(#telety_path),).to_tokens(tokens);
        // }
        // if let Some(macro_ident) = macro_ident {
        //     quote!(telety_path = stringify!(#telety_path),).to_tokens(tokens);
        // }
        // if let Some(visibility) = visibility {
        //     visibility.to_tokens(tokens);
        //     <Token![,]>::default().to_tokens(tokens);
        // }
    }
}
