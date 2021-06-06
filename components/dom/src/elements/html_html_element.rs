use crate::impl_html_convert;
use crate::dom_ref::NodeRef;

#[derive(Debug)]
pub struct HTMLHtmlElement {
    node_ref: NodeRef,
}

impl HTMLHtmlElement {
    pub fn new(node_ref: NodeRef) -> Self {
        Self { node_ref }
    }
}

impl_html_convert!(HTMLHtmlElement);

