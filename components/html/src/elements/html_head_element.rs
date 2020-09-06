use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::node::Node;
use std::any::Any;

pub struct HTMLHeadElement {
    html_element: HTMLElement
}

impl DOMObject for HTMLHeadElement {
    fn as_node(&self) -> &Node {
        self.html_element.as_node()
    }

    fn as_node_mut(&mut self) -> &mut Node {
        self.html_element.as_node_mut()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
