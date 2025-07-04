use proc_macro2::Span;

pub struct Error {
    span: Span,
    pub kind: Kind,
}

pub enum Kind {
    AssociatedType,
    Closure,
}

impl Kind {
    pub(crate) fn error(self, span: Span) -> Error {
        Error::new(span, self)
    }
}

impl Error {
    pub(crate) fn new(span: Span, kind: Kind) -> Self {
        Self { span, kind }
    }
}

impl From<Error> for syn::Error {
    fn from(value: Error) -> Self {
        let Error { span, kind } = value;

        let message = match kind {
            Kind::AssociatedType => "Associated types are not supported".to_string(),
            Kind::Closure => "Closure traits are not supported".to_string(),
        };

        syn::Error::new(span, message)
    }
}
