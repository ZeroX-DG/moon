use super::{BaseBox, LayoutBox};
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

impl BlockContainerBox {
    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }
}

impl AnonymousBlockBox {
    pub fn add_child(&mut self, child: LayoutBox) {
        self.children.push(child);
    }
}

base_box_deref_impls!(BlockContainerBox);
base_box_deref_impls!(AnonymousBlockBox);
