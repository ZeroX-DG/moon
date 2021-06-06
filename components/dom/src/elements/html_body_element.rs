use crate::dom_ref::NodeRef;
use crate::impl_html_convert;

#[derive(Debug)]
pub struct HTMLBodyElement {
    node_ref: NodeRef,
}

impl HTMLBodyElement {
    pub fn new(node_ref: NodeRef) -> Self {
        Self { node_ref }
    }
}

impl_html_convert!(HTMLBodyElement);

