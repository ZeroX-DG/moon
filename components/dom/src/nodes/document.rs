use crate::node::{NodeType, NodeRef};

pub struct Document {
    node: NodeRef,
    doctype: Option<NodeRef>
}

impl Document {
    pub fn new() -> Self {
        Self {
            node: NodeRef::new_node(NodeType::Document),
            doctype: None
        }
    }
}
