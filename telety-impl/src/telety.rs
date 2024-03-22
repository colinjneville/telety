use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, TokenStreamExt};
use syn::{
    parse2, parse_quote, parse_quote_spanned, spanned::Spanned, visit::Visit, visit_mut::VisitMut as _, AngleBracketedGenericArguments, Attribute, GenericArgument, Ident, Item, LitInt, Path, PathArguments, PathSegment, Type, TypePath, Visibility
};

use crate::{
    alias,
    item_data::{ItemData as _, Namespaces},
    syn_util, version, visitor, Alias, Options,
};

/// Wraps an [Item] which has the `#[telety]` attribute to provide additional information
/// allowing it to be reflected outside its original context.
pub struct Telety<'item> {
    options: Options,
    item: &'item Item,
    alias_map: alias::Map,
    unique_ident: Ident,
}

impl<'item> Telety<'item> {
    #[doc(hidden)]
    pub fn prepend_attribute(item: &mut Item, attr_args: TokenStream) -> syn::Result<()> {
        let span = attr_args.span();
        let mut options: Options = parse2(attr_args)?;
        visitor::Decrateify::new().visit_path_mut(&mut options.containing_path);
        
        let full_attr: Attribute = parse_quote_spanned!(span => #[telety(#options)]);
        item.attrs_mut().insert(0, full_attr);
        Ok(())
    }

    /// Generate telety information for the [Item].  
    /// The item must have a proper `#[telety(...)]` attribute.  
    /// Usually this item will come from the telety-generated macro with the same name as the item.
    pub fn new(item: &'item Item) -> syn::Result<Self> {
        let options = Options::from_attrs(item.attrs())?;

        let ident = item.ident().ok_or(syn::Error::new(
            item.span(),
            "Only named items are supported",
        ))?;

        if ident.namespaces & Namespaces::Macro != Namespaces::None {
            return Err(syn::Error::new(
                item.span(),
                "Cannot be applied to items in the macro namespace",
            ));
        }

        let parameters = item.generics().cloned().unwrap_or_default();

        let self_type: Type = {
            let arguments = PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Default::default(),
                args: syn_util::generic_params_to_arguments(&parameters),
                gt_token: Default::default(),
            });
            // Use the global path to alert user if the containing_path is incorrect
            let mut path = options.converted_containing_path();
            path.segments.push(PathSegment {
                ident: item.ident().unwrap().ident.clone(),
                arguments,
            });

            Type::Path(TypePath { qself: None, path })
        };

        let unique_ident = Self::make_unique_ident(ident.ident, &options);

        let alias_group =
            alias::Group::new(options.converted_containing_path(), format_ident!("__{}", unique_ident));

        let mut alias_map = alias::Map::new(alias_group, parameters.clone(), self_type);

        // Identify all unique non-type parameter types and give them an index
        // (based on order of appearance), stored in our map
        let mut identify_visitor = visitor::IdentifyAliases::new(&mut alias_map);
        identify_visitor.visit_item(item);

