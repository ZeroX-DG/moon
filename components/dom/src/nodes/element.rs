use crate::node::NodeRef;
use crate::dom_token_list::DOMTokenList;
use crate::dom_traits::Dom;

pub struct Element {
    pub node: NodeRef,
    local_name: String,
    tag_name: String,
    id: String,
    class_list: DOMTokenList
}

impl Element {
    pub fn class_name(&self) -> String {
        self.class_list.value()
    }
}

impl Dom for Element {
    fn get_node(&self) -> NodeRef {
        self.node.clone()
    }
}
