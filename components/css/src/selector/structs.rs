use std::cmp::{Ord, Ordering};

pub type SelectorData = Vec<(SimpleSelectorSequence, Option<Combinator>)>;

#[derive(Debug, PartialEq)]
pub struct Selector(SelectorData);

#[derive(Debug, PartialEq)]
pub enum Combinator {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling,
}

#[derive(Debug, PartialEq)]
pub struct SimpleSelectorSequence(Vec<SimpleSelector>);

#[derive(Debug, PartialEq)]
pub enum SimpleSelectorType {
    Type,
    Universal,
    Attribute,
    Class,
    ID,
    Pseudo,
}

#[derive(Debug, PartialEq)]
pub struct SimpleSelector {
    type_: SimpleSelectorType,
    value: Option<String>,
}

/// CSS Selector specificity
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd)]
pub struct Specificity(u32, u32, u32);

impl Specificity {
    pub fn new(a: u32, b: u32, c: u32) -> Self {
        Self(a, b, c)
    }
}

impl Ord for Specificity {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.cmp(&other.0) {
            Ordering::Greater => Ordering::Greater,
            Ordering::Less => Ordering::Less,
            Ordering::Equal => match self.1.cmp(&other.1) {
                Ordering::Greater => Ordering::Greater,
                Ordering::Less => Ordering::Less,
                Ordering::Equal => match self.2.cmp(&other.2) {
                    Ordering::Greater => Ordering::Greater,
                    Ordering::Less => Ordering::Less,
                    Ordering::Equal => Ordering::Equal,
                },
            },
        }
    }
}

impl Selector {
    pub fn new(data: SelectorData) -> Self {
        Self(data)
    }

    pub fn values(&self) -> &SelectorData {
        &self.0
    }

    pub fn specificity(&self) -> Specificity {
        let (a, b, c) = self.values().iter().fold((0, 0, 0), |acc, (selector, _)| {
            let specificity = selector.specificity();
            (
                acc.0 + specificity.0,
                acc.1 + specificity.1,
                acc.2 + specificity.2,
            )
        });
        Specificity::new(a, b, c)
    }
}

impl SimpleSelectorSequence {
    pub fn new(data: Vec<SimpleSelector>) -> Self {
        Self(data)
    }

    pub fn values(&self) -> &Vec<SimpleSelector> {
        &self.0
    }

    pub fn specificity(&self) -> Specificity {
        let (a, b, c) =
            self.values()
                .iter()
                .fold((0, 0, 0), |acc, curr| match curr.selector_type() {
                    SimpleSelectorType::ID => (acc.0 + 1, acc.1, acc.2),
                    SimpleSelectorType::Class | SimpleSelectorType::Attribute => {
                        (acc.0, acc.1 + 1, acc.2)
                    }
                    SimpleSelectorType::Type => (acc.0, acc.1, acc.2 + 1),
                    _ => acc,
                });
        Specificity(a, b, c)
    }
}

impl SimpleSelector {
    pub fn new(type_: SimpleSelectorType, value: Option<String>) -> Self {
        Self { type_, value }
    }

    pub fn value(&self) -> &Option<String> {
        &self.value
    }

    pub fn selector_type(&self) -> &SimpleSelectorType {
        &self.type_
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_specificity() {
        let a = Specificity::new(0, 0, 0);
        let b = Specificity::new(0, 0, 1);
        assert!(a < b);
    }
}
