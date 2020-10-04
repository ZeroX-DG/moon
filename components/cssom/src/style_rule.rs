use super::selector::Selector;
use std::collections::HashMap;
use smallbitvec::SmallBitVec;

pub struct StyleRule {
    selector: Selector,
    style: StyleDeclarations,
}

pub struct StyleDeclarations {
    properties: Vec<StyleDeclaration>,
    importance: SmallBitVec
}

pub enum StyleDeclaration {
    Background
}
