use std::collections::HashMap;

use quote::{format_ident, quote, TokenStreamExt as _};
use syn::{parse_quote, visit::Visit as _, visit_mut::VisitMut as _};

use crate::{alias, Alias, syn_util, visitor};

#[derive(Debug)]
pub(crate) struct Root {
    telety_path: Option<syn::Path>,
    map_path: syn::Path,
    generics: syn::Generics,
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
    primary: Option<alias::Target>,
    // Maps exact type to index
    lookup: HashMap<syn::TypePath, alias::Index>,
    // Maps index to de-Self'ed type
    list: Vec<alias::Target>,
}

impl<'p> Map<'p> {
    pub(crate) fn new_root(
        telety_path: Option<syn::Path>,
        map_path: syn::Path,
        module: alias::Module,
        generics: syn::Generics,
        unique_ident: syn::Ident,
    ) -> Self {
        let root = Root {
            telety_path,
            map_path,
            generics,
        };
        
        Self {
            root: OwnedOrRef::Owned(root),
            parent: None,

            module,
            unique_ident,
            primary: None,
            lookup: HashMap::new(),
            list: vec![]
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
            list: vec![],
        }
    }

    pub(crate) fn set_self(&mut self, self_type: syn::TypePath) {
        // TODO assumes `self_type` uses all generic parameters
        self.lookup.insert(self_type.clone(), alias::Index::Primary);
        self.lookup.insert(parse_quote!(Self), alias::Index::Primary);
        self.primary = Some(alias::Target::new(self.generics().clone(), self_type));
    }

    fn root(&self) -> &Root {
        self.root.as_ref()
    }

    fn sub_map_iter(&self) -> SubMapIter<'_, 'p> {
        SubMapIter(Some(self))
    }

    fn get_alias_internal(&self, ty: &syn::TypePath) -> Option<Alias> {
        if let Some(primary) = self.primary.as_ref() {
            if ty == &primary.aliased_type {
                return Some(Alias::new(self, primary, alias::Index::Primary));
            }
        }
        
        if let Some(&index) = self.lookup.get(ty) {
            // TODO this is redundant and janky now
            match index {
                alias::Index::Primary => 
                    Some(Alias::new(self, self.primary.as_ref().unwrap(), alias::Index::Primary)),
                alias::Index::Secondary(i) => 
                    Some(Alias::new(self, &self.list[i], alias::Index::Secondary(i))),
            }
        } else if let Some(parent) = self.parent {
            parent.get_alias_internal(ty)
        } else {
            None
        }
    }

    pub fn iter_aliases(&self) -> impl Iterator<Item = Alias> {
        let primary_aliases = self.primary
            .as_ref()
            .map(|p| Alias::new(self, p, alias::Index::Primary))
            .into_iter();

        let secondary_aliases = self.list
            .iter()
            .enumerate()
            .map(|(i, target)| Alias::new(self, target, alias::Index::Secondary(i)));
        
        primary_aliases.chain(secondary_aliases)
    }
}

struct SubMapIter<'m, 'p>(Option<&'m Map<'p>>);

impl<'m, 'p> Iterator for SubMapIter<'m, 'p> {
    type Item = &'m Map<'p>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(map) = self.0.take() {
            if let Some(parent) = map.parent {
                self.0 = Some(parent);
            }
            Some(map)
        } else {
            None
        }
    }
}

impl<'p> Map<'p> {
    pub fn new_sub_map(&self, suffix: &str) -> Map {
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
    pub fn insert(&mut self, ty: syn::TypePath) -> bool {
        if self.get_alias(&ty).is_some() {
            false
        } else {
            let mut deselfed_ty = ty.clone();
        
            let mut visitor = visitor::ApplyAliases::new(self);
            visitor.set_apply_free_types(false);
            visitor.visit_type_path_mut(&mut deselfed_ty);

            if let Some(alias) = self.get_alias(&deselfed_ty) {
                // Cache this alternate Self-containing type
                self.lookup.insert(ty, alias.index);
                false
            } else {
                let index = alias::Index::Secondary(self.list.len());
                
                let mut visitor = visitor::UnusedParams::new();
                visitor.visit_type_path(&deselfed_ty);
                let mut generics = self.generics().clone();
                if let Some(placeholder_lifetime) = placeholder_visitor.anonymous_lifetime() {
                    generics.params.push(parse_quote!(#placeholder_lifetime));
                }
                visitor.remove_unused(&mut generics);

                if ty != deselfed_ty {
                    self.lookup.insert(deselfed_ty.clone(), index);
                }
                self.lookup.insert(ty, index);
                self.list.push(alias::Target::new(generics, deselfed_ty));

                true
            }
        }
    }

    pub fn get_self(&self) -> Option<Alias> {
        for map in self.sub_map_iter() {
            if let Some(primary) = map.primary.as_ref() {
                return Some(Alias::new(map, primary, alias::Index::Primary));
            }
        }

        None
    }

    pub fn get_alias(&self, ty: &syn::TypePath) -> Option<Alias> {
        match self.get_alias_internal(&ty) {
            Some(alias) => Some(alias),
            None => {
                let mut deselfed_ty = ty.clone();
                let mut visitor = visitor::ApplyAliases::new(self);
                visitor.set_apply_free_types(false);
                visitor.visit_type_path_mut(&mut deselfed_ty);

                self.get_alias_internal(&deselfed_ty)
            }
        }
    }

    pub fn visitor(&self) -> visitor::ApplyAliases {
        visitor::ApplyAliases::new(self)
    }

    pub fn with_module(&self) -> impl quote::ToTokens + use<'p> {
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