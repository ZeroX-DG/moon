use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLAnchorElement {
}

impl HTMLAnchorElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLAnchorElement {
}

impl NodeHooks for HTMLAnchorElement {}

impl ElementMethods for HTMLAnchorElement {
    fn tag_name(&self) -> &'static str {
        "a"
    }
}
