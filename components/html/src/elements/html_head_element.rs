use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::node::Node;

pub struct HTMLHeadElement {
    html_element: HTMLElement
}

impl DOMObject for HTMLHeadElement {
    fn as_node(&self) -> &Node {
        self.html_element.as_node()
    }
}
