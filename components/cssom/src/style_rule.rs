use std::collections::HashMap;
use super::selector::Selector;

pub struct StyleRule {
    selector: Selector,
    style: StyleDeclarations
}

pub struct StyleDeclarations {
    properties: HashMap<String, String>
}
