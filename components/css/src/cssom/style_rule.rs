use crate::selector::structs::Selector;
use crate::parser::structs::Declaration;

#[derive(Debug)]
pub struct StyleRule {
    selectors: Vec<Selector>,
    declarations: Vec<Declaration>,
}

impl StyleRule {
    pub fn new(selectors: Vec<Selector>, declarations: Vec<Declaration>) -> Self {
        Self {
            selectors,
            declarations
        }
    }
}
