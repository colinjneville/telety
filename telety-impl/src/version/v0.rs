use quote::quote;
use syn::spanned::Spanned as _;

use crate::{command::ParameterIdents, Command};

pub(crate) const VERSION: usize = 0;

/// Returns the `haystack` unchanged, regardless of the `needle`.
pub const IDENTITY: Command = Command::new(
    VERSION,
    "identity",
    |ty| {
        let ParameterIdents { haystack, .. } = ParameterIdents::new(ty.item().span());

        Some(quote!($($#haystack)*))
    },
    None,
);

/// Replaces `needle` with the path to this item.
pub const PATH: Command = Command::new(
    VERSION,
    "path",
    |ty| {
        let ParameterIdents {
            needle, haystack, ..
        } = ParameterIdents::new(ty.item().span());
        let replacement = ty.path();
        Some(quote! {
            ::telety::__private::find_and_replace!(
                $#needle,
                [#replacement],
                $($#haystack)*
            )
        })
    },
    None,
);

pub(crate) const COMMANDS: &[Command] = &[IDENTITY, PATH];
