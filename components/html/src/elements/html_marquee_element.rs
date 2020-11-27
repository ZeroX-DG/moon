use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::element::Element;
use dom::node::Node;
use std::any::Any;

#[allow(dead_code)]
pub struct HTMLMarqueeElement {
    html_element: HTMLElement,
    behavior: String,
    bg_color: String,
    direction: String,
    height: String,
    hspace: String,
    loop_: usize,
    scroll_amount: usize,
    scroll_delay: usize,
    true_speed: bool,
    vspace: usize,
    width: String
}

impl core::fmt::Debug for HTMLMarqueeElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self.html_element)
    }
}

impl HTMLMarqueeElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            html_element,
            behavior: String::new(),
            bg_color: String::new(),
            direction: String::new(),
            height: String::new(),
            hspace: String::new(),
            loop_: 0,
            scroll_amount: 0,
            scroll_delay: 0,
            true_speed: true,
            vspace: 0,
            width: String::new()
        }
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
