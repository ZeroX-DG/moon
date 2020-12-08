use super::BaseBox;
use std::ops::Deref;
use crate::base_box_deref_impls;

#[derive(Debug)]
pub struct BlockContainerBox {
    base: BaseBox
}

#[derive(Debug)]
pub struct AnonymousBlockBox {
    base: BaseBox
}

base_box_deref_impls!(BlockContainerBox);
base_box_deref_impls!(AnonymousBlockBox);
