use syn::{
    Attribute, Fields, Generics, Ident, Item, ItemConst, ItemEnum, ItemExternCrate, ItemFn,
    ItemForeignMod, ItemImpl, ItemMacro, ItemMod, ItemStatic, ItemStruct, ItemTrait,
    ItemTraitAlias, ItemType, ItemUnion, ItemUse, Visibility,
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

#[allow(dead_code)]
pub(crate) trait ItemData {
    fn attrs(&self) -> &[Attribute];
    fn attrs_mut(&mut self) -> &mut Vec<Attribute>;

    fn vis(&self) -> Option<&Visibility>;

    fn ident(&self) -> Option<IdentData<&Ident>>;
    fn ident_mut(&mut self) -> Option<IdentData<&mut Ident>>;

    fn generics(&self) -> Option<&Generics>;
    fn generics_mut(&mut self) -> Option<&mut Generics>;
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

#[allow(dead_code)]
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
}
