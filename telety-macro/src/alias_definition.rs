use quote::format_ident;
use syn::Ident;
use telety_impl::Alias;

pub(crate) struct Definition {
    pub(crate) ident: Ident,
    pub(crate) internal_ident: Ident,
    pub(crate) macro_maker_ident: Ident,
    pub(crate) alias_unique_ident: Ident,
    pub(crate) submodule_ident: Ident,
}

impl Definition {
    pub(crate) fn new(alias: Alias, unique_ident: Ident) -> Self {
        let ident = alias.ident();

        let unique_ident = format_ident!("{unique_ident}_{ident}");

        let macro_maker_ident = format_ident!("make_{unique_ident}");

        Self {
            internal_ident: format_ident!("{ident}Internal"),
            macro_maker_ident,
            alias_unique_ident: unique_ident,
            submodule_ident: format_ident!("{ident}Mod"),

            ident,
        }
    }
}
