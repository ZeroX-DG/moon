use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::node::Node;

pub struct HTMLDivElement {
    html_element: HTMLElement
}

impl DOMObject for HTMLDivElement {
    fn as_node(&self) -> &Node {
        self.html_element.as_node()
    }
}
