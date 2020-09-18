use super::dom_token_list::DOMTokenList;
use super::node::Node;
use std::collections::HashMap;

type AttributeMap = HashMap<String, String>;

#[derive(Debug)]
pub struct Element {
    pub node: Node,
    attributes: AttributeMap,
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

    pub fn attributes(&self) -> &AttributeMap {
        &self.attributes
    }

    pub fn has_attribute(&self, name: &str) -> bool {
        self.attributes.contains_key(name)
    }

    pub fn tag_name(&self) -> String {
        self.tag_name.clone()
    }
}
