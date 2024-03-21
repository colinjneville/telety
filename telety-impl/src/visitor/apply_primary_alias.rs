use syn::{
    visit_mut::{self, VisitMut},
    TypePath,
};

use crate::alias;

/// Replace `Self` with a canonically-pathed alias
pub(crate) struct ApplyPrimaryAlias<'am> {
    map: &'am alias::Map,
}

impl<'am> VisitMut for ApplyPrimaryAlias<'am> {
    fn visit_type_path_mut(&mut self, i: &mut TypePath) {
        // Replace `Self` with a global path
        if i.qself.is_none() && i.path.is_ident("Self") {
            *i = self.map.alias(alias::Index::Primary).type_path();
            return;
        }

        visit_mut::visit_type_path_mut(self, i);
    }
}

impl<'am> ApplyPrimaryAlias<'am> {
    pub fn new(map: &'am alias::Map) -> Self {
        Self { map }
    }
}
