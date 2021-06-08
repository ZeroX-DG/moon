use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLDivElement {}

impl HTMLDivElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLDivElement {}

impl NodeHooks for HTMLDivElement {}

impl ElementMethods for HTMLDivElement {
    fn tag_name(&self) -> String {
        "div".to_string()
    }
}
