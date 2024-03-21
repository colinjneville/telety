use proc_macro2::{Punct, Spacing, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use syn::{parse_quote, parse_quote_spanned, spanned::Spanned as _, Ident, Item, ItemMacro, LitInt, Path};

use crate::{find_and_replace::SingleToken, Alias, Telety};

pub(crate) type GenerateMacroTokens = fn(tele_ty: &Telety) -> Option<TokenStream>;
pub(crate) type GenerateOverrideMacroTokens = fn(alias: Alias) -> Option<TokenStream>;

/// Used to invoke the telety-generated macro in a manageable way.
pub struct Command {
    version: usize,
    keyword: &'static str,
    generate_macro_tokens: GenerateMacroTokens,
    generate_override_macro_tokens: Option<GenerateOverrideMacroTokens>,
}

impl Command {
    pub(crate) const fn new(
        version: usize,
        keyword: &'static str,
        generate_macro_tokens: GenerateMacroTokens,
        generate_override_macro_tokens: Option<GenerateOverrideMacroTokens>,
    ) -> Self {
        Self {
            version,
            keyword,
            generate_macro_tokens,
            generate_override_macro_tokens,
        }
    }

    pub(crate) const fn version(&self) -> usize {
        self.version
    }

    pub(crate) fn version_lit(&self, span: Option<Span>) -> LitInt {
        let span = span.unwrap_or(Span::call_site());
        LitInt::new(&self.version().to_string(), span)
    }

    pub(crate) fn keyword(&self, span: Option<Span>) -> Ident {
        let span = span.unwrap_or(Span::call_site());
        Ident::new(self.keyword, span)
    }

    pub(crate) fn generate_macro_arm(&self, ty: &Telety) -> syn::Result<Option<TokenStream>> {
        self.generate_macro_arm_internal((self.generate_macro_tokens)(ty), ty.item().span())
    }

    #[allow(dead_code)]
    pub(crate) fn generate_override_macro_arm(
        &self,
        alias: Alias,
    ) -> syn::Result<Option<TokenStream>> {
        if let Some(generate_override_macro_tokens) = &self.generate_override_macro_tokens {
            self.generate_macro_arm_internal(
                generate_override_macro_tokens(alias),
                alias.aliased_type().span(),
            )
        } else {
            Ok(None)
        }
    }

    fn generate_macro_arm_internal(
        &self,
        implementation: Option<TokenStream>,
        span: Span,
    ) -> syn::Result<Option<TokenStream>> {
        if let Some(implementation) = implementation {
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
        &self,
        macro_path: &Path,
        needle: impl Into<SingleToken>,
        haystack: impl ToTokens,
        args: Option<TokenStream>,
    ) -> ItemMacro {
        let needle = needle.into();

        let span = haystack.span();
        let version = self.version_lit(Some(span));
        let keyword = self.keyword(Some(span));

        let args = args.map(|ts| quote!((#ts)));

        parse_quote_spanned! { span =>
            #macro_path!(#version, #keyword #args, #needle, #haystack);
        }
    }

    /// Similar to [Command::apply], except if `macro_path` does not include a macro, `macro_fallback` will be used instead.  
    /// Note that macro_path must still be valid path of some sort (i.e. a type or value), otherwise compilation will fail.  
    /// Additionally, for name resolution to succeed, `macro_path` must start with a qualifier (e.g. `::`, `self::`, `crate::`, ...).
    /// If you see the error "import resolution is stuck, try simplifying macro imports", you are probably missing the qualifier.  
    /// Finally, the output will be placed inside a block, which means any items defined inside cannot be easily referenced elsewhere.
    /// The primary use-case is to create `impl`s.
    pub fn apply_or(
        &self,
        macro_path: &Path,
        needle: impl Into<SingleToken>,
        haystack: impl ToTokens,
        args: Option<TokenStream>,
        macro_fallback: &Path,
    ) -> ItemMacro {
        let needle = needle.into();

        let span = haystack.span();
        let version = self.version_lit(Some(span));
        let keyword = self.keyword(Some(span));

        let args = args.map(|ts| quote!((#ts)));

        parse_quote_spanned! { span =>
            ::telety::util::macro_fallback!(
                #macro_path,
                #macro_fallback,
                #version, #keyword #args, #needle, #haystack
            );
        }
    }

    // TODO apply_exported_or

    /// Similar to [Command::apply_or], except `haystack` is expanded in the current module or block if `macro_path` is a macro, 
    /// otherwise, a noop macro is expanded instead.
    /// `haystack` is forwarded through a `macro_rules!` macro, but `$` tokens within `haystack` will be
    /// automatically converted to work within the macro. 
    /// `unique_macro_ident` must be an identifier unique to the crate, as the forwarding macro must be `#[macro_export]`. 
    pub fn apply_exported_or_noop(
        &self,
        macro_path: &Path,
        needle: impl Into<SingleToken>,
        haystack: impl ToTokens,
        args: Option<TokenStream>,
        unique_macro_ident: &Ident,
    ) -> Vec<Item> {
        let needle = needle.into();
        
        let span = haystack.span();        

        let textual_macro_ident: Ident = format_ident!("my_macro_{unique_macro_ident}");

        // Replace `$` in the original haystack with `$dollar dollar`
        // because we have 2 extra layers of macro rules indirection
        let haystack = crate::find_and_replace::find_and_replace(
            Punct::new('$', Spacing::Alone), 
            quote!($dollar dollar), 
            haystack.into_token_stream());

        let haystack = quote_spanned! { span =>
            // Export a macro...
            #[doc(hidden)]
            #[macro_export]
            macro_rules! #unique_macro_ident {
                ($dollar:tt) => {
                    // which defines a macro, ...
                    macro_rules! #textual_macro_ident {
                        ($dollar dollar:tt) => {
                            // which expands to `haystack`
                            #haystack
                        };
                    }
                };
            }
        };

        // Our fallback exports a macro, which defines a macro, which expands to nothing
        let make_noop = parse_quote_spanned! { span => 
            macro_rules! make_noop {
                ($($tokens:tt)*) => { 
                    ::telety::__private::make_noop!(#unique_macro_ident, #textual_macro_ident);
                };
            }
        };

        let make_noop_ident = format_ident!("make_noop_{unique_macro_ident}");

        // Put the first macro in module scope so it works with `apply_or`
        let use_noop = parse_quote_spanned! { span => 
            use make_noop as #make_noop_ident;
        };

        let apply_or = Item::Macro(self.apply_or(
            macro_path,
            needle,
            haystack,
            args,
            &parse_quote!(self::#make_noop_ident),
        ));
        
        let temp_ident = format_ident!("_{unique_macro_ident}");

        // In order to invoke the macro from its export at the crate root, we need this trick:
        // We must glob import the crate root, and invoke the macro without a path. To avoid polluting the current module,
        // we make a sub-module, and invoke within there. Since we need to expand the caller's `haystack` in the main module,
        // this macro is a layer of indirection which defines another macro. Name resolution does not like the main module 
        // peeking into our sub-module, so we will invoke our final macro using textual scope instead of module scope.  
        // #[macro_use] allows the macro to remain in textual scope after the sub-module, so we can invoke it there.
        let import = parse_quote_spanned! { span => 
            #[macro_use]
            #[doc(hidden)]
            mod #temp_ident {
                pub(super) use crate::*;
                
                #unique_macro_ident!($);
            }
        };

        let invoke = parse_quote_spanned! { span => 
            #textual_macro_ident!($);
        };

        vec![
            make_noop,
            use_noop,
            apply_or,
            import,
            invoke,
        ]
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
