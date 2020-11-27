use dom::dom_ref::DOMObject;
use dom::element::Element;
use dom::node::Node;
use std::any::Any;
use std::ops::Deref;

mod html_base_element;
mod html_body_element;
mod html_div_element;
mod html_head_element;
mod html_html_element;
mod html_script_element;
mod html_title_element;
mod html_marquee_element;
mod html_anchor_element;

pub use html_base_element::*;
pub use html_body_element::*;
pub use html_div_element::*;
pub use html_head_element::*;
pub use html_html_element::*;
pub use html_script_element::*;
pub use html_title_element::*;
pub use html_marquee_element::*;
pub use html_anchor_element::*;

pub struct HTMLElement {
    element: Element,
}

impl HTMLElement {
    pub fn new(element: Element) -> Self {
        Self { element }
    }
}

impl Deref for HTMLElement {
    type Target = Element;
    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl core::fmt::Debug for HTMLElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "HTMLElement({:?}) at {:#?}",
            self.element.tag_name(),
            self as *const HTMLElement
        )
    }
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

    fn as_element(&self) -> Option<&Element> {
        Some(&self.element)
    }

    fn as_element_mut(&mut self) -> Option<&mut Element> {
        Some(&mut self.element)
    }
}
