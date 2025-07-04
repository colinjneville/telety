use quote::{format_ident, quote};
use syn::spanned::Spanned as _;

#[derive(Debug)]
pub struct Module {
    visibility: syn::Visibility,
    ident: syn::Ident,
}

impl Module {
    pub fn from_named_item(item: &syn::Item) -> syn::Result<Self> {
        let (visibility, ident) = match item {
            syn::Item::Enum(item_enum) => (&item_enum.vis, &item_enum.ident),
            syn::Item::Struct(item_struct) => (&item_struct.vis, &item_struct.ident),
            syn::Item::Union(item_union) => (&item_union.vis, &item_union.ident),
            syn::Item::Trait(item_trait) => (&item_trait.vis, &item_trait.ident),
            _ => {
                return Err(syn::Error::new(
                    item.span(),
                    "Only enums, structs, unions, and traits are currently supported",
                ));
            }
        };

        let visibility = visibility.clone();
        let ident = format_ident!("__telety_alias_map_{ident}");
        Ok(Self { visibility, ident })
    }

    pub fn visibility(&self) -> &syn::Visibility {
        &self.visibility
    }

    pub fn ident(&self) -> &syn::Ident {
        &self.ident
    }

    pub fn new_child(&self, suffix: &str) -> Self {
        let Self { visibility, ident } = self;
        let visibility = visibility.clone();
        let ident = format_ident!("{ident}__{suffix}");

        Self { visibility, ident }
    }

    pub fn with_contents<C: quote::ToTokens>(&self, contents: &C) -> impl quote::ToTokens + use<C> {
        let Self { visibility, ident } = self;

        quote! {
            #[doc(hidden)]
            #[allow(dead_code)]
            #[allow(unused_macros)]
            #[allow(unused_imports)]
            #[allow(non_snake_case)]
            #[allow(non_camel_case_types)]
            #[allow(non_local_definitions)]
            #visibility mod #ident {
                #contents
            }
        }
    }
}
