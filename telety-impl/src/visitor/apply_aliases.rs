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
                if visitor.apply_associated_types
                    && let Some(self_mapped) = visitor.map.get_self()
                {
                    node.path = self_mapped.to_path();
                }
                break 'apply;
            }

            if visitor.apply_free_types
                && node.qself.is_none()
                && let Ok(Some(mapped)) = visitor.map.get_alias(&node.path)
            {
                node.path = mapped.to_path();
                break 'apply;
            }
        };

        directed_visit::Visitor::visit_mut(visitor, node);
    }

    fn visit_trait_bound_mut<D>(
        visitor: directed_visit::Visitor<'_, D, Self>,
        node: &mut syn::TraitBound,
    ) where
        D: directed_visit::DirectMut<Self, syn::TraitBound> + ?Sized,
    {
        if visitor.apply_free_types
            && let Ok(Some(mapped)) = visitor.map.get_alias(&node.path)
        {
            let mut kept = vec![];

            if let Some(last_segment) = node.path.segments.last_mut()
                && let syn::PathArguments::AngleBracketed(args) =
                    std::mem::take(&mut last_segment.arguments)
            {
                for arg in args.args {
                    if let arg @ (syn::GenericArgument::AssocType(_)
                    | syn::GenericArgument::AssocConst(_)
                    | syn::GenericArgument::Constraint(_)) = arg
                    {
                        kept.push(arg);
                    }
                }
            }

            node.path = mapped.to_path();
            if let Some(last_segment) = node.path.segments.last_mut()
                && !kept.is_empty()
            {
                if let syn::PathArguments::None = &mut last_segment.arguments {
                    last_segment.arguments =
                        syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                            colon2_token: Default::default(),
                            lt_token: Default::default(),
                            args: Default::default(),
                            gt_token: Default::default(),
                        });
                }

                if let syn::PathArguments::AngleBracketed(args) = &mut last_segment.arguments {
                    args.args.extend(kept);
                }
            }
        }
    }
}
