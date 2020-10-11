use crate::parser::structs::Declaration;
use crate::selector::structs::Selector;

#[derive(Debug, PartialEq)]
pub struct StyleRule {
    pub selectors: Vec<Selector>,
    pub declarations: Vec<Declaration>,
}

impl StyleRule {
    pub fn new(selectors: Vec<Selector>, declarations: Vec<Declaration>) -> Self {
        Self {
            selectors,
            declarations,
        }
    }
}
