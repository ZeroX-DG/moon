use super::HTMLElement;
use dom::dom_ref::DOMObject;
use dom::element::Element;
use dom::dom_token_list::DOMTokenList;
use dom::node::Node;
use std::any::Any;

#[allow(dead_code)]
pub struct HTMLAnchorElement {
    html_element: HTMLElement,
    target: String,
    download: String,
    ping: String,
    rel: String,
    rel_list: DOMTokenList,
    hreflang: String,
    type_: String,
    text: String,
    referrer_policy: String
}

impl core::fmt::Debug for HTMLAnchorElement {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#?}", self.html_element)
    }
}

impl HTMLAnchorElement {
    pub fn new(html_element: HTMLElement) -> Self {
        Self {
            html_element,
            target: String::new(),
            download: String::new(),
            ping: String::new(),
            rel: String::new(),
            rel_list: DOMTokenList::new(),
            hreflang: String::new(),
            type_: String::new(),
            text: String::new(),
            referrer_policy: String::new()
        }
    }
}

impl DOMObject for HTMLAnchorElement {
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
