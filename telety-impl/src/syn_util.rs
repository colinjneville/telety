use syn::{
    punctuated::Punctuated, Expr, ExprPath, GenericArgument, GenericParam, Generics, Path,
    PathArguments, PathSegment, Token, Type, TypePath,
};

pub(crate) fn generic_params_to_arguments(
    params: &Generics,
) -> Punctuated<GenericArgument, Token![,]> {
    let mut args = Punctuated::new();
    for pair in params.params.pairs() {
        args.push_value(generic_param_to_argument(pair.value()));
        if let Some(&comma) = pair.punct() {
            args.push_punct(*comma);
        }
    }
    args
}

fn generic_param_to_argument(param: &GenericParam) -> GenericArgument {
    match param {
        GenericParam::Lifetime(lifetime) => GenericArgument::Lifetime(lifetime.lifetime.clone()),
        GenericParam::Type(ty) => {
            let mut segments = Punctuated::new();
            segments.push_value(PathSegment {
                ident: ty.ident.clone(),
                arguments: PathArguments::None,
            });
            GenericArgument::Type(Type::Path(TypePath {
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }))
        }
        GenericParam::Const(cnst) => {
            let mut segments = Punctuated::new();
            segments.push_value(PathSegment {
                ident: cnst.ident.clone(),
                arguments: PathArguments::None,
            });
            GenericArgument::Const(Expr::Path(ExprPath {
                attrs: vec![],
                qself: None,
                path: Path {
                    leading_colon: None,
                    segments,
                },
            }))
        }
    }
}
