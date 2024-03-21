mod definition;
pub(crate) use definition::Definition;
mod details;
pub(crate) use details::Details;
mod index;
pub use index::Index;
mod map;
pub use map::Map;
mod group;
pub(crate) use group::Group;

use syn::{
    punctuated::Punctuated, AngleBracketedGenericArguments, GenericArgument, GenericParam, Generics, Path, PathArguments, PathSegment, Token, Type, TypePath
};

use crate::syn_util;

/// The details of a type alias.
#[derive(Debug, Clone, Copy)]
pub struct Alias<'m> {
    map: &'m Map,
    index: Index,
    details: &'m Details,
}

impl<'m> Alias<'m> {
    pub(crate) fn new(map: &'m Map, index: Index, details: &'m Details) -> Self {
        Self { map, index, details }
    }

    /// The [Index] of this alias.
    pub fn index(&self) -> Index {
        self.index
    }

    /// Is this alias for a type parameter?  
    /// Only lone type parameters are included (i.e. `T`, but not `Vec<T>`).  
    pub fn is_generic_parameter_type(&self) -> bool {
        if let Type::Path(type_path) = &self.details.aliased_type {
            let mut iter = self.details.parameters.params.iter();
            if let (Some(GenericParam::Type(single)), None) = (iter.next(), iter.next()) {
                return type_path.path.is_ident(&single.ident);
            }
        }
        false
    }

    /// The generic parameters required for this type alias.  
    /// For example, this item has two type parameters:  
    /// ```rust,ignore
    /// struct Either<A, B> {
    ///     A(Box<A>),
    ///     B(Box<B>),
    /// }
    /// ```
    /// But reusing these parameters on the alias to the type in variant `A` will fail,
    /// as parameter `B` is unused by the aliased type:
    /// ```rust,ignore
    /// type Alias<A, B> = Box<A>;
    /// ```
    /// ```text,ignore
    /// type parameter `B` is unused
    /// ```
    /// This method returns only the generic parameters used by the alias.
    pub fn parameters(&self) -> &Generics {
        &self.details.parameters
    }

    /// Returns this alias formatted as a [PathSegment], without generic parameters
    pub fn path_segment_no_generics(&self) -> PathSegment {
        PathSegment {
            ident: self.index.ident(),
            arguments: PathArguments::None,
        }
    }

    /// Returns this alias formatted as a [PathSegment], with unsubstituted generic
    /// arguments if applicable.
    pub fn path_segment(&self) -> PathSegment {
        let mut segment = self.path_segment_no_generics();
        segment.arguments = PathArguments::AngleBracketed(AngleBracketedGenericArguments {
            colon2_token: None,
            lt_token: Default::default(),
            args: syn_util::generic_params_to_arguments(self.parameters()),
            gt_token: Default::default(),
        });
        segment
    }

    /// The original type being aliased
    pub fn aliased_type(&self) -> &Type {
        &self.details.aliased_type
    }

    /// The type arguments on the original aliased type.  
    /// e.g. `i32, u64` for `my_crate::MyType<i32, i64>`
    pub fn aliased_type_arguments(&self) -> Option<&Punctuated<GenericArgument, Token![,]>> {
        if let Type::Path(type_path) = self.aliased_type() {
            if let Some(last_segment) = type_path.path.segments.last() {
                if let PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments {
                    return Some(&angle_bracketed.args);
                }
            }
        }
        None
    }

    /// Creates a qualified [Path] to the alias with no generic arguments
    pub fn path(&self) -> Path {
        let mut path = self.map.group().path();
        path.segments.push(self.path_segment_no_generics());
        path
    }

    /// Creates a qualified [TypePath] to the alias, with generic arguments.
    pub fn type_path(&self) -> TypePath {
        let mut path = self.map.group().path();
        path.segments.push(self.path_segment());
        TypePath { qself: None, path }
    }

    /// Creates a qualified [Type] of the alias.
    pub fn ty(&self) -> Type {
        Type::Path(self.type_path())
    }
}
