use super::BaseBox;
use std::ops::Deref;
use crate::base_box_deref_impls;

#[derive(Debug)]
pub struct InlineBox {
    base: BaseBox
}

#[derive(Debug)]
pub struct AtomicInlineLevelBox {
    base: BaseBox
}

#[derive(Debug)]
pub struct AnonymousInlineBox {
    base: BaseBox
}

base_box_deref_impls!(InlineBox);
base_box_deref_impls!(AtomicInlineLevelBox);
base_box_deref_impls!(AnonymousInlineBox);
