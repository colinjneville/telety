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

impl<'map> directed_visit::syn::visit::FullMut for ApplyAliases<'map> {
    fn visit_type_path_mut<D>(
        visitor: directed_visit::Visitor<'_, D, Self>,
        node: &mut syn::TypePath,
    ) where
        D: directed_visit::DirectMut<Self, syn::TypePath> + ?Sized,
    {
        'apply: {
            // Replace `Self` with a global path
            // TODO Associated types, if ever supported, would also be done here
            if node.qself.is_none() && node.path.is_ident("Self") {
                if visitor.apply_associated_types {
                    if let Some(self_mapped) = visitor.map.get_self() {
                        *node = self_mapped.to_type_path();
                    }
                }
                break 'apply;
            }

            if visitor.apply_free_types {
                if let Ok(Some(mapped)) = visitor.map.get_alias(node) {
                    *node = mapped.to_type_path();
                    break 'apply;
                }
            }
        };

        directed_visit::Visitor::visit_mut(visitor, node);
    }
}
