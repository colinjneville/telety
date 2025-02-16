#[derive(Debug)]
pub(crate) struct Target {
    pub(crate) generics: syn::Generics,
    pub(crate) aliased_type: syn::TypePath,
}

impl Target {
    pub(crate) fn new(generics: syn::Generics, aliased_type: syn::TypePath) -> Self {
        Self {
            generics,
            aliased_type,
        }
    }
}