        Ok(Self {
            options,
            item,
            alias_map,
            unique_ident,
        })
    }

    /// Provides the [alias::Map] for this item, which describes the mapping
    /// of types appearing in the item to the aliases created for them.
    pub fn alias_map(&self) -> &alias::Map {
        &self.alias_map
    }

    /// Create a visitor which substitutes generic parameters as if this type were monomorphized
    /// with the provided generic arguments.  
    /// For example, if we have a type:
    /// ```rust,ignore
    /// #[telety(crate)]
    /// struct S<T, U, V = T>(T, U, V);
    /// ```
    /// and provided the arguments `[i32, u64]`,
    /// the visitor would replace types `T` with `i32`,
    /// `U` with `u64`, and `V` with `i32`.  
    /// See [syn::visit_mut].
    pub fn generics_visitor<'a>(
        &self,
        generic_arguments: impl IntoIterator<Item = &'a GenericArgument>,
    ) -> syn::Result<visitor::ApplyGenericArguments> {
        let Some(parameters) = self.item.generics() else {
            return Err(syn::Error::new(
                self.item.span(),
                "Item kind does not have generic parameters",
            ));
        };

        visitor::ApplyGenericArguments::new(parameters, generic_arguments)
    }

    #[doc(hidden)]
    pub fn appendix(&self) -> syn::Result<TokenStream> {
        // We now need to re-remove the telety macro
        // so that we aren't infinitely expanding the macro repeatedly
        let mut item = self.item.clone();
        let telety_attr = item.attrs_mut().remove(0);
        assert!(
            telety_attr.path().is_ident("telety"),
            "expected telety as the first attribute macro"
        );

        let ident = self.item.ident().expect("Item must have an Ident").ident;

        let textual_ident = &self.unique_ident;

        let vis = self.item.vis().expect("Item must have a Visibility");

        let alias_mod = self.generate_alias_mod()?;

        let item_macro = self.generate_macro(textual_ident)?;

        let span = Span::call_site();

        Ok(quote_spanned! { span =>
            #item

            #item_macro

            #vis use #textual_ident as #ident;

            #alias_mod
        })
    }

    fn make_unique_ident(ident: &Ident, options: &Options) -> Ident {
        let mut iter = options.containing_path.segments.iter();
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

    /// The [Item] this describes
    pub fn item(&self) -> &Item {
        self.item
    }

    /// The [Path] to the item, using the crate name, not the `crate::` qualifier,
    /// and no arguments on the item.
    pub fn path(&self) -> Path {
        let mut path = self.options.containing_path.clone();
        if let Some(ident) = self.item.ident() {
            path.segments.push(PathSegment {
                ident: ident.ident.clone(),
                arguments: PathArguments::None,
            });
        }
        path
    }

    /// The [Attribute]s on the [Item]
    pub fn attributes(&self) -> &[Attribute] {
        self.item.attrs()
    }

    /// The [Path] of the module containing this [Item].
    /// Provided by argument to the telety attribute.
    pub fn containing_mod_path(&self) -> Path {
        self.options.converted_containing_path()
    }

    /// A (reasonably) unique [struct@Ident] for this item.  
    pub fn unique_ident(&self) -> Ident {
        self.unique_ident.clone()
    }

    pub(crate) fn vis(&self) -> Option<&Visibility> {
        self.item.vis()
    }

    pub(crate) fn macro_export(&self) -> Option<Attribute> {
        match self.vis()? {
            Visibility::Public(vis_pub) => {
                let span = vis_pub.span;
                Some(parse_quote_spanned!(span => #[macro_export]))
            }
            Visibility::Restricted(_vis_restricted) => None,
            Visibility::Inherited => None,
        }
    }

    pub(crate) fn generate_macro(&self, ident: &Ident) -> syn::Result<TokenStream> {
        let span = self.item.span();

        let mut arms = TokenStream::new();
        for &(version, commands) in version::VERSIONS {
            for command in commands {
                let arm = command.generate_macro_arm(self)?;
                arms.append_all(arm);
            }

            let version = LitInt::new(&version.to_string(), span);
            arms.append_all(quote_spanned! { span =>
                (#version $command:ident $($tokens:tt)*) => {
                    compile_error!(concat!("No command '",  stringify!($command), "' for version ", stringify!(#version)));
                };
                (#version $($tokens:tt)*) => {
                    compile_error!("Expected a command");
                };
            });
        }
        arms.append_all(quote_spanned! { span =>
            ($version:literal $($tokens:tt)*) => {
                compile_error!(concat!("Unsupported version ", stringify!($version)));
            };
            ($($tokens:tt)*) => {
                compile_error!("Version not provided");
            };
        });
        
        let macro_export = self.macro_export();

        Ok(quote_spanned! { span =>
            #macro_export
            macro_rules! #ident {
                #arms
            }
        })
    }

    pub(crate) fn generate_alias_mod(&self) -> syn::Result<TokenStream> {
        let vis = self.item.vis().ok_or(syn::Error::new(
            self.item.span(),
            "Not supported for this type of item",
        ))?;

        // items inside our module need to convert blank vis to `pub(super)`, etc.
        let super_vis = syn_util::sublevel_visibility(vis);

        let mut aliases = TokenStream::new();
        for alias in self.alias_map.iter() {
            aliases.append_all(self.generate_alias(&super_vis, alias));
        }

        let mod_ident = self.alias_map.group().ident();

        let exact_alias_mod = self.generate_exact_alias_mod(&super_vis)?;

        let span = Span::call_site();
        Ok(quote_spanned! { span =>
            #[doc(hidden)]
            #[allow(dead_code)]
            #[allow(unused_macros)]
            #[allow(unused_imports)]
            #vis mod #mod_ident {
                #exact_alias_mod

                #aliases
            }
        })
    }

    fn generate_exact_alias_mod(&self, vis: &Visibility) -> syn::Result<TokenStream> {
        let span = Span::call_site();

        let super_vis = syn_util::sublevel_visibility(vis);

        let exact_aliases: syn::Result<Vec<TokenStream>> = self.alias_map().iter().map(|a| self.generate_exact_alias(&super_vis, a)).collect();
        let exact_aliases = exact_aliases?;

        Ok(quote_spanned! { span =>
            mod exact {
                #super_vis use super::super::*;

                #(#exact_aliases)*
            }
        })
    }

    fn generate_exact_alias(&self, vis: &Visibility, alias: Alias) -> syn::Result<TokenStream> {
        let aliased_type = alias.aliased_type();
        let span = aliased_type.span();

        let alias::Definition {
            ident,
            internal_ident,
            ..
        } = alias.index().definition(self.unique_ident.clone());

        let item_use = if let (Type::Path(type_path), false) = (aliased_type, alias.is_generic_parameter_type()) {
            let mut aliased_type_no_generics = type_path.clone();
            aliased_type_no_generics
                        .path.segments.last_mut().expect("Path should have at least one segment").arguments = PathArguments::None;

            quote_spanned! { span =>
                // Create a fixed alias for our submodule to reference
                #vis use #aliased_type_no_generics as #ident;
            }
        } else {
            quote!()
        };

        let mut parameters = alias.parameters().clone();
        // `type` aliases should not have bounds
        syn_util::remove_generics_bounds(&mut parameters);

        Ok(quote_spanned! { span =>
            #item_use
            #vis type #internal_ident #parameters = #aliased_type;
        })
    }

    fn generate_alias(&self, vis: &Visibility, alias: Alias) -> syn::Result<TokenStream> {
        let aliased_type = alias.aliased_type();
        let span = aliased_type.span();

        let alias::Definition {
            ident,
            internal_ident,
            macro_maker_ident,
            alias_unique_ident,
            submodule_ident,
        } = alias.index().definition(self.unique_ident.clone());

        let macro_vis = self.macro_export();

        let super_vis = syn_util::sublevel_visibility(vis);
        let super_super_vis = syn_util::sublevel_visibility(&super_vis);
        
        let mut parameters = alias.parameters().clone();
        // `type` aliases should not have bounds
        syn_util::remove_generics_bounds(&mut parameters);

        // Only non-type parameter path types can have an 'embedded' macro 
        if let (Type::Path(type_path), false) = (aliased_type, alias.is_generic_parameter_type()) {
            let mut aliased_type_no_generics = type_path.clone();
            aliased_type_no_generics
                        .path.segments.last_mut().expect("Path should have at least one segment").arguments = PathArguments::None;
            
            // We are allowed to export items *in* private modules at their original visibility. e.g.
            // ```rust
            // use my_crate;
            // pub use my_crate::MyPubStruct;
            // ```
            // But we can't re-export items themselves at greater visibility than our import, even if their
            // original visibility is greater. This is invalid:
            // ```rust
            // use my_crate::MyPubStruct;
            // pub use MyPubStruct as MyPubReexport;
            // ```
            // We can work around this, but it requires 2 extra exported macros per item, so prefer the simple
            // way if we have a multi-segment path as our type.
            if type_path.path.segments.len() == 1 {
                let needle: Ident = parse_quote!(__needle);

                let exported_apply = version::v0::PATH.apply_exported_or_noop(
                    &parse_quote!(self::exact::#ident), 
                    needle.clone(), 
                    quote! {
                        #[doc(hidden)]
                        #macro_vis
                        macro_rules! #alias_unique_ident {
                            ($($tokens:tt)*) => {
                                ::telety::__private::crateify! {
                                    #needle!($($tokens)*);
                                };
                            };
                        }
                    }, 
                    None, 
                    &macro_maker_ident
                );

                Ok(quote! {
                    // Create an exported macro. If the type's macro existed, it is a forwarder.
                    // If it did not exist, it is a noop
                    #(#exported_apply)*

                    // Create an alias for just the type
                    #vis type #alias_unique_ident #parameters = self::exact::#internal_ident #parameters;

                    #vis use #alias_unique_ident as #ident;
                })
            } else {

                Ok(quote_spanned! { span => 
                    // Setup for a glob import
                    mod #submodule_ident {
                        #super_vis use super::exact::#ident as #ident;

                        pub(super) mod globbed {
                            // Use the macro if it exists. The type will be imported, but...
                            #super_super_vis use super::*;
                            // Overwritten by our 'reduced generics' type alias
                            #super_super_vis type #ident #parameters = super::super::exact::#internal_ident #parameters;
                        }
                    }
                    
                    #vis use #submodule_ident::globbed::#ident;
                })
            }
        } else {
            // TODO In the future, we could do some special handling to support some non-path types,
            // but for now do not provide a macro
            Ok(quote_spanned! { span =>
                #vis use self::exact::#internal_ident as #ident;
            })
        }
    }
}
