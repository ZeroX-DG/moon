use dom::element::Element;
use dom::dom_ref::DOMObject;
use dom::node::Node;
use std::any::Any;

pub mod html_html_element;
pub mod html_head_element;
pub mod html_body_element;
pub mod html_title_element;
pub mod html_div_element;

pub trait DerivedFromHtml {
    fn as_element(&self) -> &Element;
}

pub struct HTMLElement {
    element: Element
}

impl DOMObject for HTMLElement {
    fn as_node(&self) -> &Node {
        self.element.as_node()
    }

    fn as_node_mut(&mut self) -> &mut Node {
        self.element.as_node_mut()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

