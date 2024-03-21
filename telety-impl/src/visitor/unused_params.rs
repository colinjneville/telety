use std::{collections::HashSet, mem};

use syn::{
    punctuated,
    visit::{self, Visit},
    ExprPath, GenericParam, Generics, Ident, Lifetime, TypePath,
};

pub(crate) struct UnusedParams<'ast> {
    lifetimes: HashSet<&'ast Ident>,
    types: HashSet<&'ast Ident>,
    consts: HashSet<&'ast Ident>,
}

impl<'ast> UnusedParams<'ast> {
    pub fn new() -> Self {
        Self {
            lifetimes: HashSet::new(),
            types: HashSet::new(),
            consts: HashSet::new(),
        }
    }

    pub fn remove_unused(&self, generics: &mut Generics) {
        let params = mem::take(&mut generics.params);
        for param in params.into_pairs() {
            if match param.value() {
                GenericParam::Lifetime(lifetime) => {
                    self.lifetimes.contains(&lifetime.lifetime.ident)
                }
                GenericParam::Type(ty) => self.types.contains(&ty.ident),
                GenericParam::Const(cnst) => self.consts.contains(&cnst.ident),
            } {
                match param {
                    punctuated::Pair::Punctuated(t, p) => {
                        generics.params.push_value(t);
                        generics.params.push_punct(p);
                    }
                    punctuated::Pair::End(t) => generics.params.push_value(t),
                }
            }
        }
        generics.where_clause = None;
    }
}

impl<'ast> Visit<'ast> for UnusedParams<'ast> {
    fn visit_lifetime(&mut self, i: &'ast Lifetime) {
        self.lifetimes.insert(&i.ident);

        visit::visit_lifetime(self, i);
    }

    fn visit_type_path(&mut self, i: &'ast TypePath) {
        if let Some(ident) = i.path.get_ident() {
            self.types.insert(ident);
        }

        visit::visit_type_path(self, i);
    }

    fn visit_expr_path(&mut self, i: &'ast ExprPath) {
        if let Some(ident) = i.path.get_ident() {
            self.consts.insert(ident);
        }

        visit::visit_expr_path(self, i);
    }
}
