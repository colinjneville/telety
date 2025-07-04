use proc_macro2::Span;
use syn::{Ident, PathArguments, PathSegment, UseTree};

use super::calling_crate;

pub struct Decrateify(PathSegment);

impl Decrateify {
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

impl Default for Decrateify {
    fn default() -> Self {
        Self::new()
    }
}

impl directed_visit::syn::visit::FullMut for Decrateify {
    fn visit_path_mut<D>(visitor: directed_visit::Visitor<'_, D, Self>, node: &mut syn::Path)
    where
        D: directed_visit::DirectMut<Self, syn::Path> + ?Sized,
    {
        if let Some(first_segment) = node.segments.first_mut() {
            if first_segment.ident == "crate" {
                let mut segment = visitor.0.clone();
                segment.ident.set_span(first_segment.ident.span());
                *first_segment = segment;
                node.leading_colon = Some(Default::default());
            }
        }

        directed_visit::Visitor::visit_mut(visitor, node);
    }

    fn visit_item_use_mut<D>(visitor: directed_visit::Visitor<'_, D, Self>, node: &mut syn::ItemUse)
    where
        D: directed_visit::DirectMut<Self, syn::ItemUse> + ?Sized,
    {
        if let UseTree::Path(path) = &mut node.tree {
            if path.ident == "crate" {
                let mut ident = visitor.0.ident.clone();
                ident.set_span(path.ident.span());
                path.ident = ident;
                node.leading_colon = Some(Default::default());
            }
        }

        directed_visit::Visitor::visit_mut(visitor, node);
    }
}
