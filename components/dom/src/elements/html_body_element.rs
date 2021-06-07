use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLBodyElement {}

impl HTMLBodyElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLBodyElement {}

impl NodeHooks for HTMLBodyElement {}

impl ElementMethods for HTMLBodyElement {
    fn tag_name(&self) -> String {
        "body".to_string()
    }
}
