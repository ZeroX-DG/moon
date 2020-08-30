use crate::node::{NodeType, NodeRef, WeakNodeRef};
use crate::node_list::NodeList;

pub trait Node {
    fn node_type(&self) -> NodeType;
    fn child_nodes(&self) -> NodeList;
    fn first_child(&self) -> Option<NodeRef>;
    fn last_child(&self) -> Option<NodeRef>;
    fn next_sibling(&self) -> Option<NodeRef>;
    fn prev_sibling(&self) -> Option<NodeRef>;
    fn owner_document(&self) -> Option<NodeRef>;
    fn set_document(&self, document: WeakNodeRef);
    fn append_child(&self, child: NodeRef);
}

impl Node for NodeRef {
    fn node_type(&self) -> NodeType {
        let ref_self = self.borrow();
        ref_self.node_type.clone()
    }

    fn first_child(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.first_child.clone()
    }

    fn last_child(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.last_child {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    fn next_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        ref_self.next_sibling.clone()
    }

    fn prev_sibling(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.prev_sibling {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    fn child_nodes(&self) -> NodeList {
        NodeList::new(self.first_child())
    }

    fn owner_document(&self) -> Option<NodeRef> {
        let ref_self = self.borrow();
        match &ref_self.owner_document {
            Some(node) => node.clone().upgrade(),
            _ => None
        }
    }

    fn set_document(&self, document: WeakNodeRef) {
        let mut ref_self = self.borrow_mut();
        ref_self.owner_document = Some(document);
    }

    fn append_child(&self, child: NodeRef) {
        let mut ref_self = self.borrow_mut();

        if let Some(last_child) = self.last_child() {
            last_child.borrow_mut().next_sibling = Some(child.clone());
        }

        child.borrow_mut().parent_node = Some(self.clone().downgrade());

        ref_self.last_child = Some(child.downgrade());
    }
}
