use std::collections::HashMap;
use super::selector::Selector;

pub struct StyleRule {
    selector: Selector,
    style: StyleDeclaration
}

pub struct StyleDeclaration {
    properties: HashMap<String, String>
}
