use quote::quote;
use syn::spanned::Spanned as _;

use crate::{command::ParameterIdents, Command};

pub(crate) const VERSION: usize = 1;

/// Replaces `needle` with the unique identifier for this item.
pub const UNIQUE_IDENT: Command = Command::new(VERSION, "unique_ident", |ty| {
    let ParameterIdents {
        needle, haystack, ..
    } = ParameterIdents::new(ty.item().span());
    let replacement = ty.unique_ident();

    let telety_path = ty.options().telety_path();

    Some(quote! {
        #telety_path::__private::find_and_replace!(
            $#needle,
            [#replacement],
            $($#haystack)*
        )
    })
});

/// Replaces `needle` with the full definition of the item.
pub const TY: Command = Command::new(VERSION, "ty", |ty| {
    let ParameterIdents {
        needle, haystack, ..
    } = ParameterIdents::new(ty.item().span());

    let options = ty.options();
    let item = ty.item();
    let replacement = quote! {
        #[telety(#options)]
        #item
    };

    let telety_path = ty.options().telety_path();

    Some(quote! {
        #telety_path::__private::find_and_replace!(
            $#needle,
            [#replacement],
            $($#haystack)*
        )
    })
});

pub(crate) const COMMANDS: &[Command] = &[UNIQUE_IDENT, TY];
