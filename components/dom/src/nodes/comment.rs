use crate::node::{NodeRef, NodeType};

pub struct Comment {
    node: NodeRef,
    data: String
}

impl Comment {
    pub fn new(data: String) -> Self {
        Self {
            node: NodeRef::new_node(NodeType::Comment),
            data
        }
    }
}
