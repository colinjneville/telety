use quote::quote;
use syn::spanned::Spanned as _;

use crate::{command::ParameterIdents, Command};

pub(crate) const VERSION: usize = 0;

/// Replaces `needle` with the path to this item.
pub const PATH: Command = Command::new(VERSION, "path", |ty| {
    let ParameterIdents {
        needle, haystack, ..
    } = ParameterIdents::new(ty.item().span());
    let replacement = ty.path();

    let telety_path = ty.options().telety_path();

    Some(quote! {
        #telety_path::__private::find_and_replace! {
            $#needle,
            [#replacement],
            $($#haystack)*
        }
    })
});

pub(crate) const COMMANDS: &[Command] = &[PATH];
