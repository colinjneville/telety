use syn::{Generics, Type};

#[derive(Debug)]
pub(crate) struct Details {
    pub(crate) parameters: Generics,
    pub(crate) aliased_type: Type,
}

impl Details {
    pub(crate) fn new(parameters: Generics, aliased_type: Type) -> Self {
        Self {
            parameters,
            aliased_type,
        }
    }
}
