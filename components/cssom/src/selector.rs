pub struct Selector(Vec<SimpleSelectorSequence>);

pub struct SimpleSelectorSequence(Vec<SimpleSelector>);

pub enum SimpleSelector {
    Type,
    Universal,
    Attribute,
    Class,
    ID,
    Pseudo
}
