use super::implementations::Node;
use super::node::{NodeRef, NodeType, NodeInner, WeakNodeRef};
use super::nodes::{Element, ElementData};

pub fn create_element(document: WeakNodeRef, tag_name: &str) -> NodeRef {
    let element = Element::new(ElementData::Element, tag_name.to_owned());
    let node = NodeRef::new_node(NodeType::Element, NodeInner::Element(element));
    node.set_document(document);
    node
}
