use dom::element::Element;
use dom::dom_ref::DOMObject;
use dom::node::Node;

pub mod html_html_element;
pub mod html_head_element;
pub mod html_body_element;
pub mod html_title_element;
pub mod html_div_element;

pub struct HTMLElement {
    element: Element
}

impl DOMObject for HTMLElement {
    fn as_node(&self) -> &Node {
        self.element.as_node()
    }
}
