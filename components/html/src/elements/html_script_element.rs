use super::HTMLElement;
use dom::dom_ref::{DOMObject, NodeRef};
use dom::element::Element;
use dom::node::Node;
use std::any::Any;
use std::borrow::Cow;

pub struct HTMLScriptElement {
    html_element: HTMLElement,
    non_blocking: bool,
    parser_document: Option<NodeRef>,
    already_started: bool,
}

impl core::fmt::Debug for HTMLScriptElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self.html_element)
    }
}

impl HTMLScriptElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            html_element,
            non_blocking: true,
            parser_document: None,
            already_started: false,
        }
    }

    pub fn src(&self) -> Cow<str> {
        self.html_element.attributes().get_str("src")
    }

    pub fn type_(&self) -> Cow<str> {
        self.html_element.attributes().get_str("type")
    }

    pub fn async_(&self) -> bool {
        self.html_element.attributes().get_bool("async")
    }

    pub fn defer(&self) -> bool {
        self.html_element.attributes().get_bool("defer")
    }

    pub fn set_non_blocking(&mut self, value: bool) {
        self.non_blocking = value;
    }

    pub fn started(&mut self) {
        self.already_started = true;
    }

    pub fn set_parser_document(&mut self, parser_document: NodeRef) {
        self.parser_document = Some(parser_document);
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

    fn as_element(&self) -> Option<&Element> {
        Some(&self.html_element.element)
    }

    fn as_element_mut(&mut self) -> Option<&mut Element> {
        Some(&mut self.html_element.element)
    }
}
