use crate::parser::structs::Declaration;
use crate::selector::structs::{Selector, Specificity};

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

    pub fn specificity(&self) -> Specificity {
        let specificities = self
            .selectors
            .iter()
            .map(|selector| selector.specificity())
            .collect::<Vec<Specificity>>();

        specificities.into_iter().max().unwrap()
    }
}
