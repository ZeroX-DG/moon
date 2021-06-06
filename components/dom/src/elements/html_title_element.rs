use crate::dom_ref::NodeRef;
use crate::impl_html_convert;

#[derive(Debug)]
pub struct HTMLTitleElement {
    node_ref: NodeRef,
}

impl HTMLTitleElement {
    pub fn new(node_ref: NodeRef) -> Self {
        Self { node_ref }
    }

    pub fn text(&self) -> String {
        self.node_ref.borrow().child_text_content()
    }
}

impl_html_convert!(HTMLTitleElement);

