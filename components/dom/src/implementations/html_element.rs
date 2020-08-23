use crate::node::NodeRef;
use super::Element;

pub trait HTMLElement : Element {
}

impl HTMLElement for NodeRef {
}
