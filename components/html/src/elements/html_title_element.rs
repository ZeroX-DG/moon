use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::node::Node;

pub struct HTMLTitleElement {
    html_element: HTMLElement,
    text: String
}

impl DOMObject for HTMLTitleElement {
    fn as_node(&self) -> &Node {
        self.html_element.as_node()
    }
}
