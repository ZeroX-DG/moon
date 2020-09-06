use dom::dom_ref::{NodeRef, WeakNodeRef};
use dom::element::Element;

pub fn create_element(document: WeakNodeRef, tag_name: &str) -> NodeRef {
    let mut element = Element::new(tag_name.to_owned());
    element.node.set_document(document);
    NodeRef::new(element)
}
