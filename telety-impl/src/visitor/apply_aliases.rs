use syn::{
    visit_mut::{self, VisitMut},
    Type, TypePath,
};

use crate::alias;

pub struct ApplyAliases<'m> {
    map: &'m alias::Map,
    associated_only: bool,
}

impl<'m> ApplyAliases<'m> {
    pub(crate) fn new(map: &'m alias::Map, associated_only: bool) -> Self {
        Self {
            map,
            associated_only,
        }
    }
}

impl<'m> VisitMut for ApplyAliases<'m> {
    fn visit_type_mut(&mut self, i: &mut Type) {
        if !self.associated_only {
            if let Some(alias) = self.map.alias_of(i) {
                *i = alias.ty();
                return;
            }
        }

        visit_mut::visit_type_mut(self, i);
    }

    fn visit_type_path_mut(&mut self, i: &mut TypePath) {
        // Replace `Self` with a global path
        // TODO Associated types, if ever supported, would also be done here
        if i.qself.is_none() && i.path.is_ident("Self") {
            *i = self.map.self_alias().type_path();
        } else {
            visit_mut::visit_type_path_mut(self, i);
        }
    }
}
