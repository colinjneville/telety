use std::collections::HashSet;

use syn::{
    visit::{self, Visit},
    GenericParam, Ident,
};

use crate::{alias, item_data::ItemData};

pub struct IdentifyAliases<'ast, 'm, 'map> {
    alias_map: &'m mut alias::Map<'map>,
    generic_scopes: Vec<GenericScope<'ast>>,
}

impl<'ast, 'm, 'map> IdentifyAliases<'ast, 'm, 'map> {
    pub fn new(alias_map: &'m mut alias::Map<'map>) -> Self {
        Self {
            alias_map,
            generic_scopes: vec![],
        }
    }

    pub fn push_generics_scope(&mut self, generics: &'ast syn::Generics) {
        self.generic_scopes.push(GenericScope::new());
        let scope = self.generic_scopes.last_mut().unwrap();
        scope.visit_generics(generics);
    }

    pub fn pop_generics_scope(&mut self) {
        self.generic_scopes
            .pop()
            .expect("generic scope stack is mismatched");
    }
}

impl<'ast, 'm, 'map> Visit<'ast> for IdentifyAliases<'ast, 'm, 'map> {
    fn visit_type_path(&mut self, i: &'ast syn::TypePath) {
        if self.alias_map.insert(i.clone()) {
            visit::visit_type_path(self, i);
        }
    }

    fn visit_item(&mut self, i: &'ast syn::Item) {
        if let Some(generics) = i.generics() {
            self.push_generics_scope(generics);

            i.visit_generics_scope(self);

            self.pop_generics_scope();
        }

        syn::visit::visit_item(self, i);
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
