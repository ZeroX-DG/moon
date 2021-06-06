use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLUnknownElement {
}

impl HTMLUnknownElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLUnknownElement {
}

impl NodeHooks for HTMLUnknownElement {}

impl ElementMethods for HTMLUnknownElement {
    fn tag_name(&self) -> &'static str {
        "unknown"
    }
}
