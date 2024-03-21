use syn::{
    visit_mut::{self, VisitMut},
    Type,
};

use crate::alias;

pub struct ApplySecondaryAliases<'m> {
    map: &'m alias::Map,
}

impl<'m> ApplySecondaryAliases<'m> {
    pub(crate) fn new(map: &'m alias::Map) -> Self {
        Self { map }
    }
}

impl<'m> VisitMut for ApplySecondaryAliases<'m> {
    fn visit_type_mut(&mut self, i: &mut Type) {
        if let Some(index) = self.map.get_index(i) {
            *i = self.map.alias(index).ty();
        } else {
            // TODO should this ever happen?
            visit_mut::visit_type_mut(self, i);
        }
    }
}
