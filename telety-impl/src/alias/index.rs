use quote::format_ident;
use syn::Ident;

use crate::alias;

/// Indicates the type and index of an alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Index {
    /// An alias to the item itself, like `Self`.
    Primary,
    /// An alias to a top-level type in the item.
    Secondary(usize),
}

impl Index {
    pub(crate) fn definition(&self, unique_ident: Ident) -> alias::Definition {
        alias::Definition::new(*self, unique_ident)
    }

    pub(crate) fn ident(&self) -> Ident {
        match self {
            Self::Primary => format_ident!("AliasSelf"),
            Self::Secondary(index) => format_ident!("Alias{index}"),
        }
    }
}
