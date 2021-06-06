use super::ElementHooks;
use super::ElementMethods;
use crate::node::NodeHooks;

#[derive(Debug)]
pub struct HTMLHtmlElement {
}

impl HTMLHtmlElement {
    pub fn empty() -> Self {
        Self {}
    }
}

impl ElementHooks for HTMLHtmlElement {
}

impl NodeHooks for HTMLHtmlElement {}

impl ElementMethods for HTMLHtmlElement {
    fn tag_name(&self) -> &'static str {
        "html"
    }
}
