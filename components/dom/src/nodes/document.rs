use crate::node::NodeRef;
use crate::dom_traits::Dom;

pub struct Document {
    pub node: NodeRef
}

impl Dom for Document {
    fn get_node(&self) -> NodeRef {
        self.node.clone()
    }
}
