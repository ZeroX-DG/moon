use crate::node::NodeRef;

pub struct Document {
}

pub trait DocumentImpl {
    fn adoptNode(node: NodeRef) -> NodeRef;
}
