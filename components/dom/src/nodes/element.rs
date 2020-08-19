use crate::node::NodeRef;

pub struct Element {
    node: NodeRef,
    local_name: String,
    tag_name: String,
    id: String,
    class_name: String,
}
