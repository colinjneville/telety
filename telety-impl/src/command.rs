use std::borrow::Cow;

use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{Ident, LitInt, Path, parse_quote, parse_quote_spanned, spanned::Spanned as _};

use crate::{Telety, find_and_replace::SingleToken};

pub(crate) type GenerateMacroTokens = fn(tele_ty: &Telety) -> Option<TokenStream>;

/// Used to invoke the telety-generated macro in a manageable way.
pub struct Command {
    version: usize,
    keyword: &'static str,
    generate_macro_tokens: GenerateMacroTokens,
}

impl Command {
    pub(crate) const fn new(
        version: usize,
        keyword: &'static str,
        generate_macro_tokens: GenerateMacroTokens,
    ) -> Self {
        Self {
            version,
            keyword,
            generate_macro_tokens,
        }
    }

    pub(crate) const fn version(&self) -> usize {
        self.version
    }

    fn version_lit(&self, span: Option<Span>) -> LitInt {
        let span = span.unwrap_or(Span::call_site());
        LitInt::new(&self.version().to_string(), span)
    }

    fn keyword(&self, span: Option<Span>) -> Ident {
        let span = span.unwrap_or(Span::call_site());
        Ident::new(self.keyword, span)
    }

    #[doc(hidden)]
    pub fn generate_macro_arm(&self, ty: &Telety) -> syn::Result<Option<TokenStream>> {
        if let Some(implementation) = (self.generate_macro_tokens)(ty) {
            let span = ty.item().span();

            let ParameterIdents {
                args,
                needle,
                haystack,
            } = ParameterIdents::new(span);

            let keyword = self.keyword(Some(span));
            let version = self.version_lit(Some(span));
            Ok(Some(quote_spanned! { span =>
                (#version, #keyword $( ( $($#args:tt)* ) )?, $#needle:tt, $($#haystack:tt)*) => {
                    #implementation
                };
            }))
        } else {
            Ok(None)
        }
    }

    /// Creates a macro invocation to use this command with the telety-generated macro at `macro_path`.  
    /// The output of the command will be inserted into `haystack` at each instance of `needle`.
    /// `macro_path` must point to a valid telety-generated macro, otherwise a compile error will occur.  
    /// To support future [Command]s, `args` are passed to the command invocation, but they are not currently used.  
    /// ## Example
    /// ```rust,ignore
    /// # use syn::parse2;
    /// #[proc_macro]
    /// pub fn my_public_macro(tokens: TokenStream) -> TokenStream {
    ///     // ...
    ///     let my_needle: TokenTree = format_ident!("__my_needle__").into();
    ///     v1::UNIQUE_IDENT.apply(
    ///         &parse_quote!(crate::MyTeletyObj),
    ///         &my_needle,
    ///         quote! {
    ///             my_crate::my_macro_implementation!(#my_needle);
    ///         },
    ///         None,
    ///     )
    /// }
    /// #[doc(hidden)]
    /// #[proc_macro]
    /// pub fn my_macro_implementation(tokens: TokenStream) -> TokenStream {
    ///     let ident: Ident = parse2(tokens);
    ///     // ...
    /// }
    /// ```
    pub fn apply(
        &'static self,
        macro_path: Path,
        needle: impl Into<SingleToken>,
        haystack: impl ToTokens,
    ) -> Apply {
        Apply::new(
            self,
            macro_path,
            needle.into(),
            haystack.into_token_stream(),
        )
    }
}

pub(crate) struct ParameterIdents {
    pub args: Ident,
    pub needle: Ident,
    pub haystack: Ident,
}

impl ParameterIdents {
    pub fn new(span: Span) -> Self {
        Self {
            args: Ident::new("args", span),
            needle: Ident::new("needle", span),
            haystack: Ident::new("haystack", span),
        }
    }
}

/// Creates the [TokenStream] for the [Command] using the given arguments.  
/// Can be interpolated directly in a [quote!] macro.
pub struct Apply {
    command: &'static Command,
    macro_path: Path,
    needle: SingleToken,
    haystack: TokenStream,
    args: Option<TokenStream>,
    fallback: Option<TokenStream>,
    telety_path: Option<Path>,
    unique_macro_ident: Option<Ident>,
}

impl Apply {
    fn new(
        command: &'static Command,
        macro_path: Path,
        needle: SingleToken,
        haystack: TokenStream,
    ) -> Self {
        Self {
            command,
            macro_path,
            needle,
            haystack,
            args: None,
            fallback: None,
            telety_path: None,
            unique_macro_ident: None,
        }
    }

