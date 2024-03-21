use std::collections::HashMap;

use syn::{visit::Visit as _, visit_mut::VisitMut as _, Generics, Type};

use crate::{alias, visitor, Alias};

#[derive(Debug)]
pub struct Map {
    group: alias::Group,
    primary: alias::Details,
    // Maps exact type to index
    lookup: HashMap<Type, usize>,
    // Maps index to de-Self'ed type
    list: Vec<alias::Details>,
}

impl Map {
    pub(crate) fn new(group: alias::Group, parameters: Generics, primary_type: Type) -> Self {
        Self {
            group,
            primary: alias::Details::new(parameters, primary_type),
            lookup: HashMap::new(),
            list: vec![],
        }
    }

    pub(crate) fn insert(&mut self, ty: &Type) -> Option<usize> {
        if !self.lookup.contains_key(ty) {
            let mut deselfed_ty = ty.clone();
            let mut visitor = visitor::ApplyPrimaryAlias::new(self);
            visitor.visit_type_mut(&mut deselfed_ty);

            let index = self.list.len();
            self.lookup.insert(ty.clone(), index);

            let mut visitor = visitor::UnusedParams::new();
            visitor.visit_type(&deselfed_ty);
            let mut parameters = self.primary.parameters.clone();
            visitor.remove_unused(&mut parameters);

            self.list.push(alias::Details::new(parameters, deselfed_ty));

            Some(index)
        } else {
            None
        }
    }

    pub(crate) fn group(&self) -> &alias::Group {
        &self.group
    }

    /// Look up the [Index](alias::Index) of a given type.  
    /// Returns [None] if there is no alias for the type.  
    /// Note that lookup is done based on exact token equality, not type equality.
    pub fn get_index(&self, ty: &Type) -> Option<alias::Index> {
        let mut deselfed_ty = ty.clone();
        let mut visitor = visitor::ApplyPrimaryAlias::new(self);
        visitor.visit_type_mut(&mut deselfed_ty);

        if self.primary.aliased_type == deselfed_ty {
            Some(alias::Index::Primary)
        } else if let Some(&index) = self.lookup.get(&deselfed_ty) {
            Some(alias::Index::Secondary(index))
        } else {
            None
        }
    }

    /// Look up the [Alias] of a given type.  
    /// Returns [None] if there is no alias for the type.  
    /// Note that lookup is done based on exact token equality, not type equality.  
    /// Equivalent to [Self::get_index] followed by [Self::alias].
    pub fn alias_of(&self, ty: &Type) -> Option<Alias> {
        let index = self.get_index(ty)?;
        Some(self.alias(index))
    }

    /// Get the [Alias] corresponding to an [alias::Index].
    pub fn alias(&self, index: alias::Index) -> Alias {
        let details = match index {
            alias::Index::Primary => &self.primary,
            alias::Index::Secondary(index) => &self.list[index],
        };
        Alias::new(self, index, details)
    }

    /// Iterate through all [alias::Index]es.
    pub fn iter(&self) -> Iter {
        Iter::new(self)
    }

    /// Create a visitor which can be used to replace all types found in this map
    /// with their aliases.
    pub fn visitor(&self) -> visitor::ApplySecondaryAliases {
        visitor::ApplySecondaryAliases::new(self)
    }
}

/// Iterator over [alias::Index]es
pub struct Iter<'m> {
    map: &'m alias::Map,
    state: IterState,
}

enum IterState {
    Primary,
    Secondary(usize),
}

impl<'m> Iter<'m> {
    fn new(map: &'m alias::Map) -> Self {
        Self {
            map,
            state: IterState::Primary,
        }
    }
}

impl<'m> Iterator for Iter<'m> {
    type Item = Alias<'m>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.state {
            IterState::Primary => {
                self.state = IterState::Secondary(0);
                Some(Alias::new(self.map, alias::Index::Primary, &self.map.primary))
            }
            IterState::Secondary(index) => {
                let details = self.map.list.get(*index)?;
                let n = Alias::new(self.map, alias::Index::Secondary(*index), details);
                *index += 1;
                Some(n)
            }
        }
    }
}

impl<'m> IntoIterator for &'m alias::Map {
    type Item = Alias<'m>;

    type IntoIter = Iter<'m>;

    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
