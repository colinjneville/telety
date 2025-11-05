use proc_macro2::Span;
use syn::{Ident, PathArguments, PathSegment, UseTree};

use super::calling_crate;

pub struct Crateify(PathSegment);

impl Crateify {
    pub fn new() -> Self {
        Self::new_as_crate(calling_crate(Span::call_site()))
    }

    pub fn new_as_crate(crate_ident: Ident) -> Self {
        Self(PathSegment {
            ident: crate_ident,
            arguments: PathArguments::None,
        })
    }
}

impl Default for Crateify {
    fn default() -> Self {
        Self::new()
    }
}

impl directed_visit::syn::visit::FullMut for Crateify {
    fn visit_path_mut<D>(visitor: directed_visit::Visitor<'_, D, Self>, node: &mut syn::Path)
    where
        D: directed_visit::DirectMut<Self, syn::Path> + ?Sized,
    {
        if let Some(first_segment) = node.segments.first_mut()
            && *first_segment == visitor.0
        {
            first_segment.ident = Ident::new("crate", first_segment.ident.span());
            node.leading_colon = None;
        }

        directed_visit::Visitor::visit_mut(visitor, node);
    }

    fn visit_item_use_mut<D>(visitor: directed_visit::Visitor<'_, D, Self>, node: &mut syn::ItemUse)
    where
        D: directed_visit::DirectMut<Self, syn::ItemUse> + ?Sized,
    {
        if let UseTree::Path(path) = &mut node.tree
            && path.ident == visitor.0.ident
        {
            path.ident = Ident::new("crate", path.ident.span());
            node.leading_colon = None;
        }

        directed_visit::Visitor::visit_mut(visitor, node);
    }
}
