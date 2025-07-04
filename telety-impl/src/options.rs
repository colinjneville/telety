use quote::{ToTokens, quote};
use syn::{
    Attribute, Expr, ExprLit, Ident, Lit, MetaNameValue, Path, Token, Visibility, parse::Parse,
    parse_quote, parse2, punctuated::Punctuated, spanned::Spanned as _,
};

use crate::visitor;

pub struct Options {
    pub module_path: Path,
    pub telety_path: Option<Path>,
    pub macro_ident: Option<Ident>,
    pub visibility: Option<Visibility>,
    pub proxy: Option<Path>,
}

impl Options {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut args = None;
        for attr in attrs {
            if attr.path().is_ident("telety") {
                #[allow(clippy::collapsible_if, reason = "separate mutating if for clarity")]
                if args
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
        directed_visit::visit_mut(
            &mut directed_visit::syn::direct::FullDefault,
            &mut visitor::Crateify::new(),
            &mut containing_path,
        );

        containing_path
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
        directed_visit::visit_mut(
            &mut directed_visit::syn::direct::FullDefault,
            &mut visitor::Decrateify::new(),
            &mut module_path,
        );

        let mut telety_path = None;
        let mut macro_ident = None;
        let mut visibility = None;
        let mut proxy = None;

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
                    } else if ident == "proxy" {
                        proxy = Some(value.parse()?);
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
            proxy,
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
            proxy,
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
        let proxy = proxy
            .as_ref()
            .map(ToTokens::to_token_stream)
            .as_ref()
            .map(ToString::to_string)
            .into_iter();

        quote!(
            #module_path
            #(, telety_path = #telety_path)*
            #(, macro_ident = #macro_ident)*
            #(, visibility = #visibility)*
            #(, proxy = #proxy)*
        )
        .to_tokens(tokens);
    }
}
