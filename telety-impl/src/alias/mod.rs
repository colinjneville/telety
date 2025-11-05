pub(crate) mod arguments;
pub(crate) use arguments::Arguments;
pub mod error;
pub use error::Error;
mod exact;
pub(crate) use exact::Exact;
mod index;
pub(crate) use index::Index;
mod kind;
pub use kind::Kind;
mod map;
pub use map::Map;
mod module;
pub use module::Module;
mod public;
pub(crate) use public::Public;
mod path;
pub(crate) use path::Path;

use quote::quote;
use syn::parse_quote;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Alias<'map> {
    pub(crate) map: &'map Map<'map>,
    pub(crate) path: &'map Path,
    pub(crate) index: Index,
    pub(crate) arguments: Arguments,
    pub(crate) kind: Kind,
}

impl<'map> Alias<'map> {
    pub(crate) fn new(
        map: &'map Map,
        path: &'map Path,
        index: Index,
        arguments: Arguments,
        kind: Kind,
    ) -> Self {
        Self {
            map,
            path,
            index,
            arguments,
            kind,
        }
    }

    // The original type path this alias points to, with generic arguments removed
    pub fn aliased_path(&self) -> &syn::Path {
        &self.path.truncated_path
    }

    // Path to the alias with no generic arguments. Does not include `!`.
    pub fn to_macro_path(&self) -> syn::Path {
        let path = self.map.map_path();
        let module = self.map.module().ident();
        let alias_ident = self.index.ident(self.path.friendly_path());

        parse_quote!(#path::#module::#alias_ident)
    }

    pub fn to_path(&self) -> syn::Path {
        let macro_path = self.to_macro_path();
        // Janky turbofish
        let arguments = self.arguments.args.as_ref().map(|a| quote!(::#a));

        parse_quote!(#macro_path #arguments)
    }

    pub fn generic_arguments(&self) -> Option<&syn::AngleBracketedGenericArguments> {
        self.arguments.args.as_ref()
    }

    pub fn kind(&self) -> Kind {
        self.kind
    }

    pub(crate) fn exact(self) -> Exact<'map> {
        Exact::new(self)
    }

    pub(crate) fn public(self) -> Public<'map> {
        Public::new(self)
    }
}
