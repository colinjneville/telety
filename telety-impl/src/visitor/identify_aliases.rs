use std::collections::HashSet;

use crate::alias;

pub struct IdentifyAliases<'m, 'map> {
    alias_map: &'m mut alias::Map<'map>,
    parameters: HashSet<syn::Ident>,
}

impl<'m, 'map> IdentifyAliases<'m, 'map> {
    pub fn new(alias_map: &'m mut alias::Map<'map>) -> Self {
        let parameters = alias_map
            .generics()
            .params
            .iter()
            .filter_map(|p| match p {
                syn::GenericParam::Lifetime(_lifetime_param) => None,
                syn::GenericParam::Type(type_param) => Some(&type_param.ident),
                syn::GenericParam::Const(const_param) => Some(&const_param.ident),
            })
            .cloned()
            .collect();

        Self {
            alias_map,
            parameters,
        }
    }

    fn is_parameter(&self, ident: &syn::Ident) -> bool {
        self.parameters.contains(ident)
    }
}

impl<'m, 'map> directed_visit::syn::visit::Full for IdentifyAliases<'m, 'map> {
    fn visit_generics_enter<D>(
        mut visitor: directed_visit::Visitor<'_, D, Self>,
        node: &directed_visit::syn::GenericsEnter,
    ) where
        D: directed_visit::Direct<Self, directed_visit::syn::GenericsEnter> + ?Sized,
    {
        for param in node {
            if let syn::GenericParam::Type(param) = param {
                visitor.parameters.insert(param.ident.clone());
            }
        }
    }

    fn visit_generics_exit<D>(
        mut visitor: directed_visit::Visitor<'_, D, Self>,
        node: &directed_visit::syn::GenericsExit,
    ) where
        D: directed_visit::Direct<Self, directed_visit::syn::GenericsExit> + ?Sized,
    {
        for param in node {
            if let syn::GenericParam::Type(param) = param {
                visitor.parameters.remove(&param.ident);
            }
        }
    }

    fn visit_type_path<D>(mut visitor: directed_visit::Visitor<'_, D, Self>, node: &syn::TypePath)
    where
        D: directed_visit::Direct<Self, syn::TypePath> + ?Sized,
    {
        if let Some(first_segment) = node.path.segments.first()
            && node.path.leading_colon.is_none()
            && (first_segment.ident == "Self" || visitor.is_parameter(&first_segment.ident))
        {
            // TypePath is a type parameter or associated type of one
            return;
        }

        // No error handling, we just alias everything we are able to
        let _ = visitor.alias_map.insert_type(node);

        directed_visit::Visitor::visit(visitor, node);
    }

    fn visit_trait_bound<D>(
        mut visitor: directed_visit::Visitor<'_, D, Self>,
        node: &syn::TraitBound,
    ) where
        D: directed_visit::Direct<Self, syn::TraitBound> + ?Sized,
    {
        // No error handling, we just alias everything we are able to
        let _ = visitor.alias_map.insert_trait(&node.path);
        directed_visit::Visitor::visit(visitor, node);
    }
}
