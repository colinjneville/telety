use quote::format_ident;

/// Indicates the type and index of an alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Index {
    /// An alias to the item itself, like `Self`.
    Primary,
    /// An alias to a top-level type in the item.
    Secondary(usize),
}

impl Index {
    pub(crate) fn ident(self, friendly_ident: &syn::Ident) -> syn::Ident {
        format_ident!("{}__{friendly_ident}", self.ident_core())
    }

    pub(crate) fn ident_internal(self, friendly_ident: &syn::Ident) -> syn::Ident {
        format_ident!("{}Internal__{friendly_ident}", self.ident_core())
    }

    pub(crate) fn ident_core(self) -> syn::Ident {
        match self {
            Self::Primary => format_ident!("AliasSelf"),
            Self::Secondary(index) => format_ident!("Alias{index}"),
        }
    }
}
