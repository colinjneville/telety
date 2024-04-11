use proc_macro2::Span;
use syn::{
    visit_mut::{self, VisitMut},
    Ident, ItemUse, Path, PathArguments, PathSegment, UseTree,
};

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

impl VisitMut for Decrateify {
    fn visit_path_mut(&mut self, i: &mut Path) {
        if let Some(first_segment) = i.segments.first_mut() {
            if first_segment.ident == "crate" {
                let mut segment = self.0.clone();
                segment.ident.set_span(first_segment.ident.span());
                *first_segment = segment;
                i.leading_colon = Some(Default::default());
            }
        }

        visit_mut::visit_path_mut(self, i);
    }

    fn visit_item_use_mut(&mut self, i: &mut ItemUse) {
        if let UseTree::Path(path) = &mut i.tree {
            if path.ident == "crate" {
                let mut ident = self.0.ident.clone();
                ident.set_span(path.ident.span());
                path.ident = ident;
                i.leading_colon = Some(Default::default());
            }
        }
    }
}
