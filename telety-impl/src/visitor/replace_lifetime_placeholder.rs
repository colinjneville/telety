pub(crate) struct ReplaceLifetimePlaceholder(Option<syn::Lifetime>);

impl ReplaceLifetimePlaceholder {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn anonymous_lifetime(&self) -> Option<&syn::Lifetime> {
        self.0.as_ref()
    }
}

const ANON_LIFETIME: &str = "__anon";

impl syn::visit_mut::VisitMut for ReplaceLifetimePlaceholder {
    fn visit_lifetime_mut(&mut self, i: &mut syn::Lifetime) {
        if i.ident == "_" {
            let span = i.ident.span();
            i.ident = syn::Ident::new(ANON_LIFETIME, span);
            self.0.get_or_insert_with(|| syn::Lifetime {
                apostrophe: i.apostrophe.clone(),
                ident: i.ident.clone(),
            });
        }

        syn::visit_mut::visit_lifetime_mut(self, i);
    }
}