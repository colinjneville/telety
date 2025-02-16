mod exact;
pub(crate) use exact::Exact;
mod index;
pub(crate) use index::Index;
mod map;
pub use map::Map;
mod module;
pub use module::Module;
mod public;
pub(crate) use public::Public;
mod target;
use syn::parse_quote;
pub(crate) use target::Target;

#[derive(Debug, Clone, Copy)]
pub struct Alias<'map> {
    pub(crate) map: &'map Map<'map>, 
    pub(crate) target: &'map Target, 
    pub(crate) index: Index,
}

impl<'map> Alias<'map> {
    pub(crate) fn new(map: &'map Map, target: &'map Target, index: Index) -> Self {
        Self {
            map,
            target,
            index,
        }
    }

    pub fn qualified_path(&self) -> syn::Path {
        let map_path = self.map.map_path();
        let module_ident = self.map.module().ident();
        let ident = self.index.ident();
        parse_quote!(#map_path::#module_ident::#ident)
    }

    pub fn qualified_type_path(&self) -> syn::TypePath {
        let path = self.qualified_path();
        let(_, type_generics, _) = self.target.generics.split_for_impl();
        parse_quote!(#path #type_generics)
    }

    pub fn qualified_type(&self) -> syn::Type {
        syn::Type::Path(self.qualified_type_path())
    }

    /// The type arguments on the original aliased type.  
    /// e.g. `i32, u64` for `my_crate::MyType<i32, i64>`
    pub fn aliased_type_arguments(&self) -> Option<&syn::punctuated::Punctuated<syn::GenericArgument, syn::Token![,]>> {
        if let Some(last_segment) = self.aliased_type().path.segments.last() {
            if let syn::PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments {
                return Some(&angle_bracketed.args);
            }
        }
        
        None
    }

    /// Is this alias for a type parameter?  
    /// Only lone type parameters are included (i.e. `T`, but not `Vec<T>`).  
    pub(crate) fn is_generic_parameter_type(&self) -> bool {
        let type_path = &self.target.aliased_type;
        let mut iter = self.target.generics.params.iter();

        if let (Some(syn::GenericParam::Type(single)), None) = (iter.next(), iter.next()) {
            type_path.path.is_ident(&single.ident)
        } else {
            false
        }
    }

    pub fn aliased_type(&self) -> &syn::TypePath {
        &self.target.aliased_type
    }

    pub(crate) fn exact(self) -> Exact<'map> {
        Exact::new(self)
    }

    pub(crate) fn public(self) -> Public<'map> {
        Public::new(self)
    }
}

