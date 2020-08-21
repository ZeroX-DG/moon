use super::node::NodeData;
use super::node::NodeRef;

pub struct Document {
    URL: String,
    doctype: Option<NodeRef>
}

impl NodeData for Document {}

