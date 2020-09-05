use std::collections::HashMap;
use super::dom_token_list::DOMTokenList;
use super::node::Node;

pub struct Element {
    pub node: Node,
    attributes: HashMap<String, String>,
    id: String,
    class_list: DOMTokenList,
    tag_name: String,
}
