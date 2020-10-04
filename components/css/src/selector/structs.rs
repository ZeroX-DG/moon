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

impl Selector {
    pub fn new(data: SelectorData) -> Self {
        Self(data)
    }
}

impl SimpleSelectorSequence {
    pub fn new(data: Vec<SimpleSelector>) -> Self {
        Self(data)
    }
}

impl SimpleSelector {
    pub fn new(type_: SimpleSelectorType, value: Option<String>) -> Self {
        Self { type_, value }
    }
}
