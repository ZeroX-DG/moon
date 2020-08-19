use crate::node::NodeRef;

pub trait Dom {
    fn get_node(&self) -> NodeRef;
}
