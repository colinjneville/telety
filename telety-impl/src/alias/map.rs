use std::collections::HashMap;

use quote::{TokenStreamExt as _, format_ident, quote};
use syn::{parse_quote, spanned::Spanned as _};

use crate::{Alias, Options, alias, syn_util, visitor};

#[derive(Debug)]
pub(crate) struct Root {
    telety_path: Option<syn::Path>,
    map_path: syn::Path,
    generics: syn::Generics,
    alias_traits: Option<bool>,
}

#[derive(Debug)]
enum OwnedOrRef<'l, T> {
    Owned(T),
    Ref(&'l T),
}

impl<'l, T> AsRef<T> for OwnedOrRef<'l, T> {
    fn as_ref(&self) -> &T {
        match self {
            OwnedOrRef::Owned(owned) => owned,
            OwnedOrRef::Ref(rf) => rf,
        }
    }
}

#[derive(Debug)]
pub struct Map<'p> {
    root: OwnedOrRef<'p, Root>,
    parent: Option<&'p Map<'p>>,

    module: alias::Module,
    unique_ident: syn::Ident,
    primary: Option<(alias::Path, alias::Arguments)>,
    // Maps exact type to index
    lookup: HashMap<alias::Path, (usize, alias::Arguments, alias::Kind)>,
    // // Maps index to de-Self'ed type
    // list: Vec<alias::Path>,
}

impl<'p> Map<'p> {
    pub(crate) fn new_root(
        telety_path: Option<syn::Path>,
        map_path: syn::Path,
        module: alias::Module,
        generics: syn::Generics,
        unique_ident: syn::Ident,
        options: &Options,
    ) -> Self {
        let root = Root {
            telety_path,
            map_path,
            generics,
            alias_traits: options.alias_traits,
        };

        Self {
            root: OwnedOrRef::Owned(root),
            parent: None,

            module,
            unique_ident,
            primary: None,
            lookup: HashMap::new(),
            // list: vec![]
        }
    }

    pub(crate) fn new_child(&'p self, suffix: &str) -> Self {
        let module = self.module.new_child(suffix);
        let parent_ident = &self.unique_ident;
        let unique_ident = format_ident!("{parent_ident}__{suffix}");
        Self {
            root: OwnedOrRef::Ref(self.root()),
            parent: Some(self),

            module,
            unique_ident,
            primary: None,
            lookup: HashMap::new(),
            // list: vec![],
        }
    }

    pub(crate) fn set_self(&mut self, self_type: &syn::Path) -> Result<(), alias::Error> {
        let (path, args) = alias::Path::new(self_type)?;
        // Self may have 'baked-in' generic parameters, so we can't always reuse the same alias.
        // If the explicit type also appears, we can just add it as an ordinary secondary alias
        self.primary = Some((path, args));

        Ok(())
    }

    pub(crate) fn full_lookup<'map>(
        &'map self,
        ty: &syn::Path,
    ) -> Result<Option<Alias<'map>>, alias::Error> {
        match self.local_lookup(ty)? {
            some @ Some(_) => Ok(some),
            None => {
                if let Some(parent) = self.parent {
                    parent.full_lookup(ty)
                } else {
                    Ok(None)
                }
            }
        }
    }

    pub(crate) fn local_lookup<'map>(
        &'map self,
        ty: &syn::Path,
    ) -> Result<Option<Alias<'map>>, alias::Error> {
        let (path, args) = alias::Path::new(ty)?;
        if let Some((path, (index, _canon_args, kind))) = self.lookup.get_key_value(&path) {
            Ok(Some(Alias::new(
                self,
                path,
                alias::Index::Secondary(*index),
                args,
                *kind,
            )))
        } else {
            Ok(None)
        }
    }

    pub(crate) fn local_get_self(&self) -> Option<Alias<'_>> {
        if let Some((path, args)) = &self.primary {
            Some(Alias::new(
                self,
                path,
                alias::Index::Primary,
                args.clone(),
                alias::Kind::Type,
            ))
        } else {
            None
        }
    }

    fn root(&self) -> &Root {
        self.root.as_ref()
    }

    /// Iterate all [Alias]es at this map level ([Alias]es from parent maps are not included)
    pub fn iter_aliases(&self) -> impl Iterator<Item = Alias<'_>> {
        let primary_aliases = self.local_get_self().into_iter();

        let secondary_aliases = self.lookup.iter().map(|(path, (index, args, alias_type))| {
            Alias::new(
                self,
                path,
                alias::Index::Secondary(*index),
                args.clone(),
                *alias_type,
            )
        });

        primary_aliases.chain(secondary_aliases)
    }
}

