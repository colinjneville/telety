use syn::{
    visit::Visit, visit_mut::VisitMut, Attribute, Fields, Generics, Ident, Item, ItemConst,
    ItemEnum, ItemExternCrate, ItemFn, ItemForeignMod, ItemImpl, ItemMacro, ItemMod, ItemStatic,
    ItemStruct, ItemTrait, ItemTraitAlias, ItemType, ItemUnion, ItemUse, Visibility,
};

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Namespaces: u8 {
        const None = 0b000;
        const Type = 0b001;
        const Value = 0b010;
        const Macro = 0b100;
    }
}

pub(crate) struct IdentData<T> {
    pub ident: T,
    pub namespaces: Namespaces,
}

impl<T> IdentData<T> {
    fn new(ident: T, namespaces: Namespaces) -> Self {
        Self { ident, namespaces }
    }
}

pub(crate) trait ItemData {
    fn attrs(&self) -> &[Attribute];
    fn attrs_mut(&mut self) -> &mut Vec<Attribute>;

    fn vis(&self) -> Option<&Visibility>;

    fn ident(&self) -> Option<IdentData<&Ident>>;
    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>>;

    fn generics(&self) -> Option<&Generics>;
    fn generics_mut(&mut self) -> Option<&mut Generics>;

    /// Apply a visitor to the subsets of the AST which is affected
    /// by `generics`.
    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>);
    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut);
}

impl ItemData for ItemConst {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::Value))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::Value))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        visitor.visit_type(&self.ty);
        visitor.visit_expr(&self.expr);
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        visitor.visit_type_mut(&mut self.ty);
        visitor.visit_expr_mut(&mut self.expr);
    }
}

impl ItemData for ItemEnum {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::Type))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::Type))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        for variant in &self.variants {
            visitor.visit_variant(variant);
        }
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        for variant in &mut self.variants {
            visitor.visit_variant_mut(variant);
        }
    }
}

impl ItemData for ItemExternCrate {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(
            self.rename.as_ref().map(|(_, i)| i).unwrap_or(&self.ident),
            Namespaces::None,
        ))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(
            self.rename
                .as_mut()
                .map(|(_, i)| i)
                .unwrap_or(&mut self.ident),
            Namespaces::None,
        ))
    }

    fn generics(&self) -> Option<&Generics> {
        None
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        None
    }

    fn visit_generics_scope<'ast>(&'ast self, _visitor: &mut dyn Visit<'ast>) {}

    fn visit_generics_scope_mut(&mut self, _visitor: &mut dyn VisitMut) {}
}

impl ItemData for ItemFn {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.sig.ident, Namespaces::Value))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.sig.ident, Namespaces::Value))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.sig.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.sig.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        for fn_arg in &self.sig.inputs {
            visitor.visit_fn_arg(fn_arg);
        }
        if let Some(variadic) = &self.sig.variadic {
            visitor.visit_variadic(variadic);
        }
        visitor.visit_return_type(&self.sig.output);

        visitor.visit_block(&self.block);
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        for fn_arg in &mut self.sig.inputs {
            visitor.visit_fn_arg_mut(fn_arg);
        }
        if let Some(variadic) = &mut self.sig.variadic {
            visitor.visit_variadic_mut(variadic);
        }
        visitor.visit_return_type_mut(&mut self.sig.output);

        visitor.visit_block_mut(&mut self.block);
    }
}

impl ItemData for ItemForeignMod {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        None
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        None
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        None
    }

    fn generics(&self) -> Option<&Generics> {
        None
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        None
    }

    fn visit_generics_scope<'ast>(&'ast self, _visitor: &mut dyn Visit<'ast>) {}

    fn visit_generics_scope_mut(&mut self, _visitor: &mut dyn VisitMut) {}
}

impl ItemData for ItemImpl {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        None
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        None
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        None
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        if let Some((_, trait_path, _)) = &self.trait_ {
            visitor.visit_path(trait_path);
        }

        visitor.visit_type(&self.self_ty);

        for item in &self.items {
            visitor.visit_impl_item(item);
        }
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        if let Some((_, trait_path, _)) = &mut self.trait_ {
            visitor.visit_path_mut(trait_path);
        }

        visitor.visit_type_mut(&mut self.self_ty);

        for item in &mut self.items {
            visitor.visit_impl_item_mut(item);
        }
    }
}

impl ItemData for ItemMacro {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        None
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        self.ident
            .as_ref()
            .map(|i| IdentData::new(i, Namespaces::Macro))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        self.ident
            .as_mut()
            .map(|i| IdentData::new(i, Namespaces::Macro))
    }

    fn generics(&self) -> Option<&Generics> {
        None
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        None
    }

    fn visit_generics_scope<'ast>(&'ast self, _visitor: &mut dyn Visit<'ast>) {}

    fn visit_generics_scope_mut(&mut self, _visitor: &mut dyn VisitMut) {}
}

impl ItemData for ItemMod {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::None))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::None))
    }

    fn generics(&self) -> Option<&Generics> {
        None
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        None
    }

    fn visit_generics_scope<'ast>(&'ast self, _visitor: &mut dyn Visit<'ast>) {}

    fn visit_generics_scope_mut(&mut self, _visitor: &mut dyn VisitMut) {}
}

