mod apply_generic_arguments;
pub use apply_generic_arguments::ApplyGenericArguments;
mod apply_aliases;
pub use apply_aliases::ApplyAliases;
mod crateify;
pub use crateify::Crateify;
mod decrateify;
pub use decrateify::Decrateify;
pub mod identify_aliases;
pub use identify_aliases::IdentifyAliases;

use proc_macro2::{Ident, Span};
use std::env;

fn calling_crate(span: Span) -> Ident {
    let value = env::var("CARGO_CRATE_NAME").unwrap();
    let value = value.replace('-', "_");
    Ident::new(value.as_str(), span)
}
