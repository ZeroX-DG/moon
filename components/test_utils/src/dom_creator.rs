use std::rc::Rc;

use css::selector::parse_selector_str;
use css::selector::structs::*;
use dom::create_element;
use dom::document::Document;
use dom::node::{Node, NodeData};
use dom::text::Text;

pub fn document() -> Rc<Node> {
    Rc::new(Node::new(NodeData::Document(Document::new())))
}

pub fn element(selector: &str, doc: Rc<Node>, children: Vec<Rc<Node>>) -> Rc<Node> {
    let selector =
        parse_selector_str(selector).expect("Unable to parse selector in test_utils#element");

    let selector = selector.values().get(0).clone().unwrap();

    let selector_parts = selector.0.values();

    let tag_name = selector_parts
        .iter()
        .find(|part| match part.selector_type() {
            SimpleSelectorType::Type => true,
            _ => false,
        })
        .expect("Unable to find tag name in test_utils#element")
        .value()
        .clone()
        .unwrap();

    let node = create_element(Rc::downgrade(&doc), &tag_name);
    let mut classes = Vec::new();

    for part in selector_parts {
        match part.selector_type() {
            SimpleSelectorType::ID => {
                node.as_element()
                    .set_attribute("id", &part.value().clone().unwrap());
            }
            SimpleSelectorType::Class => {
                classes.push(part.value().clone().unwrap());
            }
            _ => {}
        }
    }

    if classes.len() > 0 {
        node.as_element()
            .set_attribute("class", &classes.join(" ").to_string());
    }

    for child in children {
        Node::append_child(node.clone(), child.clone());
    }
    node
}

pub fn create_elemt_recursively() {}

pub fn text(value: &str, doc: Rc<Node>) -> Rc<Node> {
    let text_node = Rc::new(Node::new(NodeData::Text(Text::new(value.to_string()))));
    text_node.set_document(Rc::downgrade(&doc));
    text_node
}