impl ItemData for ItemStatic {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::Value))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::Value))
    }

    fn generics(&self) -> Option<&Generics> {
        None
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        None
    }

    fn visit_generics_scope<'ast>(&'ast self, _visitor: &mut dyn Visit<'ast>) {}

    fn visit_generics_scope_mut(&mut self, _visitor: &mut dyn VisitMut) {}
}

impl ItemData for ItemStruct {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        let namespaces = match self.fields {
            Fields::Unit => Namespaces::Type | Namespaces::Value,
            _ => Namespaces::Type,
        };
        Some(IdentData::new(&self.ident, namespaces))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        let namespaces = match self.fields {
            Fields::Unit => Namespaces::Type | Namespaces::Value,
            _ => Namespaces::Type,
        };
        Some(IdentData::new(&mut self.ident, namespaces))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        for field in &self.fields {
            visitor.visit_field(field);
        }
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        for field in &mut self.fields {
            visitor.visit_field_mut(field);
        }
    }
}

impl ItemData for ItemTrait {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::Type))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::Type))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        for supertrait in &self.supertraits {
            visitor.visit_type_param_bound(supertrait);
        }
        for item in &self.items {
            visitor.visit_trait_item(item);
        }
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        for supertrait in &mut self.supertraits {
            visitor.visit_type_param_bound_mut(supertrait);
        }
        for item in &mut self.items {
            visitor.visit_trait_item_mut(item);
        }
    }
}

impl ItemData for ItemTraitAlias {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::Type))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::Type))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        for bound in &self.bounds {
            visitor.visit_type_param_bound(bound);
        }
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        for bound in &mut self.bounds {
            visitor.visit_type_param_bound_mut(bound);
        }
    }
}

impl ItemData for ItemType {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::Type))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::Type))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        visitor.visit_type(&self.ty);
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        visitor.visit_type_mut(&mut self.ty);
    }
}

impl ItemData for ItemUnion {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        Some(IdentData::new(&self.ident, Namespaces::Type))
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        Some(IdentData::new(&mut self.ident, Namespaces::Type))
    }

    fn generics(&self) -> Option<&Generics> {
        Some(&self.generics)
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        Some(&mut self.generics)
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        visitor.visit_fields_named(&self.fields);
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        visitor.visit_fields_named_mut(&mut self.fields);
    }
}

impl ItemData for ItemUse {
    fn attrs(&self) -> &[Attribute] {
        &self.attrs
    }
    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        &mut self.attrs
    }

    fn vis(&self) -> Option<&Visibility> {
        Some(&self.vis)
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        None
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        None
    }

    fn generics(&self) -> Option<&Generics> {
        None
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        None
    }

    fn visit_generics_scope<'ast>(&'ast self, _visitor: &mut dyn Visit<'ast>) {}

    fn visit_generics_scope_mut(&mut self, _visitor: &mut dyn VisitMut) {}
}

fn item_variant_trait(item: &Item) -> &dyn ItemData {
    match item {
        Item::Const(i) => i,
        Item::Enum(i) => i,
        Item::ExternCrate(i) => i,
        Item::Fn(i) => i,
        Item::ForeignMod(i) => i,
        Item::Impl(i) => i,
        Item::Macro(i) => i,
        Item::Mod(i) => i,
        Item::Static(i) => i,
        Item::Struct(i) => i,
        Item::Trait(i) => i,
        Item::TraitAlias(i) => i,
        Item::Type(i) => i,
        Item::Union(i) => i,
        Item::Use(i) => i,
        _ => panic!("Unknown item type"),
    }
}

fn item_variant_trait_mut(item: &mut Item) -> &mut dyn ItemData {
    match item {
        Item::Const(i) => i,
        Item::Enum(i) => i,
        Item::ExternCrate(i) => i,
        Item::Fn(i) => i,
        Item::ForeignMod(i) => i,
        Item::Impl(i) => i,
        Item::Macro(i) => i,
        Item::Mod(i) => i,
        Item::Static(i) => i,
        Item::Struct(i) => i,
        Item::Trait(i) => i,
        Item::TraitAlias(i) => i,
        Item::Type(i) => i,
        Item::Union(i) => i,
        Item::Use(i) => i,
        _ => panic!("Unknown item type"),
    }
}

impl ItemData for Item {
    fn attrs(&self) -> &[Attribute] {
        item_variant_trait(self).attrs()
    }

    fn attrs_mut(&mut self) -> &mut Vec<Attribute> {
        item_variant_trait_mut(self).attrs_mut()
    }

    fn vis(&self) -> Option<&Visibility> {
        item_variant_trait(self).vis()
    }

    fn ident(&self) -> Option<IdentData<&Ident>> {
        item_variant_trait(self).ident()
    }

    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>> {
        item_variant_trait_mut(self).ident_mut()
    }

    fn generics(&self) -> Option<&Generics> {
        item_variant_trait(self).generics()
    }

    fn generics_mut(&mut self) -> Option<&mut Generics> {
        item_variant_trait_mut(self).generics_mut()
    }

    fn visit_generics_scope<'ast>(&'ast self, visitor: &mut dyn Visit<'ast>) {
        item_variant_trait(self).visit_generics_scope(visitor);
    }

    fn visit_generics_scope_mut(&mut self, visitor: &mut dyn VisitMut) {
        item_variant_trait_mut(self).visit_generics_scope_mut(visitor);
    }
}
