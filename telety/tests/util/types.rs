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
