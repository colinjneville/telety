use quote::format_ident;
use syn::Ident;

use crate::alias;

/// Indicates the type and index of an alias.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Index {
    /// An alias to the item itself, like `Self`.
    Primary,
    /// An alias to a top-level type in the item.
    Secondary(usize),
}

impl Index {
    pub(crate) fn ident(self) -> Ident {
        match self {
            alias::Index::Primary => format_ident!("AliasSelf"),
            alias::Index::Secondary(index) => format_ident!("Alias{index}"),
        }
    }
}
