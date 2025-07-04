use proc_macro2::{Group, TokenStream};
use syn::{
    Token,
    parse::{Parse, ParseStream},
    parse2,
};
use telety_impl::find_and_replace::SingleToken;

struct FindAndReplaceArgs {
    needle: SingleToken,
    _comma0: Token![,],
    replacement: Group,
    _comma1: Token![,],
    haystack: TokenStream,
}

impl FindAndReplaceArgs {
    pub fn find_and_replace(self) -> TokenStream {
        let Self {
            needle,
            _comma0,
            replacement,
            _comma1,
            haystack,
        } = self;

        telety_impl::find_and_replace::find_and_replace(needle, replacement.stream(), haystack)
    }
}

impl Parse for FindAndReplaceArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            needle: input.parse()?,
            _comma0: input.parse()?,
            replacement: input.parse()?,
            _comma1: input.parse()?,
            haystack: input.parse()?,
        })
    }
}

pub(crate) fn find_and_replace(args: TokenStream) -> syn::Result<TokenStream> {
    let args: FindAndReplaceArgs = parse2(args)?;
    Ok(args.find_and_replace())
}

#[cfg(test)]
mod test {
    use quote::{ToTokens as _, quote};
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test() {
        let args: FindAndReplaceArgs = parse_quote!(
            $,
            [$dollar],
            macro_rules! my_macro {
                ($($tokens:tt)*) => {
                    $($tokens)*
                };
            }
        );

        let output = args.find_and_replace();

        assert_eq!(
            output.to_token_stream().to_string(),
            quote!(
                macro_rules! my_macro {
                    ($dollar ($dollar tokens:tt)*) => {
                        $dollar ($dollar tokens)*
                    };
                }
            )
            .to_string()
        );
    }
}
