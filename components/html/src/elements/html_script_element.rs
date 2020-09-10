use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::node::Node;
use std::any::Any;

pub struct HTMLScriptElement {
    html_element: HTMLElement,
    src: String,
    type_: String,
    non_blocking: bool
}

impl HTMLScriptElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            html_element,
            src: String::new(),
            type_: String::new(),
            non_blocking: true
        }
    }

    pub fn set_non_blocking(&mut self, value: bool) {
        self.set_non_blocking(value);
    }
}

impl DOMObject for HTMLScriptElement {
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
