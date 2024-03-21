use std::collections::HashSet;

use syn::{
    visit::{self, Visit},
    GenericParam, Ident, Type,
};

use crate::{alias, item_data::ItemData};

pub(crate) struct IdentifyAliases<'ast, 'm> {
    alias_map: &'m mut alias::Map,
    generic_scopes: Vec<GenericScope<'ast>>,
}

impl<'ast, 'm> IdentifyAliases<'ast, 'm> {
    pub(crate) fn new(alias_map: &'m mut alias::Map) -> Self {
        Self {
            alias_map,
            generic_scopes: vec![],
        }
    }
}

impl<'ast, 'am> Visit<'ast> for IdentifyAliases<'ast, 'am> {
    fn visit_type(&mut self, i: &'ast Type) {
        if let Type::Path(type_path) = i {
            if let Some(qself) = &type_path.qself {
                // TODO Currently ignoring associated types
                if qself.position > 0 {
                    visit::visit_type(self, i);
                    return;
                }
            }
        }

        if self.alias_map.insert(i).is_some() {
            visit::visit_type(self, i);
        }
    }

    fn visit_item(&mut self, i: &'ast syn::Item) {
        if let Some(generics) = i.generics() {
            self.generic_scopes.push(GenericScope::new());
            let scope = self.generic_scopes.last_mut().unwrap();

            scope.visit_generics(generics);

            i.visit_generics_scope(self);

            self.generic_scopes
                .pop()
                .expect("generic scope stack is mismatched");
        }
    }
}

struct GenericScope<'ast>(HashSet<&'ast Ident>);

impl<'ast> GenericScope<'ast> {
    pub fn new() -> Self {
        // TODO nested generic scopes won't actually work at the moment
        Self(Default::default())
    }

    #[allow(dead_code)]
    pub fn contains(&self, ident: &Ident) -> bool {
        self.0.contains(ident)
    }
}

impl<'ast> Visit<'ast> for GenericScope<'ast> {
    fn visit_generic_param(&mut self, i: &'ast GenericParam) {
        match i {
            GenericParam::Lifetime(_) => {}
            GenericParam::Type(ty) => {
                self.0.insert(&ty.ident);
            }
            GenericParam::Const(_cnst) => {
                // TODO
            }
        }
    }
}
