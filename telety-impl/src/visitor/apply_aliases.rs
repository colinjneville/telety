use syn::visit_mut::{self, VisitMut};

use crate::alias;

pub struct ApplyAliases<'map> {
    map: &'map alias::Map<'map>,
    apply_free_types: bool,
    apply_associated_types: bool,
}

impl<'map> ApplyAliases<'map> {
    pub(crate) fn new(map: &'map alias::Map) -> Self {
        Self {
            map,
            apply_free_types: true,
            apply_associated_types: true,
        }
    }

    pub fn set_apply_free_types(&mut self, apply_free_types: bool) {
        self.apply_free_types = apply_free_types;
    }

    pub fn set_apply_associated_types(&mut self, apply_associated_types: bool) {
        self.apply_associated_types = apply_associated_types;
    }
}

impl<'map> VisitMut for ApplyAliases<'map> {
    fn visit_type_path_mut(&mut self, i: &mut syn::TypePath) {
        // Replace `Self` with a global path
        // TODO Associated types, if ever supported, would also be done here
        if self.apply_associated_types {
            if i.qself.is_none() && i.path.is_ident("Self") {
                if let Some(self_mapped) = self.map.get_self() {
                    *i = self_mapped.qualified_type_path();
                    return;
                }
            }
        }

        if self.apply_free_types {
            if let Some(mapped) = self.map.get_alias(i) {
                *i = mapped.qualified_type_path();
                return;
            }
        }

        visit_mut::visit_type_path_mut(self, i);
    }
}
