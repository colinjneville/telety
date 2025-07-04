#![allow(dead_code)]

use telety::telety;

#[telety(crate::util::types)]
pub struct Simple(pub i32);

#[telety(crate::util::types)]
pub enum MyEnum {
    A,
    B(i32),
    C(i32, i64),
    D(Box<Self>),
    E(()),
    F(Option<u64>),
    G(std::vec::Vec<u8>),
}

#[telety(crate::util::types)]
pub enum MyGeneric<T> {
    A(T),
    B([T; 2]),
    C(Box<Self>),
    D(MyEmpty),
}

#[telety(crate::util::types)]
pub struct A(B, C);

#[telety(crate::util::types)]
pub struct B(i32);

pub struct C;

pub struct MyEmpty;

pub struct NoTelety;

pub trait Gat<'a, A> {
    type Ty<B>;
    type Lifetime<'b>;
    type Both<'b, B: 'b>;
}

impl<'a, A> Gat<'a, A> for i32 {
    type Ty<B> = Option<B>;
    type Lifetime<'b> = &'b i32;
    type Both<'b, B: 'b> = &'b B;
}

#[telety(crate::util::types)]
pub enum AssociatedTypes {
    A(<Option<i32> as IntoIterator>::Item),
    B(<i32 as Gat<'static, u32>>::Ty<i32>),
    C(<i32 as Gat<'static, u32>>::Lifetime<'static>),
    D(<i32 as Gat<'static, u32>>::Both<'static, i32>),
}

#[telety(crate::util::types)]
pub trait GenericParam<Param> {
    fn apply_item(param: Param) -> Param;
}
