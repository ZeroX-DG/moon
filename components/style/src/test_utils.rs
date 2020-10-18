use css::cssom::stylesheet::StyleSheet;
use css::parser::Parser;
use css::selector::parse_selector_str;
use css::selector::structs::*;
use css::tokenizer::Tokenizer;
use css::tokenizer::token::Token;
use dom::dom_ref::NodeRef;
use dom::element::Element;
use dom::node::Node;
use dom::text::Text;

pub fn parse_stylesheet(style: &str) -> StyleSheet {
    let tokenizer = Tokenizer::new(style.to_string());
    let mut parser = Parser::<Token>::new(tokenizer.run());
    parser.parse_a_css_stylesheet()
}

pub fn element(selector: &str, children: Vec<NodeRef>) -> NodeRef {
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

    let mut element = Element::new(tag_name);
    let mut classes = Vec::new();

    for part in selector_parts {
        match part.selector_type() {
            SimpleSelectorType::ID => {
                element.set_attribute("id", &part.value().clone().unwrap());
            }
            SimpleSelectorType::Class => {
                classes.push(part.value().clone().unwrap());
            }
            _ => {}
        }
    }

    element.set_attribute("class", &classes.join(" ").to_string());

    let node = NodeRef::new(element);

    for child in children {
        Node::append_child(node.clone(), child.clone());
    }
    node
}

pub fn text(value: &str) -> NodeRef {
    NodeRef::new(Text::new(value.to_string()))
}
