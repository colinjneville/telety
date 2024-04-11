use quote::format_ident;
use syn::{
    spanned::Spanned, visit::Visit, AngleBracketedGenericArguments, Attribute, GenericArgument,
    Ident, Item, Path, PathArguments, PathSegment, Type, TypePath, Visibility,
};

use crate::{
    alias,
    item_data::{ItemData as _, Namespaces},
    syn_util, visitor, Alias, Options,
};

/// Wraps an [Item] which has the `#[telety]` attribute to provide additional information
/// allowing it to be reflected outside its original context.
pub struct Telety<'item> {
    options: Options,
    item: &'item Item,
    alias_map: alias::Map,
    macro_ident: Ident,
    unique_ident: Ident,
    visibility: Visibility,
}

impl<'item> Telety<'item> {
    #[doc(hidden)]
    pub fn new_with_options(item: &'item Item, options: Options) -> syn::Result<Self> {
        if let Some(ident) = item.ident() {
            if ident.namespaces.contains(Namespaces::Macro) {
                return Err(syn::Error::new(
                    item.span(),
                    "Cannot be applied to items in the macro namespace",
                ));
            }
        }

        let Some(macro_ident) = options
            .macro_ident
            .as_ref()
            .or(item.ident().map(|i| i.ident))
            .cloned()
        else {
            return Err(syn::Error::new(
                item.span(),
                "Items without an identifier require a 'macro_ident' argument",
            ));
        };

        let Some(visibility) = options.visibility.as_ref().or(item.vis()).cloned() else {
            return Err(syn::Error::new(
                item.span(),
                "Items without a visibility require a 'visibility' argument",
            ));
        };

        let unique_ident = Self::make_unique_ident(&macro_ident, &options);

        let parameters = item.generics().cloned().unwrap_or_default();

        let self_type: Type = {
            let arguments = PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                colon2_token: None,
                lt_token: Default::default(),
                args: syn_util::generic_params_to_arguments(&parameters),
                gt_token: Default::default(),
            });
            // Use the global path to alert user if the containing_path is incorrect
            let mut path = options.converted_containing_path();
            path.segments.push(PathSegment {
                ident: item.ident().unwrap().ident.clone(),
                arguments,
            });

            Type::Path(TypePath { qself: None, path })
        };

        let alias_group = alias::Group::new(
            options.converted_containing_path(),
            format_ident!("__{unique_ident}"),
        );

        let mut alias_map = alias::Map::new(alias_group, parameters.clone(), self_type);

        // Identify all unique non-type parameter types and give them an index
        // (based on order of appearance), stored in our map
        let mut identify_visitor = visitor::IdentifyAliases::new(&mut alias_map);
        identify_visitor.visit_item(item);

        Ok(Self {
            options,
            item,
            alias_map,
            macro_ident,
            unique_ident,
            visibility,
        })
    }

    /// Generate telety information for the [Item].  
    /// The item must have a proper `#[telety(...)]` attribute.  
    /// Usually this item will come from the telety-generated macro with the same name as the item.
    pub fn new(item: &'item Item) -> syn::Result<Self> {
        let options = Options::from_attrs(item.attrs())?;

        Self::new_with_options(item, options)
    }

    pub fn options(&self) -> &Options {
        &self.options
    }

    #[doc(hidden)]
    pub fn visibility(&self) -> &Visibility {
        &self.visibility
    }

    #[doc(hidden)]
    pub fn module_ident(&self) -> &Ident {
        self.alias_map.module_ident()
    }

    // /// Provides the [alias::Map] for this item, which describes the mapping
    // /// of types appearing in the item to the aliases created for them.
    // pub fn alias_map(&self) -> &alias::Map {
    //     &self.alias_map
    // }

    pub fn self_alias(&self) -> Alias {
        self.alias_map.self_alias()
    }

    pub fn alias_of(&self, ty: &Type) -> Option<Alias> {
        self.alias_map.alias_of(ty)
    }

    pub fn iter_aliases(&self) -> impl Iterator<Item = Alias> {
        self.alias_map.iter()
    }

    /// Create a visitor which can be used to replace all types found in this map
    /// with their aliases.
    pub fn aliases_visitor(&self) -> visitor::ApplyAliases {
        visitor::ApplyAliases::new(&self.alias_map, false)
    }

    /// Create a visitor which substitutes generic parameters as if this type were monomorphized
    /// with the provided generic arguments.  
    /// For example, if we have a type:
    /// ```rust,ignore
    /// #[telety(crate)]
    /// struct S<T, U, V = T>(T, U, V);
    /// ```
    /// and provided the arguments `[i32, u64]`,
    /// the visitor would replace types `T` with `i32`,
    /// `U` with `u64`, and `V` with `i32`.  
    /// See [syn::visit_mut].
    pub fn generics_visitor<'a>(
        &self,
        generic_arguments: impl IntoIterator<Item = &'a GenericArgument>,
    ) -> syn::Result<visitor::ApplyGenericArguments> {
        let Some(parameters) = self.item.generics() else {
            return Err(syn::Error::new(
                self.item.span(),
                "Item kind does not have generic parameters",
            ));
        };

        visitor::ApplyGenericArguments::new(parameters, generic_arguments)
    }

    fn make_unique_ident(macro_ident: &Ident, options: &Options) -> Ident {
        let mut iter = options.module_path.segments.iter();
        let mut unique_ident = iter
            .next()
            .expect("Path must have at least one segment")
            .ident
            .clone();
        for segment in iter {
            let i = &segment.ident;
            unique_ident = format_ident!("{unique_ident}_{i}");
        }
        format_ident!("{unique_ident}_{macro_ident}")
    }

    /// The [Item] this describes
    pub fn item(&self) -> &Item {
        self.item
    }

    /// The [Path] to the item, using the crate name, not the `crate::` qualifier,
    /// and no arguments on the item.
    pub fn path(&self) -> Path {
        let mut path = self.options.module_path.clone();
        if let Some(ident) = self.item.ident() {
            path.segments.push(PathSegment {
                ident: ident.ident.clone(),
                arguments: PathArguments::None,
            });
        }
        path
    }

    /// The [Attribute]s on the [Item]
    pub fn attributes(&self) -> &[Attribute] {
        self.item.attrs()
    }

    /// The [Path] of the module containing this [Item].
    /// Provided by argument to the telety attribute.
    pub fn containing_mod_path(&self) -> Path {
        self.options.converted_containing_path()
    }

    pub fn macro_ident(&self) -> &Ident {
        &self.macro_ident
    }

    /// A (reasonably) unique [struct@Ident] for this item.  
    pub fn unique_ident(&self) -> &Ident {
        &self.unique_ident
    }
}
