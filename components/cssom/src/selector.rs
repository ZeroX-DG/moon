pub struct Selector(Vec<SimpleSelectorSequence>);

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
