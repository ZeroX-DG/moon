use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLTitleElement {
}

impl HTMLTitleElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLTitleElement {
}

impl NodeHooks for HTMLTitleElement {}

impl ElementMethods for HTMLTitleElement {
    fn tag_name(&self) -> String {
        "title".to_string()
    }
}
