use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLHeadElement {
}

impl HTMLHeadElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLHeadElement {
}

impl NodeHooks for HTMLHeadElement {}

impl ElementMethods for HTMLHeadElement {
    fn tag_name(&self) -> &'static str {
        "head"
    }
}
