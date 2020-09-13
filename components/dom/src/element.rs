use super::dom_token_list::DOMTokenList;
use super::node::Node;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Element {
    pub node: Node,
    attributes: HashMap<String, String>,
    id: String,
    class_list: DOMTokenList,
    tag_name: String,
}

impl Element {
    pub fn new(tag_name: String) -> Self {
        Self {
            node: Node::new(),
            attributes: HashMap::new(),
            id: String::new(),
            class_list: DOMTokenList::new(),
            tag_name,
        }
    }

    pub fn set_attribute(&mut self, name: &str, value: &str) {
        self.attributes.insert(name.to_owned(), value.to_owned());
    }

    pub fn tag_name(&self) -> String {
        self.tag_name.clone()
    }
}
