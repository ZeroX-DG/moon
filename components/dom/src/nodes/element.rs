use crate::node::NodeRef;
use crate::dom_token_list::DOMTokenList;

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
