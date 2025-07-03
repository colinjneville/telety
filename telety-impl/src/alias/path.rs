use std::{cmp, hash, mem};

use syn::spanned::Spanned as _;

use crate::alias;

#[derive(Debug, Clone)]
pub(crate) struct Path {
    lifetime_count: usize,
    type_count: usize,
    const_count: usize,
    pub(crate) truncated_path: syn::Path,
}

impl Path {
    pub(crate) fn new(aliased_type: &syn::TypePath) -> Result<(Self, alias::Arguments), alias::Error> {
        let span = aliased_type.span();

        if let Some(_qself) = &aliased_type.qself {
            return Err(alias::Error::new(span, alias::error::Kind::AssociatedType));
        }

        let mut args = alias::Arguments::default();

        let mut truncated_path = aliased_type.path.clone();

        for segment in &mut truncated_path.segments {
            let segment_args = mem::take(&mut segment.arguments);

            if !mem::replace(&mut args, alias::Arguments::new(segment_args)?).is_empty() {
                // If there are arguments before the final segment, this must be an associated type
                return Err(alias::Error::new(span, alias::error::Kind::AssociatedType));
            }
        }

        let lifetime_count = args.lifetime_count;
        let type_count = args.type_count;
        let const_count = args.const_count;

        let path = Self {
            lifetime_count,
            type_count,
            const_count,
            truncated_path,
        };

        Ok((path, args))
    }
}

impl cmp::PartialEq for Path {
    fn eq(&self, other: &Self) -> bool {
        let Self {
            lifetime_count,
            type_count,
            const_count,
            truncated_path: aliased_type,
        } = self;

        lifetime_count == &other.lifetime_count
            && type_count == &other.type_count
            && const_count == &other.const_count
            && aliased_type == &other.truncated_path
    }
}

impl cmp::Eq for Path { }

impl hash::Hash for Path {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        let Self {
            lifetime_count,
            type_count,
            const_count,
            truncated_path: aliased_type,
        } = self;
        
        lifetime_count.hash(state);
        type_count.hash(state);
        const_count.hash(state);
        aliased_type.hash(state);
    }
}