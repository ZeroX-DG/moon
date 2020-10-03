pub struct Selector(Vec<(SimpleSelectorSequence, Option<Combinator>)>);

pub enum Combinator {
    Descendant,
    Child,
    NextSibling,
    SubsequentSibling
}

pub struct SimpleSelectorSequence(Vec<SimpleSelector>);

pub enum SimpleSelectorType {
    Type,
    Universal,
    Attribute,
    Class,
    ID,
    Pseudo
}

pub struct SimpleSelector {
    type_: SimpleSelectorType,
    value: String
}
