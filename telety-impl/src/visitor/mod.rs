mod apply_generic_arguments;
pub use apply_generic_arguments::ApplyGenericArguments;
mod apply_primary_alias;
pub(crate) use apply_primary_alias::ApplyPrimaryAlias;
mod apply_secondary_aliases;
pub(crate) use apply_secondary_aliases::ApplySecondaryAliases;
mod crateify;
pub use crateify::Crateify;
mod decrateify;
pub use decrateify::Decrateify;
mod identify_aliases;
pub(crate) use identify_aliases::IdentifyAliases;
mod unused_params;
pub(crate) use unused_params::UnusedParams;

use std::env;
use proc_macro2::{Ident, Span};

fn calling_crate(span: Span) -> Ident {
    let value = env::var("CARGO_CRATE_NAME").unwrap();
    let value = value.replace('-', "_");
    Ident::new(value.as_str(), span)
}