use crate::selector::structs::Selector;
use smallbitvec::SmallBitVec;

pub struct StyleRule {
    selector: Selector,
    style: StyleDeclarations,
}

pub struct StyleDeclarations {
    properties: Vec<StyleDeclaration>,
    importances: SmallBitVec
}

pub enum StyleDeclaration {
    Background
}
