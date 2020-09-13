use dom::dom_ref::{NodeRef, WeakNodeRef};
use dom::element::Element;

use super::elements::HTMLBaseElement;
use super::elements::HTMLBodyElement;
use super::elements::HTMLDivElement;
use super::elements::HTMLElement;
use super::elements::HTMLHeadElement;
use super::elements::HTMLHtmlElement;
use super::elements::HTMLScriptElement;
use super::elements::HTMLTitleElement;

pub fn create_element(document: WeakNodeRef, tag_name: &str) -> NodeRef {
    let mut element = Element::new(tag_name.to_owned());
    element.node.set_document(document);

    match tag_name {
        "html" => NodeRef::new(HTMLHtmlElement::new(HTMLElement::new(element))),
        "head" => NodeRef::new(HTMLHeadElement::new(HTMLElement::new(element))),
        "base" => NodeRef::new(HTMLBaseElement::new(HTMLElement::new(element))),
        "title" => NodeRef::new(HTMLTitleElement::new(HTMLElement::new(element))),
        "body" => NodeRef::new(HTMLBodyElement::new(HTMLElement::new(element))),
        "div" => NodeRef::new(HTMLDivElement::new(HTMLElement::new(element))),
        "script" => NodeRef::new(HTMLScriptElement::new(HTMLElement::new(element))),
        _ => NodeRef::new(element),
    }
}
