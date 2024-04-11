use proc_macro2::Span;
use syn::{
    visit_mut::{self, VisitMut},
    Ident, ItemUse, Path, PathArguments, PathSegment, UseTree,
};

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

impl VisitMut for Crateify {
    fn visit_path_mut(&mut self, i: &mut Path) {
        if let Some(first_segment) = i.segments.first_mut() {
            if *first_segment == self.0 {
                first_segment.ident = Ident::new("crate", first_segment.ident.span());
                i.leading_colon = None;
            }
        }

        visit_mut::visit_path_mut(self, i);
    }

    fn visit_item_use_mut(&mut self, i: &mut ItemUse) {
        if let UseTree::Path(path) = &mut i.tree {
            if path.ident == self.0.ident {
                path.ident = Ident::new("crate", path.ident.span());
                i.leading_colon = None;
            }
        }

        visit_mut::visit_item_use_mut(self, i);
    }
}
