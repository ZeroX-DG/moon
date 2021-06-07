use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLUnknownElement {
    tag_name: String
}

impl HTMLUnknownElement {
    pub fn new(tag_name: String) -> Self {
        Self {
            tag_name
        }
    }
}

impl ElementHooks for HTMLUnknownElement {
}

impl NodeHooks for HTMLUnknownElement {}

impl ElementMethods for HTMLUnknownElement {
    fn tag_name(&self) -> String {
        self.tag_name.clone()
    }
}