    #[doc(hidden)]
    /// Pass arguments to the command invocation.  
    /// Note: no commands currently use any arguments
    pub fn with_arguments(mut self, arguments: impl ToTokens) -> Self {
        self.args.replace(arguments.into_token_stream());
        self
    }

    /// If `macro_path` does not contain a macro, instead expand to the `fallback` tokens.  
    /// By default, the command or `fallback` will be expanded inside an anonymous block,
    /// so any items cannot not be referenced from outside. Use [Apply::with_macro_forwarding]
    /// to expand the output directly in this scope.
    pub fn with_fallback(mut self, fallback: impl ToTokens) -> Self {
        self.fallback.replace(fallback.into_token_stream());
        self
    }

    /// Specify the location of the telety crate.  
    /// This is only required if telety is not located at the default path `::telety`
    /// and [Apply::with_fallback] is used.
    pub fn with_telety_path(mut self, telety_path: Path) -> Self {
        self.telety_path.replace(telety_path);
        self
    }

    /// If a fallback is set, forward the final haystack/fallback tokens through a macro
    /// so that they are evaluated without additional block scopes.  
    /// This is usually required if you a creating a named item (such as a `struct` or `enum`), but
    /// not for `impls`.
    /// `unique_macro_ident` must be unique within the crate.  
    /// This has no effect if [Apply::with_fallback] is not used.
    pub fn with_macro_forwarding(mut self, unique_macro_ident: Ident) -> Self {
        self.unique_macro_ident.replace(unique_macro_ident);
        self
    }
}

impl ToTokens for Apply {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let macro_path = &self.macro_path;
        let needle = &self.needle;
        let mut haystack = self.haystack.to_token_stream();
        let args = self.args.as_ref().map(|ts| quote!((#ts)));

        let span = self.haystack.span();
        let version = self.command.version_lit(Some(span));
        let keyword = self.command.keyword(Some(span));

        let textual_macro_ident: Ident = format_ident!(
            "my_macro_{}",
            self.unique_macro_ident
                .as_ref()
                .unwrap_or(&format_ident!("a"))
        );

        let mut fallback = self.fallback.as_ref().map(|f| f.to_token_stream());

        if let Some(fallback) = fallback.as_mut()
            && let Some(unique_macro_ident) = &self.unique_macro_ident
        {
            let macro_wrapper = |contents: &TokenStream| {
                // Replace `$` in the original content with `$dollar dollar`
                // because we have 2 extra layers of macro rules indirection
                let contents = crate::find_and_replace::find_and_replace(
                    Punct::new('$', Spacing::Alone),
                    quote!($dollar dollar),
                    contents.into_token_stream(),
                );

                quote_spanned! { span =>
                    // Export a macro...
                    #[doc(hidden)]
                    #[macro_export]
                    macro_rules! #unique_macro_ident {
                        ($dollar:tt) => {
                            // which defines a macro, ...
                            macro_rules! #textual_macro_ident {
                                ($dollar dollar:tt) => {
                                    // which expands to our actual contents
                                    #contents
                                };
                            }
                        };
                    }
                }
            };

            *fallback = macro_wrapper(fallback);
            haystack = macro_wrapper(&haystack);
        }

        let mut output = parse_quote_spanned! { span =>
            #macro_path! { #version, #keyword #args, #needle, #haystack }
        };

        if let Some(fallback) = fallback {
            let telety_path = self
                .telety_path
                .as_ref()
                .map(Cow::Borrowed)
                .unwrap_or_else(|| Cow::Owned(parse_quote!(::telety)));

            output = parse_quote_spanned! { span =>
                #telety_path::util::try_invoke! {
                    #output
                    #fallback
                }
            };

            if let Some(unique_macro_ident) = &self.unique_macro_ident {
                let temp_ident = format_ident!("_{unique_macro_ident}");

                // In order to invoke the macro from its export at the crate root, we need this trick:
                // We must glob import the crate root, and invoke the macro without a path. To avoid polluting the current module,
                // we make a sub-module, and invoke within there. Since we need to expand the caller's `haystack` in the main module,
                // this macro is a layer of indirection which defines another macro. Name resolution does not like the main module
                // peeking into our sub-module, so we will invoke our final macro using textual scope instead of module scope.
                // #[macro_use] allows the macro to remain in textual scope after the sub-module, so we can invoke it there.
                let import = quote_spanned! { span =>
                    #[macro_use]
                    #[doc(hidden)]
                    mod #temp_ident {
                        pub(super) use crate::*;

                        #unique_macro_ident!($);
                    }
                };

                let invoke = quote_spanned! { span =>
                    #textual_macro_ident! { $ }
                };

                output = quote_spanned! { span =>
                    #output
                    #import
                    #invoke
                };
            }
        }

        output.to_tokens(tokens);
    }
}
