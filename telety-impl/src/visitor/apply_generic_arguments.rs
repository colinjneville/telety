use std::collections::HashMap;

use syn::{
    parse_quote_spanned,
    spanned::Spanned as _,
    Expr, GenericArgument, GenericParam, Generics, Ident, Lifetime, Type,
};

pub struct ApplyGenericArguments<'p> {
    lifetimes: HashMap<&'p Lifetime, Option<Lifetime>>,
    types: HashMap<&'p Ident, Type>,
    consts: HashMap<&'p Ident, Expr>,
}

impl<'p> ApplyGenericArguments<'p> {
    pub fn new<'a>(
        params: &'p Generics,
        args: impl IntoIterator<Item = &'a GenericArgument>,
    ) -> syn::Result<Self> {
        // Default values can refer to preceding arguments (e.g. `<T, U = T>`),
        // so we need to run replacement on those defaults when
        // we encounter them, using the mapping we have built so far.
        let mut v = Self {
            lifetimes: HashMap::new(),
            types: HashMap::new(),
            consts: HashMap::new(),
        };

        let mut args_iter = args.into_iter().peekable();
        for param in &params.params {
            match param {
                GenericParam::Lifetime(param_lifetime) => {
                    if let Some(GenericArgument::Lifetime(arg_lifetime)) = args_iter.peek() {
                        v.lifetimes
                            .insert(&param_lifetime.lifetime, Some(arg_lifetime.clone()));
                    } else {
                        // TODO This isn't correct in locations where the parameter is used multiple times
                        v.lifetimes.insert(&param_lifetime.lifetime, None);
                    }
                }
                GenericParam::Type(param_type) => {
                    if let Some(arg) = args_iter.next() {
                        if let GenericArgument::Type(arg_type) = arg {
                            v.types.insert(&param_type.ident, arg_type.clone());
                        } else {
                            return Err(syn::Error::new(arg.span(), "Expected a type argument"));
                        }
                    } else if let Some(param_default) = &param_type.default {
                        let mut param_default = param_default.clone();
                        directed_visit::visit_mut(
                            &mut directed_visit::syn::direct::FullDefault,
                            &mut v,
                            &mut param_default,
                        );
                        v.types.insert(&param_type.ident, param_default);
                    } else {
                        return Err(syn::Error::new(
                            param_type.span(),
                            "Expected an argument for parameter",
                        ));
                    }
                }
                GenericParam::Const(param_const) => {
                    if let Some(arg) = args_iter.next() {
                        if let GenericArgument::Const(arg_const) = arg {
                            v.consts.insert(&param_const.ident, arg_const.clone());
                        } else {
                            return Err(syn::Error::new(arg.span(), "Expected a const argument"));
                        }
                    } else if let Some(param_default) = &param_const.default {
                        let mut param_default = param_default.clone();
                        directed_visit::visit_mut(
                            &mut directed_visit::syn::direct::FullDefault,
                            &mut v,
                            &mut param_default,
                        );
                        v.consts.insert(&param_const.ident, param_default);
                    } else {
                        return Err(syn::Error::new(
                            param_const.span(),
                            "Expected an argument for parameter",
                        ));
                    }
                }
            }
        }

        Ok(v)
    }
}

impl<'p> directed_visit::syn::visit::FullMut for ApplyGenericArguments<'p> {
    fn visit_lifetime_mut<D>(visitor: directed_visit::Visitor<'_, D, Self>, node: &mut syn::Lifetime)
    where 
        D: directed_visit::DirectMut<Self, syn::Lifetime> + ?Sized, 
    {
        if let Some(lifetime_arg) = visitor.lifetimes.get(node) {
            if let Some(lifetime_arg) = lifetime_arg {
                *node = lifetime_arg.clone();
            } else {
                let span = node.span();
                *node = parse_quote_spanned! { span => '_ };
            }
            return;
        }

        directed_visit::Visitor::visit_mut(visitor, node);
    }

    fn visit_type_mut<D>(visitor: directed_visit::Visitor<'_, D, Self>, node: &mut syn::Type)
    where 
        D: directed_visit::DirectMut<Self, syn::Type> + ?Sized, 
    {
        if let Type::Path(path) = node {
            // TODO should check first segment to support some associated types
            if let Some(ident) = path.path.get_ident() {
                if let Some(value) = visitor.types.get(ident) {
                    *node = value.clone();
                    return;
                }
            }
        }

        directed_visit::Visitor::visit_mut(visitor, node);
    }

    fn visit_expr_mut<D>(visitor: directed_visit::Visitor<'_, D, Self>, node: &mut syn::Expr)
    where 
        D: directed_visit::DirectMut<Self, syn::Expr> + ?Sized, 
    {
        if let Expr::Path(path) = node {
            if let Some(ident) = path.path.get_ident() {
                if let Some(value) = visitor.consts.get(ident) {
                    *node = value.clone();
                    return;
                }
            }
        }

        directed_visit::Visitor::visit_mut(visitor, node);
    }
}
