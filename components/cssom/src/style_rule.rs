use super::selector::Selector;
use std::collections::HashMap;

pub struct StyleRule {
    selector: Selector,
    style: StyleDeclarations,
}

pub struct StyleDeclarations {
    properties: HashMap<String, String>,
}