impl<'p> Map<'p> {
    pub fn new_sub_map(&self, suffix: &str) -> Map<'_> {
        Map::new_child(self, suffix)
    }

    pub fn telety_path(&self) -> Option<&syn::Path> {
        self.root().telety_path.as_ref()
    }

    pub fn map_path(&self) -> &syn::Path {
        &self.root().map_path
    }

    pub fn module(&self) -> &alias::Module {
        &self.module
    }

    pub fn visibility(&self) -> &syn::Visibility {
        self.module.visibility()
    }

    pub fn unique_ident(&self) -> &syn::Ident {
        &self.unique_ident
    }

    pub fn generics(&self) -> &syn::Generics {
        &self.root().generics
    }

    /// Register a [syn::TypePath] in the [Map]. If the exact (i.e. identical tokens, not equivalent Rust types) type
    /// already exists in the map, this is a no-op. Maps constructed with the same parameters and order of
    /// inserts will yield the same [Alias]es.
    pub fn insert_type(&mut self, ty: &syn::TypePath) -> Result<bool, alias::Error> {
        // Associated types are not supported (watch https://github.com/rust-lang/rust/issues/134691)
        if ty.qself.is_some() {
            return Err(alias::Error::new(
                ty.span(),
                alias::error::Kind::AssociatedType,
            ));
        }
        self.insert(&ty.path, alias::Kind::Type)
    }

    /// Register a [syn::Path] to a trait in the [Map]. If the exact (i.e. identical tokens, not equivalent Rust types) type
    /// already exists in the map, this is a no-op. Maps constructed with the same parameters and order of
    /// inserts will yield the same [Alias]es.
    pub fn insert_trait(&mut self, ty: &syn::Path) -> Result<bool, alias::Error> {
        let can_insert = self
            .root()
            .alias_traits
            .unwrap_or_else(|| ty.leading_colon.is_some() || ty.segments.len() > 1);

        if can_insert {
            self.insert(ty, alias::Kind::Trait)
        } else {
            Err(alias::Error::new(ty.span(), alias::error::Kind::Trait))
        }
    }

    fn insert(&mut self, ty: &syn::Path, alias_type: alias::Kind) -> Result<bool, alias::Error> {
        if ty.is_ident("Self") {
            // TODO should this be an error instead?
            Ok(false)
        } else if self.full_lookup(ty)?.is_some() {
            // Path already exists
            Ok(false)
        } else {
            let (path, mut args) = alias::Path::new(ty)?;

            let index = self.lookup.len();
            args.parameterize();

            self.lookup.insert(path, (index, args, alias_type));

            Ok(true)
        }
    }

    pub fn get_self(&self) -> Option<Alias<'_>> {
        if let Some(alias) = self.local_get_self() {
            Some(alias)
        } else if let Some(parent) = self.parent {
            parent.get_self()
        } else {
            None
        }
    }

    pub fn get_alias<'map>(
        &'map self,
        ty: &syn::Path,
    ) -> Result<Option<Alias<'map>>, alias::Error> {
        if ty == &parse_quote!(Self) {
            Ok(self.get_self())
        } else {
            self.full_lookup(ty)
        }
    }

    pub fn visitor(&self) -> visitor::ApplyAliases<'_> {
        visitor::ApplyAliases::new(self)
    }

    pub fn with_module(&self) -> impl quote::ToTokens + use<'_> {
        self.module.with_contents(self)
    }
}

impl<'p> quote::ToTokens for Map<'p> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        // items inside our module need to convert blank vis to `pub(super)`, etc.
        let super_visibility = syn_util::super_visibility(self.module.visibility());

        let exact_aliases = self.iter_aliases().map(alias::Alias::exact);

        let public_aliases = self.iter_aliases().map(alias::Alias::public);

        let map_mod = quote! {
            mod exact {
                #super_visibility use super::super::*;

                #(#exact_aliases)*
            }

            #(#public_aliases)*
        };

        tokens.append_all(map_mod);
    }
}
