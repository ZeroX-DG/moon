use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::element::Element;
use dom::node::Node;
use std::any::Any;

pub struct HTMLMarqueeElement {
    html_element: HTMLElement,
}

impl core::fmt::Debug for HTMLMarqueeElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self.html_element)
    }
}

impl HTMLMarqueeElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self { html_element }
    }
}

impl DOMObject for HTMLMarqueeElement {
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

    fn as_element(&self) -> Option<&Element> {
        Some(&self.html_element.element)
    }

    fn as_element_mut(&mut self) -> Option<&mut Element> {
        Some(&mut self.html_element.element)
    }
}
